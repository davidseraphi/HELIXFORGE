//! Shared CI / isolation command policy (injection-resistant allowlist).
//!
//! Used by sandbox steps and `container::run_isolated` so host/docker never
//! execute unsanitized shell strings.

use shared_core::{HelixError, HelixResult};

pub fn ci_allow_all() -> bool {
    std::env::var("HELIX_CODE_CI_ALLOW_ALL")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Host fallback after docker failure requires explicit allow (or CI allow-all).
pub fn allow_host_fallback() -> bool {
    ci_allow_all()
        || std::env::var("HELIX_CODE_ALLOW_HOST_FALLBACK")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
}

/// Gate a command before `sh -c` / `cmd /C`.
pub fn validate_isolation_command(cmd: &str) -> HelixResult<()> {
    if !is_allowed_command(cmd) {
        return Err(HelixError::validation(format!(
            "command denied by isolation policy: {cmd}"
        )));
    }
    Ok(())
}

/// Gated command allowlist for E2 sandbox / isolation.
pub fn is_allowed_command(cmd: &str) -> bool {
    let t = cmd.trim();
    if t.is_empty() || t.contains('\0') || t.len() > 1024 {
        return false;
    }
    if has_shell_metachar(t) {
        return false;
    }
    if ci_allow_all() {
        return !hard_deny(t);
    }
    if hard_deny(t) || t.contains("..") {
        return false;
    }
    // File-read / show: only relative safe args
    if !file_args_safe(t) {
        return false;
    }

    const EXACT: &[&str] = &[
        "true",
        "echo helix-code-ci",
        "git status",
        "git rev-parse HEAD",
        "pwd",
        "whoami",
        "hostname",
        "ls",
        "dir",
    ];
    let lower = t.to_ascii_lowercase();
    if EXACT.iter().any(|e| lower == *e || t == *e) {
        return true;
    }

    const PREFIXES: &[&str] = &[
        "echo ",
        "cargo test",
        "cargo check",
        "cargo build",
        "cargo clippy",
        "git status",
        "git log",
        "git rev-parse",
        "git show ",
        "git branch",
        "git diff",
        "dir ",
        "ls ",
        "type ",
        "cat ",
        "rustc --version",
        "hostname",
        "whoami",
        "pwd",
    ];
    PREFIXES.iter().any(|p| {
        let pl = p.to_ascii_lowercase();
        t == p.trim() || t.starts_with(p) || lower.starts_with(pl.trim()) || lower.starts_with(&pl)
    })
}

fn hard_deny(cmd: &str) -> bool {
    let lower = cmd.to_ascii_lowercase();
    const BAD: &[&str] = &[
        "rm -rf",
        "rm -r ",
        "del /s",
        "format ",
        "mkfs",
        "curl ",
        "wget ",
        "powershell",
        "pwsh",
        "invoke-webrequest",
        "bash -c",
        "sh -c",
        "cmd /c",
        "python -c",
        "node -e",
        "/dev/tcp",
    ];
    BAD.iter().any(|b| lower.contains(b))
}

fn has_shell_metachar(cmd: &str) -> bool {
    // Note: do not ban bare parentheses used rarely; ban shell control operators.
    const META: &[char] = &['|', '&', ';', '`', '\n', '\r', '<', '>'];
    if cmd.chars().any(|c| META.contains(&c)) {
        return true;
    }
    if cmd.contains("$(") || cmd.contains("${") || cmd.contains("$`") {
        return true;
    }
    false
}

/// For cat/type/git show/ls: reject absolute paths and `..`.
fn file_args_safe(cmd: &str) -> bool {
    let lower = cmd.to_ascii_lowercase();
    let needs = lower.starts_with("cat ")
        || lower.starts_with("type ")
        || lower.starts_with("head ")
        || lower.starts_with("tail ")
        || lower.starts_with("git show ")
        || lower.starts_with("ls ")
        || lower.starts_with("dir ");
    if !needs {
        return true;
    }
    // skip first token
    let mut parts = cmd.split_whitespace();
    let _ = parts.next();
    for arg in parts {
        if arg.starts_with('-') {
            // flags only — allow short flags without paths
            if arg.contains('/') || arg.contains('\\') {
                return false;
            }
            continue;
        }
        if !is_safe_rel_arg(arg) {
            return false;
        }
    }
    true
}

pub fn is_safe_rel_arg(p: &str) -> bool {
    let p = p.trim().replace('\\', "/");
    if p.is_empty() || p == "." {
        return true;
    }
    // git object ids
    if p.chars().all(|c| c.is_ascii_hexdigit()) && (7..=64).contains(&p.len()) {
        return true;
    }
    // git show HEAD:path form
    if let Some((rev, path)) = p.split_once(':') {
        if rev
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/')
            && is_safe_rel_path(path)
        {
            return true;
        }
    }
    is_safe_rel_path(&p)
}

fn is_safe_rel_path(p: &str) -> bool {
    let p = p.trim().replace('\\', "/");
    if p.is_empty() || p.starts_with('/') || p.starts_with('~') {
        return false;
    }
    // Windows drive
    if p.len() >= 2 && p.as_bytes()[1] == b':' {
        return false;
    }
    if p.contains("..") || p.contains('\0') {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_echo_and_cargo() {
        assert!(is_allowed_command("echo helix-code-ci"));
        assert!(is_allowed_command("cargo test -q"));
        assert!(is_allowed_command("echo hello"));
    }

    #[test]
    fn denies_injection_and_curl() {
        assert!(!is_allowed_command("echo hi; rm -rf /"));
        assert!(!is_allowed_command("echo hi | curl http://x"));
        assert!(!is_allowed_command("curl http://evil"));
        assert!(!is_allowed_command("cat /etc/passwd"));
        assert!(!is_allowed_command("cat ../../secrets"));
        assert!(is_allowed_command("cat src/main.rs"));
    }
}
