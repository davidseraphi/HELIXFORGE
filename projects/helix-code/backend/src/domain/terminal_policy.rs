//! Terminal command policy — **allowlist** first (not a weak denylist).
//!
//! Break-glass: `HELIX_CODE_TERM_ALLOW_ALL=1` (still rejects NUL / empty / hard deny).
//! File readers (`cat`/`type`/`head`/`tail`) only accept **relative** safe paths.

use shared_core::{HelixError, HelixResult};

pub fn term_allow_all() -> bool {
    let on = std::env::var("HELIX_CODE_TERM_ALLOW_ALL")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if on {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            super::breakglass::record("TERM_ALLOW_ALL", "HELIX_CODE_TERM_ALLOW_ALL active");
        });
    }
    on
}

/// Validate and return normalized command or error.
#[allow(dead_code)]
pub fn validate_terminal_command(cmd: &str) -> HelixResult<()> {
    validate_terminal_command_ext(cmd, term_allow_all())
}

/// `tenant_allow_all` enables per-tenant break-glass without process env.
pub fn validate_terminal_command_ext(cmd: &str, tenant_allow_all: bool) -> HelixResult<()> {
    let t = cmd.trim();
    if t.is_empty() || t.contains('\0') {
        return Err(HelixError::validation("bad command"));
    }
    if t.len() > 512 {
        return Err(HelixError::validation("command too long (max 512)"));
    }
    if tenant_allow_all || term_allow_all() {
        if tenant_allow_all && !term_allow_all() {
            super::breakglass::record("TERM_ALLOW_ALL_TENANT", "tenant allow_term_all");
        }
        if hard_deny(t) || has_shell_metachar(t) {
            return Err(HelixError::validation(
                "command denied (hard deny even with TERM_ALLOW_ALL)",
            ));
        }
        return Ok(());
    }
    if hard_deny(t) {
        return Err(HelixError::validation("command denied by terminal policy"));
    }
    if has_shell_metachar(t) {
        return Err(HelixError::validation(
            "command denied: shell metacharacters not allowed in terminal",
        ));
    }
    if !is_allowlisted(t) {
        return Err(HelixError::validation(
            "command denied: not on terminal allowlist (set HELIX_CODE_TERM_ALLOW_ALL=1 to break-glass)",
        ));
    }
    if !file_args_safe(t) {
        return Err(HelixError::validation(
            "command denied: absolute paths and '..' not allowed for file readers",
        ));
    }
    Ok(())
}

fn hard_deny(cmd: &str) -> bool {
    let lower = cmd.to_ascii_lowercase();
    const BAD: &[&str] = &[
        "rm -rf",
        "rm -r ",
        "rmdir /s",
        "rd /s",
        "del /s",
        "del /f",
        "format ",
        "mkfs",
        "diskpart",
        "shutdown",
        "reboot",
        "curl ",
        "wget ",
        "powershell",
        "pwsh",
        "invoke-webrequest",
        "invoke-expression",
        "iwr ",
        "iex ",
        "certutil",
        "bitsadmin",
        "reg add",
        "reg delete",
        "net user",
        "net localgroup",
        "schtasks",
        "wmic ",
        "bash -c",
        "sh -c",
        "cmd /c",
        "cmd.exe",
        "start /b",
        "nc ",
        "ncat ",
        "socat ",
        "python -c",
        "python3 -c",
        "perl -e",
        "ruby -e",
        "node -e",
        "eval ",
        "/dev/tcp",
        "base64 -d",
        "base64 --decode",
    ];
    BAD.iter().any(|b| lower.contains(b))
}

fn has_shell_metachar(cmd: &str) -> bool {
    const META: &[char] = &['|', '&', ';', '`', '\n', '\r', '<', '>'];
    if cmd.chars().any(|c| META.contains(&c)) {
        return true;
    }
    if cmd.contains("$(") || cmd.contains("${") {
        return true;
    }
    let lower = cmd.to_ascii_lowercase();
    if lower.contains("-enc") || lower.contains("-encodedcommand") {
        return true;
    }
    false
}

fn is_allowlisted(cmd: &str) -> bool {
    let t = cmd.trim();
    let lower = t.to_ascii_lowercase();

    const EXACT: &[&str] = &[
        "true",
        "pwd",
        "whoami",
        "hostname",
        "git status",
        "git rev-parse head",
        "ls",
        "dir",
        "echo helix-term",
        "rustc --version",
        "cargo --version",
    ];
    if EXACT.iter().any(|e| lower == *e) {
        return true;
    }

    const PREFIXES: &[&str] = &[
        "echo ",
        "git status",
        "git log",
        "git rev-parse",
        "git show ",
        "git branch",
        "git diff",
        "ls ",
        "dir ",
        "type ",
        "cat ",
        "head ",
        "tail ",
        "rustc --version",
        "cargo --version",
        "cargo check",
        "cargo test",
        "cargo build",
        "cargo clippy",
    ];
    PREFIXES
        .iter()
        .any(|p| lower == p.trim() || lower.starts_with(p) || t.starts_with(p))
}

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
    super::cmd_policy::is_allowed_command(cmd) || {
        // re-check args only
        let mut parts = cmd.split_whitespace();
        let _ = parts.next();
        for arg in parts {
            if arg.starts_with('-') {
                continue;
            }
            if !super::cmd_policy::is_safe_rel_arg(arg) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_echo_helix_term() {
        assert!(validate_terminal_command("echo helix-term").is_ok());
    }

    #[test]
    fn denies_powershell_and_rm() {
        assert!(validate_terminal_command("powershell -Command rm -rf C:\\").is_err());
        assert!(validate_terminal_command("rm -rf /").is_err());
        assert!(validate_terminal_command("rmdir /s /q .").is_err());
    }

    #[test]
    fn denies_shell_metachar() {
        assert!(validate_terminal_command("echo hi | del /s").is_err());
        assert!(validate_terminal_command("echo hi && format c:").is_err());
        assert!(validate_terminal_command("echo $(whoami)").is_err());
    }

    #[test]
    fn denies_unknown_binary() {
        assert!(validate_terminal_command("curl http://evil").is_err());
        assert!(validate_terminal_command("my-custom-tool").is_err());
    }

    #[test]
    fn denies_absolute_cat() {
        assert!(validate_terminal_command("cat /etc/passwd").is_err());
        assert!(validate_terminal_command("cat C:\\Windows\\win.ini").is_err());
        assert!(validate_terminal_command("cat src/lib.rs").is_ok());
    }
}
