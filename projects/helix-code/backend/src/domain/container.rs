//! Container isolation for CI / agent worktrees (Docker when available).
//!
//! Modes:
//! - `host` — process on host (default allowlist)
//! - `docker` — `docker run --rm -v workdir:/work -w /work image sh -c …`
//!
//! Env:
//! - `HELIX_CODE_ISOLATION=host|docker|auto` (auto = docker if daemon up)
//! - `HELIX_CODE_DOCKER_IMAGE` — preferred image (default `helixforge/helix-code-ci:local`,
//!   then falls back to `alpine:3.20` if the CI image is not built yet)

use shared_core::{HelixError, HelixResult};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::Duration;

/// Preferred full CI image (git + cargo). Build via `docker/build-ci-image.ps1`.
pub const CI_IMAGE_PREFERRED: &str = "helixforge/helix-code-ci:local";
/// Minimal fallback when preferred image is not present locally.
pub const CI_IMAGE_FALLBACK: &str = "alpine:3.20";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationMode {
    Host,
    Docker,
}

/// True when intentional host isolation is allowed (env).
pub fn allow_host_isolation_env() -> bool {
    std::env::var("HELIX_CODE_ALLOW_HOST_ISOLATION")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
        || super::cmd_policy::ci_allow_all()
}

pub fn resolve_isolation() -> IsolationMode {
    match std::env::var("HELIX_CODE_ISOLATION")
        .unwrap_or_else(|_| "auto".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "docker" | "container" => IsolationMode::Docker,
        "host" | "process" => {
            // Intentional host is privileged — require allow flag; else prefer docker.
            if allow_host_isolation_env() {
                IsolationMode::Host
            } else if docker_available() {
                super::breakglass::record(
                    "HOST_ISOLATION_REDIRECT",
                    "HELIX_CODE_ISOLATION=host ignored without ALLOW_HOST_ISOLATION; using docker",
                );
                IsolationMode::Docker
            } else {
                super::breakglass::record(
                    "HOST_ISOLATION_FORCED",
                    "no docker; host isolation used without ALLOW_HOST_ISOLATION",
                );
                IsolationMode::Host
            }
        }
        _ => {
            if docker_available() {
                IsolationMode::Docker
            } else {
                IsolationMode::Host
            }
        }
    }
}

/// Terminal isolation: prefer docker when available (reduces host process risk).
pub fn resolve_terminal_isolation() -> IsolationMode {
    match std::env::var("HELIX_CODE_TERM_ISOLATION")
        .unwrap_or_else(|_| "auto".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "host" | "process" => {
            if allow_host_isolation_env() {
                IsolationMode::Host
            } else if docker_available() {
                IsolationMode::Docker
            } else {
                IsolationMode::Host
            }
        }
        "docker" | "container" => IsolationMode::Docker,
        _ => {
            if docker_available() {
                IsolationMode::Docker
            } else {
                IsolationMode::Host
            }
        }
    }
}

