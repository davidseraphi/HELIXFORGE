//! E2 in-forge CI sandbox runner.
//!
//! - Clones bare repo to a workdir at a commit
//! - Runs allowlisted step commands with timeout (host or Docker isolation)
//! - Collects log + optional relative-path artifacts for MinIO upload
//!
//! Break-glass: `HELIX_CODE_CI_ALLOW_ALL=1` permits any non-empty command (audited at API).
//! Isolation: `HELIX_CODE_ISOLATION=host|docker|auto` (see `container` module).

use super::container::{self, IsolationMode};
use shared_core::{HelixError, HelixResult};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct StepSpec {
    pub name: String,
    pub run: String,
}

#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub status: String,
    pub exit_code: i32,
    pub log_text: String,
    pub workdir: PathBuf,
    /// Relative paths (from workdir) that exist and should be uploaded.
    pub artifact_paths: Vec<PathBuf>,
    /// Resolved isolation for this run (`host` or `docker`).
    pub isolation: String,
}

/// Parse steps + artifact paths (literal relative paths only for E2) from pipeline definition.
pub fn parse_definition(def: &serde_json::Value) -> (Vec<StepSpec>, Vec<String>) {
    let mut steps = Vec::new();
    if let Some(arr) = def.get("steps").and_then(|s| s.as_array()) {
        for step in arr {
            let name = step
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("step")
                .to_string();
            let run = step
                .get("run")
                .and_then(|v| v.as_str())
                .unwrap_or("true")
                .to_string();
            steps.push(StepSpec { name, run });
        }
    }
    let mut artifacts = Vec::new();
    if let Some(arr) = def.get("artifacts").and_then(|a| a.as_array()) {
        for a in arr {
            if let Some(s) = a.as_str() {
                if is_safe_rel_path(s) {
                    artifacts.push(s.to_string());
                }
            }
        }
    }
    (steps, artifacts)
}

pub fn allow_all_env() -> bool {
    let on = super::cmd_policy::ci_allow_all();
    if on {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            super::breakglass::record("CI_ALLOW_ALL", "HELIX_CODE_CI_ALLOW_ALL active");
        });
    }
    on
}

/// Gated command allowlist for E2 sandbox (delegates to shared cmd_policy).
pub fn is_allowed_command(cmd: &str) -> bool {
    super::cmd_policy::is_allowed_command(cmd)
}

fn is_safe_rel_path(p: &str) -> bool {
    let p = p.trim().replace('\\', "/");
    if p.is_empty() || p.starts_with('/') || p.contains("..") {
        return false;
    }
    !p.chars().any(|c| c == '\0' || c == ':')
}

