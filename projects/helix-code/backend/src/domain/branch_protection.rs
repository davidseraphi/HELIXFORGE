//! Shared branch protection enforcement for REST commits and smart HTTP receive-pack.
//!
//! Rules:
//! - `require_pr` blocks direct pushes/commits unless `HELIX_CODE_ALLOW_DIRECT_PUSH=1|true`
//! - `deny_force_push` blocks non-fast-forward updates unless `HELIX_CODE_ALLOW_FORCE_PUSH=1|true`
//! - `required_status_checks` validated on PR merge (see collab_api)

use super::git_store::GitStore;
use helix_db::{CodeBranchProtection, CodeRepoStore};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use std::path::Path;
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RefUpdate {
    pub old_oid: String,
    pub new_oid: String,
    pub refname: String,
}

impl RefUpdate {
    pub fn branch_name(&self) -> Option<&str> {
        self.refname.strip_prefix("refs/heads/").or_else(|| {
            if !self.refname.starts_with("refs/") {
                Some(self.refname.as_str())
            } else {
                None
            }
        })
    }
}

pub fn is_zero_oid(oid: &str) -> bool {
    oid.chars().all(|c| c == '0') || oid.is_empty()
}

pub fn allow_direct_push_breakglass() -> bool {
    let on = std::env::var("HELIX_CODE_ALLOW_DIRECT_PUSH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if on {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            super::breakglass::record(
                "ALLOW_DIRECT_PUSH",
                "HELIX_CODE_ALLOW_DIRECT_PUSH active (global)",
            );
        });
    }
    on
}

pub fn allow_force_push_breakglass() -> bool {
    let on = std::env::var("HELIX_CODE_ALLOW_FORCE_PUSH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if on {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            super::breakglass::record(
                "ALLOW_FORCE_PUSH",
                "HELIX_CODE_ALLOW_FORCE_PUSH active (global)",
            );
        });
    }
    on
}

/// Parse git receive-pack command pkt-lines (before the pack stream).
pub fn parse_receive_pack_commands(body: &[u8]) -> Vec<RefUpdate> {
    let mut i = 0usize;
    let mut out = Vec::new();
    while i + 4 <= body.len() {
        let Ok(len_hex) = std::str::from_utf8(&body[i..i + 4]) else {
            break;
        };
        let Ok(len) = usize::from_str_radix(len_hex, 16) else {
            break;
        };
        i += 4;
        if len == 0 {
            // flush-pkt ends the command list
            break;
        }
        if len < 4 {
            break;
        }
        let data_len = len - 4;
        if i + data_len > body.len() {
            break;
        }
        let line = &body[i..i + data_len];
        i += data_len;
        // Strip trailing LF; capabilities after NUL
        let s = String::from_utf8_lossy(line);
        let cmd = s.split('\0').next().unwrap_or("").trim_end_matches('\n');
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() >= 3 {
            out.push(RefUpdate {
                old_oid: parts[0].to_string(),
                new_oid: parts[1].to_string(),
                refname: parts[2].to_string(),
            });
        }
    }
    out
}

/// True if new_oid is not a fast-forward of old_oid (force push).
pub fn is_force_push(repo: &Path, old_oid: &str, new_oid: &str) -> bool {
    if is_zero_oid(old_oid) || is_zero_oid(new_oid) {
        return false; // create or delete
    }
    if old_oid == new_oid {
        return false;
    }
    // merge-base --is-ancestor A B => 0 if A is ancestor of B (FF)
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["merge-base", "--is-ancestor", old_oid, new_oid])
        .status();
    match status {
        Ok(s) if s.success() => false,
        Ok(_) => true,
        Err(_) => {
            // If we cannot check, be conservative for protected branches
            true
        }
    }
}

/// Optional per-request break-glass (env OR tenant). Defaults to env-only helpers.
#[derive(Debug, Clone, Copy, Default)]
pub struct PushBreakglass {
    pub allow_direct_push: bool,
    pub allow_force_push: bool,
}

impl PushBreakglass {
    pub fn from_env() -> Self {
        Self {
            allow_direct_push: allow_direct_push_breakglass(),
            allow_force_push: allow_force_push_breakglass(),
        }
    }
}

/// Enforce protection on a single branch update (push or REST-equivalent).
#[allow(dead_code)]
pub fn enforce_push_rules(
    prot: &CodeBranchProtection,
    branch: &str,
    old_oid: Option<&str>,
    new_oid: Option<&str>,
    repo_path: Option<&Path>,
) -> HelixResult<()> {
    enforce_push_rules_with(
        prot,
        branch,
        old_oid,
        new_oid,
        repo_path,
        PushBreakglass::from_env(),
    )
}

