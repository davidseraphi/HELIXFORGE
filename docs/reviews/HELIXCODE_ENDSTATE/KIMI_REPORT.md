# Kimi HelixCode re-review (A+C polish)

**Generated:** 2026-07-15  
**Command:** kimi --print --final-message-only --yolo --afk (UTF-8 process, ASCII-only prompt)  
**Session:** `kimi -r cc9ad7d1-d840-4d6c-82e2-4adff5f621e1`  
**kimi_exit:** 0  

**Scope reviewed:** `container.rs` (Docker bind normalize), `branch_protection.rs`, `webhook_policy.rs`, `terminal_policy.rs`  

**Local prove (same day):** endstate + base smoke **PASS** with `isolation=docker`; status-check green path; unit tests **23 PASS**.

---

## Verdict: **PASS_WITH_FOLLOWUPS**

### Residual risks (Kimi) — **closed 2026-07-15**

| # | Residual | Closure |
|---|----------|---------|
| 1 | Shell injection via isolation | `cmd_policy` gates every `run_isolated` before `sh -c`/`cmd /C`; metachar + allowlist |
| 2 | Privileged host fallback | Requires `HELIX_CODE_ALLOW_HOST_FALLBACK=1` or `CI_ALLOW_ALL`; breakglass logged |
| 3 | Broad terminal file reads | Relative-safe paths only (`cat`/`type`/`git show`); deny absolute/`..` |
| 4 | Webhook allowlist open + dev-headers | Fail-closed outside local; private only via `WEBHOOK_ALLOW_PRIVATE` or `HELIX_ENV=local` (not dev headers) |
| 5 | Break-glass un-audited | `breakglass` module: tracing warn + process ring + domain status; per-use DIRECT_PUSH/FORCE_PUSH records |

### Closed vs prior Kimi P0/P1 (engineering)

| Prior finding | Status |
|---------------|--------|
| Smart HTTP bypasses branch protection | Closed (`branch_protection` + receive-pack) |
| deny_force_push not enforced | Closed |
| required_status_checks not validated | Closed (+ green path smoke) |
| Weak terminal denylist | Closed (allowlist-first) |
| Webhook SSRF | Closed (policy + HTTPS non-local + re-resolve) |
| max_agent_jobs_day / max_sealed_bytes | Closed |
| Windows Docker bind path | Closed (`/c/...` normalize; docker smoke PASS) |

### Note on Windows Kimi CLI

Earlier full-prompt runs failed with `charmap` codec errors on emoji/unicode arrows. Re-run uses ASCII-only prompts + UTF-8 process encoding (`PYTHONUTF8=1`). Script `kimi_helixcode_endstate_review.ps1` updated accordingly.
