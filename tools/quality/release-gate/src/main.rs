//! Release gate — runs fresh checks against the exact candidate, records
//! commands/environment/outputs, and treats skipped required checks as blocking.

use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "release-gate")]
#[command(about = "Run fresh release checks and produce evidence")]
struct Cli {
    #[command(subcommand)]
    command: GateCommand,

    /// Skip a check by name. Required checks block the gate if skipped.
    #[arg(long = "skip")]
    skip: Vec<String>,
}

#[derive(Subcommand)]
enum GateCommand {
    /// Run all checks and exit non-zero if the gate is blocked.
    Run,
    /// Run checks and produce a report without side effects (does not block).
    DryRun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CheckState {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckResult {
    name: String,
    required: bool,
    state: CheckState,
    command: Vec<String>,
    duration_ms: u64,
    stdout: String,
    stderr: String,
    skip_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GateReport {
    passed: bool,
    dry_run: bool,
    command: Vec<String>,
    started_at: String,
    finished_at: String,
    duration_ms: u64,
    git_commit: Option<String>,
    environment: BTreeMap<String, String>,
    artifact_hashes: BTreeMap<String, String>,
    checks: Vec<CheckResult>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let dry_run = matches!(cli.command, GateCommand::DryRun);

    let report = run_gate(dry_run, &cli.skip).await?;

    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");

    if report.passed || dry_run {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

async fn run_gate(dry_run: bool, skip_list: &[String]) -> anyhow::Result<GateReport> {
    let started = Instant::now();
    let started_at = Utc::now().to_rfc3339();

    let mut checks: Vec<CheckResult> = Vec::new();

    // Required checks.
    checks.push(
        run_check(
            "cargo_build",
            true,
            &["cargo", "build", "--workspace"],
            skip_list,
        )
        .await,
    );
    checks.push(
        run_check(
            "cargo_test",
            true,
            &["cargo", "test", "--workspace", "--all-features"],
            skip_list,
        )
        .await,
    );
    checks.push(
        run_check(
            "cargo_clippy",
            true,
            &[
                "cargo",
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            skip_list,
        )
        .await,
    );
    checks.push(
        run_check(
            "cargo_fmt",
            true,
            &["cargo", "fmt", "--all", "--", "--check"],
            skip_list,
        )
        .await,
    );

    // Optional checks.
    checks.push(
        run_check(
            "helm_lint",
            false,
            &["helm", "lint", "infra/helm"],
            skip_list,
        )
        .await,
    );
    checks.push(
        run_check(
            "sqlx_migrate_info",
            false,
            &["sqlx", "migrate", "info"],
            skip_list,
        )
        .await,
    );

    let passed = checks.iter().all(|c| match c.state {
        CheckState::Passed => true,
        CheckState::Skipped => !c.required,
        CheckState::Failed => false,
    });

    let git_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .await
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    let mut environment = BTreeMap::new();
    for key in ["HELIX_ENV", "RUSTUP_TOOLCHAIN", "DATABASE_URL"] {
        if let Ok(v) = std::env::var(key) {
            environment.insert(key.into(), v);
        }
    }

    let mut artifact_hashes = BTreeMap::new();
    for path in ["Cargo.lock", "rust-toolchain.toml"] {
        if let Ok(hash) = sha256_file(path).await {
            artifact_hashes.insert(path.into(), hash);
        }
    }

    Ok(GateReport {
        passed,
        dry_run,
        command: std::env::args().collect(),
        started_at,
        finished_at: Utc::now().to_rfc3339(),
        duration_ms: started.elapsed().as_millis() as u64,
        git_commit,
        environment,
        artifact_hashes,
        checks,
    })
}

async fn run_check(
    name: &str,
    required: bool,
    command: &[&str],
    skip_list: &[String],
) -> CheckResult {
    let full_command: Vec<String> = command.iter().map(|s| s.to_string()).collect();

    if skip_list.iter().any(|s| s == name) {
        return CheckResult {
            name: name.into(),
            required,
            state: CheckState::Skipped,
            command: full_command,
            duration_ms: 0,
            stdout: String::new(),
            stderr: String::new(),
            skip_reason: Some(format!(
                "explicitly skipped via --skip; required={required}"
            )),
        };
    }

    let start = Instant::now();
    let output = Command::new(command[0])
        .args(&command[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;
    let duration_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let state = if out.status.success() {
                CheckState::Passed
            } else {
                CheckState::Failed
            };
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if matches!(state, CheckState::Failed) {
                error!(%name, %required, "check failed");
            } else {
                info!(%name, "check passed");
            }
            CheckResult {
                name: name.into(),
                required,
                state,
                command: full_command,
                duration_ms,
                stdout,
                stderr,
                skip_reason: None,
            }
        }
        Err(e) => {
            error!(%name, error = %e, "check could not run");
            CheckResult {
                name: name.into(),
                required,
                state: CheckState::Failed,
                command: full_command,
                duration_ms,
                stdout: String::new(),
                stderr: format!("failed to spawn command: {e}"),
                skip_reason: None,
            }
        }
    }
}

async fn sha256_file(path: impl AsRef<Path>) -> anyhow::Result<String> {
    let bytes = tokio::fs::read(path).await?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skipped_required_check_blocks_gate() {
        let checks = [
            CheckResult {
                name: "cargo_build".into(),
                required: true,
                state: CheckState::Skipped,
                command: vec![],
                duration_ms: 0,
                stdout: String::new(),
                stderr: String::new(),
                skip_reason: Some("test".into()),
            },
            CheckResult {
                name: "cargo_test".into(),
                required: true,
                state: CheckState::Passed,
                command: vec![],
                duration_ms: 0,
                stdout: String::new(),
                stderr: String::new(),
                skip_reason: None,
            },
        ];

        let passed = checks.iter().all(|c| match c.state {
            CheckState::Passed => true,
            CheckState::Skipped => !c.required,
            CheckState::Failed => false,
        });

        assert!(!passed);
    }

    #[test]
    fn all_required_passed_gate_opens() {
        let checks = [
            CheckResult {
                name: "cargo_build".into(),
                required: true,
                state: CheckState::Passed,
                command: vec![],
                duration_ms: 0,
                stdout: String::new(),
                stderr: String::new(),
                skip_reason: None,
            },
            CheckResult {
                name: "helm_lint".into(),
                required: false,
                state: CheckState::Skipped,
                command: vec![],
                duration_ms: 0,
                stdout: String::new(),
                stderr: String::new(),
                skip_reason: Some("helm not installed".into()),
            },
        ];

        let passed = checks.iter().all(|c| match c.state {
            CheckState::Passed => true,
            CheckState::Skipped => !c.required,
            CheckState::Failed => false,
        });

        assert!(passed);
    }
}
