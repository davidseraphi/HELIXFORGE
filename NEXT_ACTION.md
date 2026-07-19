# Next action

## Latest: HELIXNETWORK-DURABILITY closed — eighth product through the gate

HELIXNETWORK-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29668195166` is all green, including
the new **HelixNetwork durability gate** job.

- Repo: `crates/helix-db/src/network.rs` (`request_connection` as one
  transaction with profiles locked `FOR UPDATE`)
- Tests: `projects/helix-network/backend/src/main.rs`
  (`concurrent_accepts_single_winner`, `concurrent_requests_same_pair`)
- Proof: `scripts/helix_network_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `network-durability` job
- Docs: `docs/goals/HELIXNETWORK_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- profile-active, blocked-pair, and existing-row checks plus the
  insert/revive write now run in one transaction — a profile deactivated
  or a pair blocked mid-flight can no longer silently accept a request
- concurrency proof: 8 racing accepts → exactly one winner; 8 racing
  requests for one pair → one success, 7 conflicts, one row
- crash proof: acknowledged accepted connection survives a forced kill of
  the API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-network` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXNETWORK-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 13 products.
