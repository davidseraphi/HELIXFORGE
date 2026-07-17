# Next action

## Latest: HELIXCOLLAB-FULL follow-ups (complete)

**Goal:** add unit/integration tests for the HELIXCOLLAB-FULL edge cases:
offline merge, device revocation, and federation adversarial handling.

See `docs/goals/HELIXCOLLAB_FULL.md` for the full definition of done.

### Closed in this session

1. **Shared test harness** — `projects/helix-collab/backend/src/domain/mod.rs`
   now exposes a `#[cfg(test)] test_support` module that sets safe local-dev
   env vars and builds a fresh `CollabState` under a global lock, so DB pools
   stay tied to a single Tokio runtime and tests do not exhaust Postgres.
2. **Offline merge tests** (`projects/helix-collab/backend/src/domain/documents.rs`):
   - optimistic concurrency rejects a stale `base_version` (Conflict 409)
   - sequential patches create durable revisions
   - client-E2EE document rejects a plaintext patch (Validation 422)
3. **CRDT merge tests** (`projects/helix-collab/backend/src/domain/crdt.rs`):
   - out-of-order / duplicate Yjs updates converge
   - sealed CRDT hub catch-up for late joiners
4. **Device revocation tests** (`projects/helix-collab/backend/src/domain/sovereign.rs`):
   - register → list → revoke lifecycle
   - cross-user isolation (Bob cannot revoke Alice's device)
   - key shares bound to a revoked device remain stored (documents current
     soft-revocation behaviour)
5. **Federation adversarial tests** (`projects/helix-collab/backend/src/domain/sovereign.rs`):
   - happy-path export/import roundtrip
   - invalid client-E2EE payload rejected
   - spoofed `from_tenant` ignored (imported doc uses caller tenant)
   - replay creates duplicate documents (documents current lack of idempotency)
   - export of a missing document returns NotFound
6. **Test wiring for CI** — the 11 data-plane integration tests are marked
   `#[ignore = "requires HelixCore data plane ..."]` so the generic
   `cargo test --workspace --all-features` job in `.github/workflows/ci.yml`
   still passes without Postgres/NATS. The `collab-smoke` job now also runs
   `cargo test -p helix_collab_api -- --ignored` against the real data plane.
7. **Verification**:
   - `cargo test -p helix_collab_api` — **9 unit tests pass** (11 ignored)
   - `cargo test -p helix_collab_api -- --ignored` — **11 integration tests pass**
   - `cargo clippy --workspace --all-targets -- -D warnings` — **clean**
   - `scripts/helix_collab_smoke.ps1` — **PASS**

### Next options

Pick one before continuing:

1. **Set the next explicit product goal** — e.g. bring another product to full
   depth, or start a named HelixCollab polish packet (browser e2e, load tests).
2. **Harden the federation surface** — add signature verification, remote
   allowlist, and replay/idempotency protection; the adversarial tests above
   already define the expected behaviours.

Do **not** activate HelixAnvil or resume scattered product 1–20 depth work
without a named goal.

## Active goal

`HELIXCOLLAB-FULL` is complete. Awaiting next explicit goal selection.

## Open founder decisions

- Managed-service commercial model and final custody providers (per HelixCore
  spec) — does not block G0, but must be resolved before G1 capability broker.
- HelixAnvil canonical home is `projects/helix-anvil`; sequencing remains
  portfolio-last.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCORE-FULL and
HELIXCOLLAB-FULL are both CI-proven.
HELIXCOLLAB-FULL follow-up tests are complete: 20/20 Rust tests pass,
including offline merge, device revocation, and federation adversarial suites.
`cargo clippy -p helix_collab_api -- -D warnings` is clean and the deep
PowerShell smoke still passes.
Next step is either setting the next explicit product/program goal or
hardening federation (signature verification, remote allowlist, replay
idempotency) using the new adversarial tests as the spec.
Do not activate HelixAnvil or resume scattered product 1–20 depth work without
a named goal.
```