/// Clone bare repo to workdir and run steps (blocking — E2 local forge).
pub fn run_pipeline_sandbox(
    bare_repo: &Path,
    commit_sha: &str,
    steps: &[StepSpec],
    artifact_rel_paths: &[String],
    step_timeout: Duration,
) -> HelixResult<SandboxResult> {
    if !bare_repo.exists() {
        return Err(HelixError::not_found(format!(
            "bare repo missing: {}",
            bare_repo.display()
        )));
    }
    let persist_root = std::env::var("HELIX_CODE_CI_WORKDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".data/helix-code/ci-runs"));
    std::fs::create_dir_all(&persist_root)
        .map_err(|e| HelixError::internal(format!("ci workdir root: {e}")))?;
    let run_dir = persist_root.join(format!("run-{}", uuid::Uuid::now_v7().simple()));
    let bare_s = bare_repo
        .to_str()
        .ok_or_else(|| HelixError::internal("bare path utf8"))?;
    let dest_s = run_dir
        .to_str()
        .ok_or_else(|| HelixError::internal("run_dir utf8"))?;

    let mode = container::resolve_isolation();
    let mut used_mode = mode;
    let mut log = String::new();
    log.push_str(&format!(
        "E2 sandbox start bare={} commit={} allow_all={} isolation={} docker_available={}\n",
        bare_repo.display(),
        commit_sha,
        allow_all_env(),
        container::isolation_label(mode),
        container::docker_available()
    ));

    let clone = Command::new("git")
        .args(["clone", bare_s, dest_s])
        .output()
        .map_err(|e| HelixError::dependency(format!("git clone: {e}")))?;
    if !clone.status.success() {
        return Err(HelixError::dependency(format!(
            "git clone failed: {}",
            String::from_utf8_lossy(&clone.stderr)
        )));
    }
    let co = Command::new("git")
        .current_dir(&run_dir)
        .args(["checkout", "--force", commit_sha])
        .output();
    if let Ok(co) = co {
        if !co.status.success() {
            log.push_str(&format!(
                "checkout {commit_sha} note: {}\n",
                String::from_utf8_lossy(&co.stderr)
            ));
        }
    }

    let mut overall_exit = 0i32;
    let mut status = "succeeded".to_string();

    if steps.is_empty() {
        log.push_str("no steps; noop success\n");
    }

    for step in steps {
        log.push_str(&format!("== {} ==\n$ {}\n", step.name, step.run));
        if !is_allowed_command(&step.run) {
            log.push_str("DENIED by E2 allowlist (set HELIX_CODE_CI_ALLOW_ALL=1 to break-glass)\n");
            overall_exit = 126;
            status = "failed".into();
            break;
        }
        // Docker alpine lacks git/cargo; host for git-prefixed steps when image is alpine.
        let step_mode = step_isolation_mode(mode, &step.run);
        match container::run_isolated(&run_dir, &step.run, step_timeout, step_mode) {
            Ok((code, out, err, actual)) => {
                used_mode = actual;
                log.push_str(&format!(
                    "isolation={}\n",
                    container::isolation_label(actual)
                ));
                if !out.is_empty() {
                    log.push_str(&out);
                    if !out.ends_with('\n') {
                        log.push('\n');
                    }
                }
                if !err.is_empty() {
                    log.push_str(&err);
                    if !err.ends_with('\n') {
                        log.push('\n');
                    }
                }
                log.push_str(&format!("exit={code}\n"));
                if code != 0 {
                    overall_exit = code;
                    status = "failed".into();
                    break;
                }
            }
            Err(e) => {
                log.push_str(&format!("step error: {e}\n"));
                overall_exit = 1;
                status = "failed".into();
                break;
            }
        }
    }

    let mut artifact_paths = Vec::new();
    let log_path = run_dir.join("helix-ci.log");
    for rel in artifact_rel_paths {
        if !is_safe_rel_path(rel) {
            log.push_str(&format!("skip unsafe artifact path: {rel}\n"));
            continue;
        }
        let full = run_dir.join(rel);
        if full.is_file() {
            artifact_paths.push(PathBuf::from(rel.replace('\\', "/")));
        } else {
            log.push_str(&format!("artifact missing: {rel}\n"));
        }
    }
    let _ = std::fs::write(&log_path, &log);
    artifact_paths.insert(0, PathBuf::from("helix-ci.log"));

    Ok(SandboxResult {
        status,
        exit_code: overall_exit,
        log_text: log,
        workdir: run_dir,
        artifact_paths,
        isolation: container::isolation_label(used_mode).to_string(),
    })
}

/// Prefer host for git/cargo/rustc when the resolved image is alpine (no forge tools).
/// Full `helixforge/helix-code-ci:local` keeps those steps in Docker.
fn step_isolation_mode(requested: IsolationMode, cmd: &str) -> IsolationMode {
    if requested != IsolationMode::Docker {
        return requested;
    }
    if container::image_has_forge_tools() {
        return IsolationMode::Docker;
    }
    let lower = cmd.trim().to_ascii_lowercase();
    let needs_host = lower.starts_with("git ")
        || lower.starts_with("cargo ")
        || lower.starts_with("rustc ")
        || lower == "git status"
        || lower.starts_with("git")
        || lower.starts_with("cargo")
        || lower.starts_with("rustc");
    if needs_host {
        IsolationMode::Host
    } else {
        IsolationMode::Docker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_echo_and_deny_curl() {
        assert!(is_allowed_command("echo hello"));
        assert!(is_allowed_command("cargo test -q"));
        assert!(!is_allowed_command("curl http://evil"));
        assert!(!is_allowed_command("rm -rf /"));
    }

    #[test]
    fn sandbox_runs_echo_step() {
        let dir = tempfile::tempdir().unwrap();
        let bare = dir.path().join("r.git");
        assert!(Command::new("git")
            .args(["init", "--bare", bare.to_str().unwrap()])
            .status()
            .unwrap()
            .success());
        let wt = dir.path().join("wt");
        assert!(Command::new("git")
            .args(["clone", bare.to_str().unwrap(), wt.to_str().unwrap()])
            .status()
            .unwrap()
            .success());
        std::fs::write(wt.join("README.md"), "hi\n").unwrap();
        assert!(Command::new("git")
            .current_dir(&wt)
            .args(["config", "user.email", "t@t"])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .current_dir(&wt)
            .args(["config", "user.name", "t"])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .current_dir(&wt)
            .args(["add", "."])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .current_dir(&wt)
            .args(["commit", "-m", "i"])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .current_dir(&wt)
            .args(["push", "origin", "HEAD:main"])
            .status()
            .unwrap()
            .success());
        let sha = Command::new("git")
            .current_dir(&wt)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap();
        let sha = String::from_utf8_lossy(&sha.stdout).trim().to_string();

        std::env::set_var(
            "HELIX_CODE_CI_WORKDIR",
            dir.path().join("ci").to_str().unwrap(),
        );
        // Unit tests always use host isolation (Docker volume mounts on Windows temps are flaky).
        std::env::set_var("HELIX_CODE_ISOLATION", "host");
        std::env::set_var("HELIX_CODE_ALLOW_HOST_ISOLATION", "1");
        let res = run_pipeline_sandbox(
            &bare,
            &sha,
            &[StepSpec {
                name: "hello".into(),
                run: "echo helix-code-ci".into(),
            }],
            &[],
            Duration::from_secs(30),
        )
        .expect("sandbox");
        assert_eq!(res.status, "succeeded");
        assert!(
            res.log_text.contains("helix-code-ci") || res.log_text.contains("exit=0"),
            "log={}",
            res.log_text
        );
        assert!(res.workdir.join("helix-ci.log").exists());
    }
}
