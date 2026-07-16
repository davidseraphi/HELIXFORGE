# HelixForge five-year delivery roadmap

## Honest planning basis

The product sheets describe category-defining destinations. Building all of
them to their final G4 state at the same time would create shallow products and
unsafe foundations. The portfolio therefore uses shared gates and a strict work
limit.

The 60-month portfolio target is:

- every catalog product has a real domain model, tested local journey, portable
  data, and honest limits;
- every product reaches at least G2 or is explicitly paused with evidence;
- the shared foundation, HelixCollab, HelixCode, HelixFlow, HelixInsights, and
  two founder-selected domain products reach G3 or G4;
- high-stakes and physical-control products advance only when external domain,
  safety, privacy, and legal review is available.

This is close to the practical limit for one founder using AI-assisted teams.
Calling all 21 products category-defining after five years without real users,
independent review, and field proof would be a false success.

## Founder pace

> Founder pace: 5-10 sessions per active day; ~4 active days per active week.
> Convert session counts via ÷10 per active day. See
> `session_pace_calibration.md` (CP_OS git log: 41 Day-N sessions over ~4
> working days).

Roadmap estimates use **active weeks**, not generic calendar weeks. Calendar
months include research, user learning, external review, waiting, procurement,
field pilots, and recovery time that cannot be compressed by more agent sessions.

## Work-in-progress limits

- One shared-foundation gate may be active.
- At most three product gates may be active.
- At most one high-stakes or physical-control product may be in live pilot.
- One agent owns one file or component at a time. Overlap is detected before work.
- A new gate does not start while a P0 or P1 finding remains in a dependency.
- A product waiting for external evidence moves to `waiting`; another gate may
  use the slot without pretending the first gate is complete.

## Year 1 — make the platform truthful

### Portfolio outcomes

1. Put the monorepo under version control with protected review and recovery.
2. Make the full Rust and TypeScript workspace build, format, test, and package.
3. Establish native Windows, macOS, and Linux CI.
4. Close identity, tenant separation, per-resource access, secret brokerage,
   all-or-nothing writes, truthful readiness, durable jobs, audit, backup/restore,
   and 30-day recovery.
5. Build the shared shell and semantic UX states once.
6. Reach G1 for HelixCollab and HelixCode.
7. Reach G1 for HelixFlow and HelixInsights.

### Exit evidence

- Fresh complete CI on all three operating systems.
- Forced-crash, concurrent-write, tenant-isolation, cancellation, backup/restore,
  delete/restore, export/import, and secret-non-disclosure proof.
- Four full user journeys with browser or native evidence.
- No status document claims a capability that its fresh gate cannot prove.

## Year 2 — prove the shared product engine

### Portfolio outcomes

1. HelixCollab and HelixCode reach G2 with team use, full end-to-end tests,
   operations, accessibility, and independent security review.
2. HelixFlow and HelixInsights reach G2 and become shared domain services rather
   than simple CRUD products.
3. HelixCommerce, HelixEdu, HelixCapital, HelixWell, and HelixNetwork reach G1.
4. HelixForge Studio reaches G1 by generating portable, readable applications
   that use the same contracts.
5. Every product gains stable identity, recovery, portability, proof, and real UI.

### Exit evidence

- At least two real teams use the four lead products for non-critical work.
- A generated Studio application can be understood and maintained without Studio.
- Business-product records survive concurrency, migration, and tenant attacks.
- User studies show that new users can find work, progress, decisions, proof,
  and recovery without instruction.

## Year 3 — deepen domains, not names

### Portfolio outcomes

1. The business and human suite reaches G2: Commerce, Edu, Capital, Well, Network,
   and Studio.
2. NovaLabs, TerraPrime, ClimatePrime, GridPrime, and QuantumForge reach G1 with
   safe simulated or synthetic journeys.
3. LexPrime, CuraPrime, VitaPrime, and SynthBio reach G0 foundation contracts
   with external domain advisers and no real sensitive data.
4. OrbitPrime reaches G1 in simulation only.
5. Pulse begins G0 only after every current product has left generic CRUD state.

### Exit evidence

- Each active domain has one real engine and one complete user experience.
- Scientific products export reproducible research objects.
- Physical products prove simulation, hardware-in-the-loop where relevant, and
  strict observe/recommend/approve/control separation.
- High-stakes products have named governance and external review paths.

## Year 4 — category leadership in selected products

### Portfolio outcomes

1. Founder selects two domain products using user need, evidence, safety, and
   strategic value. They advance to G3.
2. HelixCollab, HelixCode, Flow, and Insights advance toward G3 or G4.
3. Remaining frontier products reach G1 or G2 with standards-based exchange and
   independent review.
4. Pulse reaches G1 single-node maturity and may serve non-critical product paths.
5. HelixAnvil implementation may begin only if the monorepo sequencing decision
   is changed or the monorepo endgame, including Pulse, is complete, **and** the
   founder resolves its missing intended root versus nested scaffold. Its design
   and fixtures may stay current without product code. No agent may move either
   location while the decision is open.

### Exit evidence

- Selected leaders show clear outcome improvement over established workflows.
- Portable exit is tested by moving real projects to a clean environment.
- Agents demonstrate narrow authority, true cancellation, and independent proof.
- No frontier pilot depends on a model claim without domain validation.

## Year 5 — sovereign network effects

### Portfolio outcomes

1. Two selected domain products and the four lead products achieve G3/G4 proof.
2. Other products achieve G2 or receive an honest pause/retire decision.
3. Pulse reaches G2 replicated-cluster maturity after formal model, chaos,
   recovery, and independent distributed-systems review.
4. Carefully selected federation pilots share narrow approved capabilities or
   derived results without central raw-data custody.
5. HelixAnvil follows its own 60-month roadmap after activation; it is never
   rushed into a weak Electron replacement merely to fit this portfolio date.

### Exit evidence

- Independent users can verify important work outside the originating machine.
- A user can run locally, self-host, migrate providers, revoke a project, and
  restore work without founder intervention.
- Products publish honest service, safety, privacy, accessibility, performance,
  and portability results.
- The portfolio contains fewer false claims, not simply more features.

## Product activation order

| Wave | Products | Start condition |
|---|---|---|
| Foundation | HelixCore | Immediate; full workspace truth first |
| Lead | Collab, Code, Flow, Insights | Foundation G0 proven |
| Human and business | Commerce, Edu, Capital, Well, Network, Studio | Lead shared contracts stable |
| Safe science | NovaLabs, Terra, Climate, Quantum, Orbit | Synthetic/simulation safety harness ready |
| High stakes | Lex, Cura, Vita, SynthBio, Grid live control | External governance and domain review ready |
| Data plane | Pulse | Products 1–20 have real domains and no critical Pulse dependency |
| Standalone | Anvil | Founder resolves canonical location, then sequencing gate is satisfied or explicitly changed |

## Portfolio decision gates

At the end of each six-month period, each active product receives one decision:

- **Advance:** evidence supports the next gate.
- **Hold:** useful work exists but a named dependency or external review is missing.
- **Narrow:** keep the strongest user journey and remove weak scope.
- **Merge:** the capability belongs in another product.
- **Retire:** preserve export and history; stop investment.

No product advances because its planned date arrived.
