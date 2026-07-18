# HELIXCAPITAL-DURABILITY

Prove the Foundation Integrity durability gate on HelixCapital: fresh crash,
concurrency, and restore, verified locally and in CI. Second product through
the gate (after `helix-collab`).

## Scope

HelixCapital's journal paths are already transactional (`post_journal` and
`void_journal` both commit journal, lines, and balance updates in one
transaction under `FOR UPDATE` locks), so this packet is gate proof, not
repair: prove the money writes cannot tear under race, crash, or restore.

## Definition of done

1. Ignored Postgres integration tests (run in the existing `capital-smoke`
   CI job):
   - `concurrent_voids_single_winner` — N concurrent voids of one journal
     produce exactly one success; the rest fail; the balance reversal is
     applied exactly once.
   - `concurrent_journals_exact_balances` — N concurrent balanced journals
     on the same accounts all commit under `FOR UPDATE` serialization, and
     the final balances equal the exact sum of all posted lines.
2. `scripts/helix_capital_durability.ps1`:
   - post a journal, void it, verify trial-balance consistency
   - post a journal, force-kill the API, restart, and verify the journal,
     its lines, and the account balances are all present and consistent
   - `pg_dump` of the `capital` schema restores into a scratch database
     with equal account/journal/line counts and equal content hashes
3. `capital-durability` CI job in `.github/workflows/ci.yml`; existing
   `capital-smoke` job untouched.
4. `cargo test --workspace --all-features` and
   `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Status

- **Closed / CI-proven**
- CI run: `29662883748` (**HelixCapital durability gate** job green)
- Proof script: `scripts/helix_capital_durability.ps1`
- Gate proven locally (Windows) and in CI (ubuntu)

## Out of scope

- Audit/metering/NATS transactionality on capital writes (post-commit
  steps stay post-commit; the gate proves domain rows are never torn).
- Idempotency keys on journal posts.
- Durability gates for other products (each needs its own named packet).