pub fn enforce_push_rules_with(
    prot: &CodeBranchProtection,
    branch: &str,
    old_oid: Option<&str>,
    new_oid: Option<&str>,
    repo_path: Option<&Path>,
    bg: PushBreakglass,
) -> HelixResult<()> {
    let deleting = new_oid.map(is_zero_oid).unwrap_or(false);
    if prot.require_pr && !deleting {
        if bg.allow_direct_push {
            super::breakglass::record(
                "DIRECT_PUSH_USED",
                &format!("require_pr bypassed for branch '{branch}'"),
            );
        } else {
            return Err(HelixError::forbidden(format!(
                "branch '{branch}' is protected (require_pr); open a pull request or set HELIX_CODE_ALLOW_DIRECT_PUSH=1 / tenant breakglass"
            )));
        }
    }

    if prot.deny_force_push {
        if let (Some(old), Some(new), Some(path)) = (old_oid, new_oid, repo_path) {
            if is_force_push(path, old, new) {
                if bg.allow_force_push {
                    super::breakglass::record(
                        "FORCE_PUSH_USED",
                        &format!("deny_force_push bypassed for branch '{branch}'"),
                    );
                } else {
                    return Err(HelixError::forbidden(format!(
                        "branch '{branch}' is protected (deny_force_push); non-fast-forward update rejected"
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Enforce all receive-pack ref updates against stored protections.
pub async fn enforce_receive_pack(
    store: &CodeRepoStore,
    tenant_id: TenantId,
    repo_id: Uuid,
    repo_path: &Path,
    body: &[u8],
) -> HelixResult<Vec<RefUpdate>> {
    let updates = parse_receive_pack_commands(body);
    for u in &updates {
        let Some(branch) = u.branch_name() else {
            // tags / notes: skip branch protection (tags can be separate later)
            continue;
        };
        let bg = super::tenant_policy::load_effective(store, tenant_id)
            .await
            .map(|e| PushBreakglass {
                allow_direct_push: e.allow_direct_push,
                allow_force_push: e.allow_force_push,
            })
            .unwrap_or_else(|_| PushBreakglass::from_env());
        if let Some(prot) = store
            .matching_protection(tenant_id, repo_id, branch)
            .await?
        {
            enforce_push_rules_with(
                &prot,
                branch,
                Some(&u.old_oid),
                Some(&u.new_oid),
                Some(repo_path),
                bg,
            )?;
        }
    }
    Ok(updates)
}

/// REST commit path: require_pr (and note force N/A for additive commits).
pub async fn enforce_rest_commit(
    store: &CodeRepoStore,
    tenant_id: TenantId,
    repo_id: Uuid,
    branch: &str,
) -> HelixResult<()> {
    let bg = super::tenant_policy::load_effective(store, tenant_id)
        .await
        .map(|e| PushBreakglass {
            allow_direct_push: e.allow_direct_push,
            allow_force_push: e.allow_force_push,
        })
        .unwrap_or_else(|_| PushBreakglass::from_env());
    if let Some(prot) = store
        .matching_protection(tenant_id, repo_id, branch)
        .await?
    {
        enforce_push_rules_with(&prot, branch, None, Some("1"), None, bg)?;
    }
    Ok(())
}

/// Required status checks for PR merge: each name must have a successful pipeline run
/// for the PR head commit (or latest run on the source branch tip).
pub async fn enforce_required_status_checks(
    store: &CodeRepoStore,
    tenant_id: TenantId,
    repo_id: Uuid,
    branch: &str,
    head_sha: &str,
) -> HelixResult<()> {
    let Some(prot) = store
        .matching_protection(tenant_id, repo_id, branch)
        .await?
    else {
        return Ok(());
    };
    let checks: Vec<String> = match &prot.required_status_checks {
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .filter(|s| !s.is_empty())
            .collect(),
        serde_json::Value::String(s) if !s.is_empty() => vec![s.clone()],
        _ => vec![],
    };
    if checks.is_empty() {
        return Ok(());
    }
    let mut missing = Vec::new();
    for name in &checks {
        let status = store
            .latest_check_status(tenant_id, repo_id, name, head_sha)
            .await?;
        let ok = status
            .as_deref()
            .map(|s| {
                matches!(
                    s.to_ascii_lowercase().as_str(),
                    "succeeded" | "success" | "passed" | "ok" | "completed"
                )
            })
            .unwrap_or(false);
        if !ok {
            missing.push(format!(
                "{name}={}",
                status.unwrap_or_else(|| "missing".into())
            ));
        }
    }
    if !missing.is_empty() {
        return Err(HelixError::validation(format!(
            "required status checks not satisfied for {branch}@{head_sha}: {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

/// Resolve source branch tip SHA for a PR (git + fallback).
pub fn branch_tip_sha(
    git: &GitStore,
    tenant_id: TenantId,
    repo_name: &str,
    branch: &str,
) -> HelixResult<String> {
    git.rev_parse(tenant_id, repo_name, branch)
        .or_else(|_| git.rev_parse(tenant_id, repo_name, &format!("refs/heads/{branch}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_receive_pack_single_update() {
        // pkt-line: length hex includes the 4 length bytes
        let cmd = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb refs/heads/main\n";
        let len = cmd.len() + 4;
        let mut body = format!("{len:04x}").into_bytes();
        body.extend_from_slice(cmd.as_bytes());
        body.extend_from_slice(b"0000");
        let u = parse_receive_pack_commands(&body);
        assert_eq!(u.len(), 1);
        assert_eq!(u[0].refname, "refs/heads/main");
        assert_eq!(u[0].branch_name(), Some("main"));
        assert!(!is_zero_oid(&u[0].new_oid));
    }

    #[test]
    fn zero_oid_detect() {
        assert!(is_zero_oid("0000000000000000000000000000000000000000"));
        assert!(!is_zero_oid("abc"));
    }
}