pub fn docker_available() -> bool {
    Command::new("docker")
        .args(["info", "--format", "{{.ServerVersion}}"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// True when `docker image inspect` succeeds for the given tag.
pub fn docker_image_present(image: &str) -> bool {
    Command::new("docker")
        .args(["image", "inspect", image])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Resolved image name for this process (cached).
pub fn docker_image() -> String {
    static IMAGE: OnceLock<String> = OnceLock::new();
    IMAGE
        .get_or_init(|| {
            if let Ok(explicit) = std::env::var("HELIX_CODE_DOCKER_IMAGE") {
                if !explicit.trim().is_empty() {
                    return explicit.trim().to_string();
                }
            }
            // Prefer full CI image when built; else alpine.
            if docker_available() && docker_image_present(CI_IMAGE_PREFERRED) {
                CI_IMAGE_PREFERRED.to_string()
            } else {
                CI_IMAGE_FALLBACK.to_string()
            }
        })
        .clone()
}

/// Whether the resolved image is the full helix-code-ci (git/cargo onboard).
pub fn image_has_forge_tools() -> bool {
    let img = docker_image();
    img.contains("helix-code-ci")
        || std::env::var("HELIX_CODE_DOCKER_HAS_TOOLS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
}

pub fn isolation_label(mode: IsolationMode) -> &'static str {
    match mode {
        IsolationMode::Host => "host",
        IsolationMode::Docker => "docker",
    }
}

/// Normalize a host path for Docker Desktop volume binds.
///
/// Windows `canonicalize()` yields `\\?\C:\...` which Docker rejects. We strip
/// the extended prefix and, for Linux containers on Docker Desktop, convert
/// `C:\foo` → `/c/foo` (also accepts `C:/foo`).
pub fn docker_bind_host_path(workdir: &Path) -> HelixResult<String> {
    let abs = if workdir.is_absolute() {
        workdir.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|c| c.join(workdir))
            .unwrap_or_else(|_| workdir.to_path_buf())
    };
    // Prefer canonicalize for symlink resolution, but strip extended prefixes.
    let canon = abs.canonicalize().unwrap_or(abs);
    let raw = canon.to_string_lossy().into_owned();
    Ok(normalize_docker_host_path(&raw))
}

/// Pure path normalizer (unit-tested).
pub fn normalize_docker_host_path(raw: &str) -> String {
    let mut s = raw.to_string();
    // Strip Windows extended-length prefixes
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        s = format!(r"\\{rest}");
    } else if let Some(rest) = s.strip_prefix(r"\\?\") {
        s = rest.to_string();
    } else if let Some(rest) = s.strip_prefix("//?/") {
        // forward-slash variant
        s = rest.to_string();
    }

    #[cfg(windows)]
    {
        // Docker Desktop Linux engine wants /c/Users/... style for host binds.
        // Also accept native C:\... — convert both to /c/...
        let mut p = s.replace('\\', "/");
        // Drive letter form: C:/Users/... or C:Users
        if p.len() >= 2 {
            let bytes = p.as_bytes();
            if bytes[1] == b':' {
                let drive = (bytes[0] as char).to_ascii_lowercase();
                let rest = if p.len() > 2 {
                    // drop "C:" and optional leading /
                    let r = &p[2..];
                    if r.starts_with('/') {
                        r.to_string()
                    } else {
                        format!("/{r}")
                    }
                } else {
                    String::new()
                };
                p = format!("/{drive}{rest}");
            }
        }
        // Collapse duplicate slashes (except leading // for UNC we already handled)
        while p.contains("//") {
            p = p.replace("//", "/");
        }
        p
    }
    #[cfg(not(windows))]
    {
        s
    }
}

/// Run a shell command either on host or inside Docker mounted at /work.
///
/// Commands are gated by `cmd_policy` before any shell invocation (injection control).
pub fn run_isolated(
    workdir: &Path,
    cmd: &str,
    timeout: Duration,
    mode: IsolationMode,
) -> HelixResult<(i32, String, String, IsolationMode)> {
    // Always gate before sh -c / cmd /C
    super::cmd_policy::validate_isolation_command(cmd)?;
    match mode {
        IsolationMode::Host => {
            let (c, o, e) = run_host(workdir, cmd, timeout)?;
            Ok((c, o, e, IsolationMode::Host))
        }
        IsolationMode::Docker => match run_docker(workdir, cmd, timeout) {
            Ok((c, o, e)) => Ok((c, o, e, IsolationMode::Docker)),
            Err(e) => {
                // Forced docker: fail closed
                if std::env::var("HELIX_CODE_ISOLATION")
                    .map(|v| v.eq_ignore_ascii_case("docker"))
                    .unwrap_or(false)
                {
                    return Err(e);
                }
                // Host fallback is privileged — require explicit allow
                if !super::cmd_policy::allow_host_fallback() {
                    super::breakglass::record(
                        "HOST_FALLBACK_DENIED",
                        &format!("docker failed; set HELIX_CODE_ALLOW_HOST_FALLBACK=1: {e}"),
                    );
                    return Err(HelixError::dependency(format!(
                        "docker isolation failed and host fallback denied (set HELIX_CODE_ALLOW_HOST_FALLBACK=1 or HELIX_CODE_CI_ALLOW_ALL=1): {e}"
                    )));
                }
                super::breakglass::record(
                    "HOST_FALLBACK",
                    &format!("docker failed, running on host: {e}"),
                );
                let (c, o, e2) = run_host(workdir, cmd, timeout)?;
                Ok((
                    c,
                    format!("docker fallback (break-glass): {e}\n{o}"),
                    e2,
                    IsolationMode::Host,
                ))
            }
        },
    }
}

fn run_host(workdir: &Path, cmd: &str, timeout: Duration) -> HelixResult<(i32, String, String)> {
    use std::sync::mpsc;
    let workdir = workdir.to_path_buf();
    let cmd = cmd.to_string();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let r = exec_host_once(&workdir, &cmd);
        let _ = tx.send(r);
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(HelixError::dependency(format!(
            "host step timed out after {}s",
            timeout.as_secs()
        ))),
    }
}

fn exec_host_once(workdir: &Path, cmd: &str) -> HelixResult<(i32, String, String)> {
    #[cfg(windows)]
    let output = Command::new("cmd")
        .arg("/C")
        .arg(cmd)
        .current_dir(workdir)
        .output()
        .map_err(|e| HelixError::dependency(format!("spawn cmd: {e}")))?;
    #[cfg(not(windows))]
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(workdir)
        .output()
        .map_err(|e| HelixError::dependency(format!("spawn sh: {e}")))?;
    Ok((
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

fn run_docker(workdir: &Path, cmd: &str, timeout: Duration) -> HelixResult<(i32, String, String)> {
    // Ensure workdir exists before bind
    if !workdir.exists() {
        std::fs::create_dir_all(workdir)
            .map_err(|e| HelixError::internal(format!("create workdir: {e}")))?;
    }
    let host_bind = docker_bind_host_path(workdir)?;
    let image = docker_image();
    let vol = format!("{host_bind}:/work");
    use std::sync::mpsc;
    let cmd_owned = cmd.to_string();
    let image_c = image.clone();
    let vol_c = vol.clone();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        // Prefer --mount for clearer errors; -v kept as primary for alpine compatibility
        let output = Command::new("docker")
            .args([
                "run",
                "--rm",
                "--network",
                "none",
                "-v",
                &vol_c,
                "-w",
                "/work",
                &image_c,
                "sh",
                "-c",
                &cmd_owned,
            ])
            .output()
            .map_err(|e| HelixError::dependency(format!("docker run: {e}")));
        let _ = tx.send(output.map(|o| (o, vol_c)));
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok((output, vol_used))) => {
            let code = output.status.code().unwrap_or(1);
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if code != 0 && stderr.is_empty() {
                stderr = format!("docker exit={code} vol={vol_used}");
            } else if code != 0 {
                stderr = format!("{stderr}\n(vol={vol_used})");
            }
            Ok((code, stdout, stderr))
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(HelixError::dependency(format!(
            "docker step timed out after {}s",
            timeout.as_secs()
        ))),
    }
}

/// Absolute path helper for tests / callers.
#[allow(dead_code)]
pub fn ensure_abs(workdir: &Path) -> PathBuf {
    if workdir.is_absolute() {
        workdir.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|c| c.join(workdir))
            .unwrap_or_else(|_| workdir.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_isolation_is_stable() {
        let m = resolve_isolation();
        let _ = isolation_label(m);
        let _ = docker_image();
    }

    #[test]
    fn preferred_ci_image_constant() {
        assert!(CI_IMAGE_PREFERRED.contains("helix-code-ci"));
        assert!(CI_IMAGE_FALLBACK.contains("alpine"));
    }

    #[test]
    fn strips_windows_extended_prefix() {
        let n = normalize_docker_host_path(r"\\?\C:\Users\divin\work");
        #[cfg(windows)]
        {
            assert_eq!(n, "/c/Users/divin/work");
        }
        #[cfg(not(windows))]
        {
            // non-windows still strips \\?\
            assert!(!n.starts_with(r"\\?\"));
        }
    }

    #[test]
    fn normalizes_forward_slash_drive() {
        let n = normalize_docker_host_path(r"C:/Users/foo/bar");
        #[cfg(windows)]
        assert_eq!(n, "/c/Users/foo/bar");
        #[cfg(not(windows))]
        let _ = n;
    }
}
