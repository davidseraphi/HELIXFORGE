# Next action

## Latest: HELIXLEXPRIME-DURABILITY closed — eleventh product through the gate

HELIXLEXPRIME-DURABILITY is complete. The implementation passed local
verification and GitHub Actions run `29670279394` is all green, including
the new **HelixLex Prime durability gate** job.

- Repo: `crates/helix-db/src/lex.rs` (atomic `create_child`
  INSERT...SELECT; guarded `close_matter` / `open_matter` /
  `reopen_matter` / `file_filing` / `withdraw_filing`)
- Tests: `projects/helix-lex-prime/backend/src/main.rs`
  (`filings_rejected_on_deleted_matter`, `concurrent_close_single_winner`)
- Proof: `scripts/helix_lex_prime_durability.ps1` (forced-kill + restore)
- CI: `.github/workflows/ci.yml` `lex-durability` job
- Docs: `docs/goals/HELIXLEXPRIME_DURABILITY.md`, `DECISION_LOG.md`

### What was delivered

- non-deleted-parent guard enforced inside the filing INSERT; a matter
  soft-deleted mid-flight can no longer leak filings
- close is one guarded UPDATE (open + not deleted + NOT EXISTS draft
  filing); open/reopen/file/withdraw carry expected-from status in the
  WHERE
- concurrency proof: 8 racing creates on a deleted matter all rejected;
  8 racing closes → exactly one winner
- crash proof: acknowledged closed matter survives a forced kill of the
  API
- restore proof: schema dump roundtrip with equal counts + content hashes
- `helix-lex-prime` recorded in `durability_gate_proven_products`

### Active goal

None. HELIXLEXPRIME-DURABILITY is closed.

### Next action

Founder selects the next explicit named goal. Open: durability gates for
the remaining 10 products.
