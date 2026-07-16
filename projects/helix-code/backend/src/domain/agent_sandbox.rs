//! E4 multi-agent sandbox mesh.
//!
//! Isolated worktree per job; structured file patches (and optional unified diffs);
//! optional multi-agent steps via `agent_framework`. Commits only when requested.

use shared_core::{HelixError, HelixResult};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FilePatch {
    pub path: String,
    pub content: String,
    #[serde(default)]
    pub create: bool,
}

#[derive(Debug, Clone)]
pub struct MeshStepResult {
    pub agent: String,
    pub status: String,
    pub summary: String,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentSandboxResult {
    pub status: String,
    pub log_text: String,
    pub workdir: PathBuf,
    pub commit_sha: Option<String>,
    pub files_changed: Vec<String>,
    pub mesh_steps: Vec<MeshStepResult>,
    /// Isolation used for optional shell verify step.
    pub isolation: String,
}

pub fn is_safe_rel_path(path: &str) -> bool {
    let p = path.trim().replace('\\', "/");
    if p.is_empty() || p.starts_with('/') || p.contains("..") || p.contains('\0') {
        return false;
    }
    // No drive letters / absolute Windows
    if p.contains(':') {
        return false;
    }
    true
}

/// Clone bare repo into a job workdir and checkout commit/branch.
pub fn prepare_worktree(bare: &Path, rev: &str) -> HelixResult<PathBuf> {
    let root = std::env::var("HELIX_CODE_AGENT_WORKDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".data/helix-code/agent-jobs"));
    std::fs::create_dir_all(&root)
        .map_err(|e| HelixError::internal(format!("agent workdir: {e}")))?;
    let dest = root.join(format!("job-{}", uuid::Uuid::now_v7().simple()));
    let bare_s = bare
        .to_str()
        .ok_or_else(|| HelixError::internal("bare utf8"))?;
    let dest_s = dest
        .to_str()
        .ok_or_else(|| HelixError::internal("dest utf8"))?;
    let out = Command::new("git")
        .args(["clone", bare_s, dest_s])
        .output()
        .map_err(|e| HelixError::dependency(format!("git clone agent: {e}")))?;
    if !out.status.success() {
        return Err(HelixError::dependency(format!(
            "git clone agent: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    let _ = Command::new("git")
        .current_dir(&dest)
        .args(["checkout", "--force", rev])
        .output();
    let _ = Command::new("git")
        .current_dir(&dest)
        .args(["config", "user.email", "agent@helixforge.local"])
        .output();
    let _ = Command::new("git")
        .current_dir(&dest)
        .args(["config", "user.name", "HelixCode Agent"])
        .output();
    Ok(dest)
}

/// Apply full-file patches into worktree. Returns list of relative paths written.
pub fn apply_file_patches(workdir: &Path, patches: &[FilePatch]) -> HelixResult<Vec<String>> {
    let mut changed = Vec::new();
    for p in patches {
        if !is_safe_rel_path(&p.path) {
            return Err(HelixError::validation(format!(
                "unsafe patch path: {}",
                p.path
            )));
        }
        let dest = workdir.join(&p.path);
        if dest.exists() && !p.create {
            // overwrite allowed
        } else if !dest.exists() && !p.create {
            // still allow create-by-write for convenience in E4
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| HelixError::internal(format!("mkdir patch: {e}")))?;
        }
        std::fs::write(&dest, p.content.as_bytes())
            .map_err(|e| HelixError::internal(format!("write patch {}: {e}", p.path)))?;
        changed.push(p.path.replace('\\', "/"));
    }
    Ok(changed)
}

/// Apply unified diff via `git apply --check` then `git apply`. Diff must not escape worktree.
pub fn apply_unified_diff(workdir: &Path, diff: &str) -> HelixResult<Vec<String>> {
    if diff.contains("\0") {
        return Err(HelixError::validation("diff contains NUL"));
    }
    // Reject absolute paths in diff headers
    for line in diff.lines() {
        if line.starts_with("+++ ") || line.starts_with("--- ") {
            if line.contains(":/") || line.contains(":\\") {
                return Err(HelixError::validation("diff must use relative paths only"));
            }
            if line.contains("..") {
                return Err(HelixError::validation("diff path traversal denied"));
            }
        }
    }
    let patch_file = workdir.join(".helix-agent.patch");
    std::fs::write(&patch_file, diff)
        .map_err(|e| HelixError::internal(format!("write patch file: {e}")))?;
    let check = Command::new("git")
        .current_dir(workdir)
        .args(["apply", "--check", ".helix-agent.patch"])
        .output()
        .map_err(|e| HelixError::dependency(format!("git apply --check: {e}")))?;
    if !check.status.success() {
        return Err(HelixError::validation(format!(
            "git apply --check failed: {}",
            String::from_utf8_lossy(&check.stderr)
        )));
    }
    let apply = Command::new("git")
        .current_dir(workdir)
        .args(["apply", ".helix-agent.patch"])
        .output()
        .map_err(|e| HelixError::dependency(format!("git apply: {e}")))?;
    if !apply.status.success() {
        return Err(HelixError::dependency(format!(
            "git apply failed: {}",
            String::from_utf8_lossy(&apply.stderr)
        )));
    }
    let _ = std::fs::remove_file(&patch_file);
    // List changed via git status
    let st = Command::new("git")
        .current_dir(workdir)
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| HelixError::dependency(format!("git status: {e}")))?;
    let mut files = Vec::new();
    for line in String::from_utf8_lossy(&st.stdout).lines() {
        let path = line.get(3..).unwrap_or("").trim();
        if !path.is_empty() {
            files.push(path.replace('\\', "/"));
        }
    }
    Ok(files)
}

pub fn commit_and_push(
    workdir: &Path,
    bare: &Path,
    branch: &str,
    message: &str,
) -> HelixResult<String> {
    let add = Command::new("git")
        .current_dir(workdir)
        .args(["add", "-A"])
        .output()
        .map_err(|e| HelixError::dependency(format!("git add: {e}")))?;
    if !add.status.success() {
        return Err(HelixError::dependency(format!(
            "git add: {}",
            String::from_utf8_lossy(&add.stderr)
        )));
    }
    // Empty commit?
    let st = Command::new("git")
        .current_dir(workdir)
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| HelixError::dependency(format!("git status: {e}")))?;
    if st.stdout.is_empty() {
        // still return HEAD
        return rev_parse_head(workdir);
    }
    let commit = Command::new("git")
        .current_dir(workdir)
        .args(["commit", "-m", message])
        .output()
        .map_err(|e| HelixError::dependency(format!("git commit: {e}")))?;
    if !commit.status.success() {
        return Err(HelixError::dependency(format!(
            "git commit: {}",
            String::from_utf8_lossy(&commit.stderr)
        )));
    }
    let bare_s = bare
        .to_str()
        .ok_or_else(|| HelixError::internal("bare utf8"))?;
    // Ensure remote origin points at bare (clone already set this)
    let push = Command::new("git")
        .current_dir(workdir)
        .args(["push", "origin", &format!("HEAD:{branch}")])
        .output()
        .map_err(|e| HelixError::dependency(format!("git push: {e}")))?;
    if !push.status.success() {
        // try setting remote
        let _ = Command::new("git")
            .current_dir(workdir)
            .args(["remote", "set-url", "origin", bare_s])
            .output();
        let push2 = Command::new("git")
            .current_dir(workdir)
            .args(["push", "origin", &format!("HEAD:{branch}")])
            .output()
            .map_err(|e| HelixError::dependency(format!("git push: {e}")))?;
        if !push2.status.success() {
            return Err(HelixError::dependency(format!(
                "git push: {}",
                String::from_utf8_lossy(&push2.stderr)
            )));
        }
    }
    rev_parse_head(workdir)
}

fn rev_parse_head(workdir: &Path) -> HelixResult<String> {
    let out = Command::new("git")
        .current_dir(workdir)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| HelixError::dependency(format!("rev-parse: {e}")))?;
    if !out.status.success() {
        return Err(HelixError::dependency("rev-parse HEAD failed"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Deterministic “analyzer” agent step without LLM: inventory worktree + prompt echo.
pub fn local_analyze_step(workdir: &Path, prompt: &str) -> MeshStepResult {
    let mut files = 0usize;
    if let Ok(rd) = std::fs::read_dir(workdir) {
        for e in rd.flatten() {
            if e.path().is_file() {
                files += 1;
            }
        }
    }
    let head = rev_parse_head(workdir).unwrap_or_else(|_| "unknown".into());
    MeshStepResult {
        agent: "local-analyzer".into(),
        status: "succeeded".into(),
        summary: format!(
            "worktree_files≈{files} head={} prompt_chars={}",
            &head[..head.len().min(12)],
            prompt.len()
        ),
        run_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_paths() {
        assert!(!is_safe_rel_path("../x"));
        assert!(!is_safe_rel_path("C:/x"));
        assert!(is_safe_rel_path("src/lib.rs"));
    }

    #[test]
    fn apply_patch_and_commit() {
        let dir = tempfile::tempdir().unwrap();
        let bare = dir.path().join("r.git");
        assert!(Command::new("git")
            .args(["init", "--bare", bare.to_str().unwrap()])
            .status()
            .unwrap()
            .success());
        let wt0 = dir.path().join("seed");
        assert!(Command::new("git")
            .args(["clone", bare.to_str().unwrap(), wt0.to_str().unwrap()])
            .status()
            .unwrap()
            .success());
        std::fs::write(wt0.join("README.md"), "hi\n").unwrap();
        assert!(Command::new("git")
            .current_dir(&wt0)
            .args(["config", "user.email", "t@t"])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .current_dir(&wt0)
            .args(["config", "user.name", "t"])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .current_dir(&wt0)
            .args(["add", "."])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .current_dir(&wt0)
            .args(["commit", "-m", "i"])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .current_dir(&wt0)
            .args(["push", "origin", "HEAD:main"])
            .status()
            .unwrap()
            .success());

        std::env::set_var(
            "HELIX_CODE_AGENT_WORKDIR",
            dir.path().join("jobs").to_str().unwrap(),
        );
        let wt = prepare_worktree(&bare, "main").unwrap();
        let changed = apply_file_patches(
            &wt,
            &[FilePatch {
                path: "src/hello.rs".into(),
                content: "pub fn hello() -> &'static str { \"e4\" }\n".into(),
                create: true,
            }],
        )
        .unwrap();
        assert_eq!(changed, vec!["src/hello.rs".to_string()]);
        let sha = commit_and_push(&wt, &bare, "main", "feat: agent patch").unwrap();
        assert!(sha.len() >= 7);
    }
}
