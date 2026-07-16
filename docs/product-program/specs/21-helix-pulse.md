# HelixPulse — sovereign distributed data and memory plane

```yaml
product: HelixPulse
catalog_order: 21
status: target-state-spec
horizon: 60 months
current_maturity: scaffold
primary_users: [application developers, platform teams, agent builders, operators]
deployment: [local, self-hosted, managed]
platforms: [windows, macos, linux, web]
```

## 1. Category claim

HelixPulse is a memory plane that starts as one safe local process and grows to
a distributed cluster without changing the meaning, ownership, recovery, or
proof of the data.

## 2. Five-year destination

The useful product is a fast typed key-value, document, counter, lease, stream,
queue, and event service with a simple local setup and excellent inspection
tools. The category advantage is explicit truth: every collection declares its
consistency, durability, placement, retention, privacy, and recovery contract,
and the system proves whether it kept that contract. The frontier is a
federation of user-owned clusters that can share narrow derived views while raw
data and control remain local. Operators own placement, deletion, failover,
schema, and cross-site authority.

## 3. Users and hard jobs

- A developer needs one process that is easy to understand and fears silent
  data loss when moving to production.
- A platform team needs predictable clusters and fears split-brain or unsafe
  failover.
- An agent builder needs scoped working memory and fears secret or tenant leakage.
- An operator needs visible health and fears a green dashboard that hides lag.
- A data owner needs portable state and fears protocol or cloud lock-in.

## 4. Product laws

1. Every collection declares consistency, durability, retention, and placement.
2. Acknowledged durable writes survive the declared failure model.
3. Ephemeral, recoverable, and permanent data are different types, not flags hidden in settings.
4. Unknown replica state is never called healthy or caught up.
5. Failover cannot weaken consistency or durability without a new human-approved policy.
6. Tenant, collection, and capability limits are enforced in storage and protocol paths.
7. Compatibility never changes native semantics silently.
8. One-node and cluster modes share the same data model and test suite.
9. Operators can export all durable data, schemas, policies, history, and proof.

## 5. Scope boundaries

Pulse owns low-latency shared state, durable streams, leases, queues, caches,
indexes, replication, placement, snapshots, and cluster operations. PostgreSQL
remains the system of record for relational business truth unless a product
explicitly chooses Pulse durability. NATS remains an adapter during migration.
MinIO owns large immutable objects. Pulse is not a secret vault, general SQL
database, data lake, or excuse to bypass product audit.

## 6. Signature experiences

| Journey | Entry | Visible progress | Human decision | Completion proof | Failure and recovery | Portability |
|---|---|---|---|---|---|---|
| Start local | Install one signed binary and choose a local data root. | Setup shows storage checks, policy creation, first write, WAL position, and restart check. | The developer accepts the collection contract before the first write. | Signed report links collection policy, write receipt, restart, and recovered value. | Invalid roots or failed durability checks block writes; setup resumes from the last safe step. | Export one logical snapshot with schema, policy, and verifier. |
| Inspect truth | Open a tenant or collection from CLI, desktop inspector, or web console. | Freshness, memory, disk, expiry, lag, backpressure, and failed checks update with timestamps. | The operator chooses whether a degraded collection can stay read-only or must stop. | View exports the exact checks and source positions behind every state. | Stale checks become `unknown`; a retry never reuses an old green result. | The same inspection bundle opens offline on another OS. |
| Move to a cluster | Select a proven local collection and a three-node target. | Placement preview shows replicas, bytes, bandwidth, checkpoints, risk, and remaining work. | The operator approves the plan and the final cutover separately. | Proof includes simulation, plan hash, checkpoints, cutover position, and post-checks. | Pause, cancel, or node loss leaves the old owner authoritative until a checked cutover; resume is idempotent. | The placement plan and snapshot are provider-neutral. |
| Use agent memory | A project owner selects collection, fields, operations, quota, purpose, and lease time. | The grant card shows requests, uses, denials, expiry, and current quota. | The owner approves, narrows, revokes, or refuses the lease. | Signed metadata proves the exact grant and every allowed or denied use without exposing values. | Revocation stops future access; rejected writes return a safe reason and do not widen scope. | Logical entries and source links export without credentials. |
| Recover an error | A user deletes a recoverable record, stream, or collection. | Impact preview shows dependants, size, retention, purge date, and restore eligibility. | The owner confirms delete; a restore authority confirms recovery. | Tombstone, recovery item, restore event, and identity check are signed. | The item stays in the 30-day bin; legal hold blocks purge; failed restore leaves the original tombstone intact. | Recovery metadata travels with a logical backup. |
| Survive failure | Start an approved drill against disposable cluster state. | Leader, quorum, disk, lag, rejected writes, retries, and unknown outcomes stream live. | The operator chooses failover, read-only mode, or stop from allowed actions. | A signed drill maps every request to accepted, rejected, retried, or unknown. | Unsafe quorum loss fails closed; recovery replays from checked logs and snapshots. | The drill package replays on another supported host. |
| Leave | Select a consistent collection or tenant export point. | Snapshot, schemas, policies, indexes, history, checksums, and validation show progress. | The data owner approves scope and redaction. | A clean target imports the bundle and matches the logical-state digest. | Export failure preserves the source; import uses a disposable namespace and rolls back on mismatch. | The bundle is documented, provider-neutral, and verifies on Windows, macOS, and Linux. |

## 7. Capability map

| ID / gate | Input | Output | Invariant | Authority | Evidence | Failure state | Testable acceptance |
|---|---|---|---|---|---|---|---|
| HP-F0-01 / G0 | Tenant, namespace, schema, retention, durability, consistency, and placement contract | Versioned typed collection | Tenant and collection identity are part of every key; no unbounded metadata escape | Tenant owner drafts; schema authority activates | Contract hash, actor, schema version, and activation event | Invalid contract is rejected or quarantined; nothing is partly active | Create, reopen, export, and cross-tenant denial fixtures pass on all three OSs |
| HP-F0-02 / G0 | Idempotency key and one or more typed mutations | Checksummed WAL record and atomic commit receipt | A batch is all visible or none visible; an acknowledged durable write survives the declared fault model | Client needs exact write capability; no admin bypass | Batch ID, log position, checksums, fsync result, and recovery trace | Outcome is rejected, committed, or `unknown` with the same idempotency handle; corruption quarantines the segment | Zero lost acknowledged writes and zero torn batches across at least 10,000 injected crash points per release |
| HP-F0-03 / G0 | Live storage, process, disk, memory, WAL, and backpressure checks | Timestamped `healthy`, `degraded`, `blocked`, or `unknown` state | Stale or missing evidence can never be healthy | Operator may acknowledge; only fresh checks set state | Check inputs, timestamps, thresholds, and signed status result | Failed or stale check becomes degraded/unknown and names the safe action | Injected disk, memory, WAL, and stale-check faults appear within 5 seconds in every release test |
| HP-F1-01 / G1 | Typed key/document/counter operation, lease, expected version, and transaction boundary | Versioned value, compare result, watch event, or explicit conflict | Version order and expiry semantics are deterministic; CAS never guesses | Collection grant controls each operation and field | Request ID, versions, transaction receipt, and watch position | Conflict, quota, expiry, or cancellation returns a typed result with no partial transaction | One million mixed operations match the reference state model on each desktop OS |
| HP-F1-02 / G1 | Stream entries, queue jobs, consumer group, visibility timeout, retry policy, and idempotency key | Ordered stream position or owned delivery state | A job is not both completed and visible; redelivery never erases earlier attempts | Producer and consumer capabilities are separate | Entry ID, group offset, delivery attempts, ack/dead-letter event | Timeout returns work to the queue; poison work enters dead-letter; duplicate input returns original result | One million duplicate, timeout, crash, and retry cases produce exactly one accepted logical effect |
| HP-F1-03 / G1 | Authorized inspection, backup, restore, schema change, delete, or export request | Human-readable view or verified portable artifact | Inspection is read-only; restore never overwrites live data before validation; recoverable delete keeps identity | Split read, backup, restore, schema, delete, and export roles | Manifest, checksums, preview, approvals, progress, validation, and completion event | Failed work is resumable or rolled back; deleted durable data enters the 30-day bin | Ten cross-OS backup/restore/export cycles per release match logical-state digests |
| HP-F2-01 / G2 | Shard state, membership, client consistency request, and replicated log command | Quorum result and deterministic shard state | One committed log order; membership changes are joint and durable; strong reads do not serve known stale state | Cluster authority proposes membership; quorum protocol commits it | Term, index, votes, membership record, replica positions, and model-check trace | Quorum loss fails closed for strong writes; uncertain leaders step down | Formal model has no safety counterexample and 1,000 chaos campaigns preserve committed state |
| HP-F2-02 / G2 | Source/target shards, placement plan, bandwidth and risk limits | Checkpointed shard transfer and atomic ownership cutover | One authoritative owner set at every point; data movement cannot imply ownership | Placement role approves plan; separate cutover approval is required | Plan hash, copied ranges, digest, throttle, pause/resume, and cutover record | Pause/cancel keeps old ownership; mismatch rolls back before cutover | Kill/restart at every phase and 100 concurrent reshard cases produce no missing or duplicate logical key |
| HP-F2-03 / G2 | Subject, tenant, collection, fields, operations, quota, residency, purpose, and lease time | Narrow capability grant, denial, use, expiry, or revocation | Deny by default; secret values never enter policy or audit; residency is enforced at placement and protocol paths | Data owner grants; security role sets non-bypassable ceilings | Signed metadata-only grant, denial, use aggregate, rotation, and revocation events | Denied/expired grants fail closed; revocation blocks future operations within 2 seconds | 100,000 adversarial tenant, field, quota, residency, and revocation cases show zero cross-boundary access |
| HP-F3-01 / G3 | Collection durability/latency class, hotness, capacity, and tier policy | Versioned tier placement with recall state | Canonical identity and declared durability survive movement; cold tier is never called immediately available | Operator approves policy; controller follows bounded policy | Object hashes, tier moves, recall time, durability class, and checks | Tier outage degrades availability by contract and never deletes the last durable copy | Repeated hot/cold movement over 100 TB synthetic state preserves all digests and class rules |
| HP-F3-02 / G3 | Canonical records, index schema, build position, and resource limit | Full-text, numeric, geospatial, or vector index with freshness | Index is derived and rebuildable; canonical writes do not depend on index truth | Schema authority creates; readers choose whether allowed staleness is acceptable | Build input position, index version, checksum, lag, query plan, and rebuild proof | Failed index is `stale` or `offline`; canonical data stays available | Delete every index, rebuild from canonical data, and match 10,000 golden queries within declared tolerance |
| HP-F3-03 / G3 | Regions, residency, failure domains, consistency class, and latency bound | Multi-region placement and routing policy | Semantics cannot weaken during failover; region and residency rules remain binding | Data owner selects class; operator approves topology and failover | Region positions, route decision, lag, failover approval, and client receipt | Partition follows declared strong, bounded-stale, or async behavior and labels unknown outcomes | 500 partition and region-loss campaigns match the formal class-specific oracle |
| HP-F4-01 / G4 | Owner-approved stream, aggregate, or index view plus recipient, purpose, and expiry | Revocable federated view without raw global custody | Each owner keeps source authority; no global normal administrator; revocation stops future flow | Every contributing owner approves its binding | View definition, minimization proof, grants, uses, lag, and revocations | Site outage isolates that site; revoked/expired views fail closed | Five independent clusters join, leave, revoke, and recover without exposing unapproved raw data |
| HP-F4-02 / G4 | Measured load, cost, latency, failure risk, policy, and candidate topologies | Ranked placement proposals with predicted trade-offs | Advisor never moves data or changes policy; measured facts and estimates stay separate | Operator may simulate; placement and cutover keep their normal approvals | Input window, model version, proposals, rejected options, simulation, and decision | Low confidence or missing telemetry produces `no recommendation` | Across 50 replayed incidents, every proposal stays inside policy and zero changes occur without approval |
| HP-F4-03 / G4 | Source-linked memory entry, purpose, confidence, contradiction, expiry, and retrieval lease | Verifiable memory result with source and limits | Memory is not fact without source; contradiction and expiry are preserved; retrieval is purpose-bound | Project owner sets policy; agent may write/read only within its lease | Source hash, entry version, contradiction links, grant, query, result IDs, and expiry | Unsupported, expired, or forbidden memory is excluded with a reason; revocation blocks future retrieval | 100,000 isolation, expiry, contradiction, and provenance cases have zero secret or cross-project leakage |

## 8. Domain model

Durable state never depends on a folder path. Ephemeral data declares that it cannot
be recovered. Tombstones preserve identity until the collection policy allows purge.

| Group and records | Owner | Lifecycle | Version rule | Retention and delete | Relationships |
|---|---|---|---|---|---|
| Control: `Cluster`, `Node`, `Tenant`, `Namespace`, `Collection`, `SchemaVersion`, `CollectionPolicy` | Cluster operator owns infrastructure; tenant owner owns collection meaning | Draft, validated, active, read-only, deprecated, closed | Stable IDs; schemas and policies are immutable versions with explicit activation | Policy and audit outlive data as required; collection delete uses preview, tombstone, 30-day bin, then authorized purge | Tenant owns namespaces; namespace owns collections; collection binds schema and policy |
| Replication: `Shard`, `Replica`, `Term`, `LogEntry`, `Snapshot`, membership record | Cluster protocol owns ordering; operator owns topology policy | Created, catching up, voting/non-voting, healthy/degraded, retiring, removed | Term and log index order events; snapshot names included index and hash | Logs compact only after safe snapshot/replica rules; removed replicas are wiped after verified handoff and retention | Collection maps to shards; shard has replicas, log, and snapshots |
| Data: `Key`, `ValueVersion`, `Tombstone`, `Lease`, `AtomicBatch` | Tenant data owner; writer acts through a grant | Proposed, committed, superseded/expired, tombstoned, purged | Monotonic logical version per key and immutable batch ID | Durable delete creates tombstone and recovery item; TTL ephemeral expiry may be final when declared | Keys belong to a collection; versions point to batch/log positions; leases guard allowed time |
| Messaging: `Stream`, `StreamEntry`, `ConsumerGroup`, `Delivery`, dead-letter record | Stream owner defines policy; group owner controls consumption | Append, visible, leased, acknowledged, retried, dead-lettered, expired | Immutable stream positions; delivery attempt is a new version | Stream retention is time/size/policy based; recoverable user delete enters bin; acknowledged delivery history follows audit policy | Stream owns entries and groups; group owns deliveries and offsets |
| Derived data: `Index`, index schema, build, query snapshot | Collection owner owns definition; Pulse owns rebuild mechanics | Requested, building, current, stale, failed, rebuilding, retired | Index version binds schema and canonical log position | Index may be deleted and rebuilt; it cannot be the only copy of truth | Index derives from one or more collections and records freshness positions |
| Operations: `PlacementPlan`, `Migration`, `Backup`, `Restore`, `Incident`, recovery item | Split operator roles by action | Proposed, simulated, approved, running, paused, failed, completed/rolled back | Every run has immutable intent/version and checkpoint sequence | Operational proof follows audit retention; backups follow data policy; failed disposable targets are cleaned after evidence | Plans create migrations; backups feed restores; incidents link affected shards and actions |
| Authority and proof: `CapabilityGrant`, `Quota`, access event, `ProofBundle` | Data owner grants within security ceilings; proof owner controls disclosure | Draft, granted/denied, active, expired/revoked; proof draft, signed, verified, superseded | Grants and proof are immutable signed versions; changes create new events | Secret values are never retained; metadata follows security/audit policy and legal hold | Grants bind subject, tenant, collection, fields, operations, purpose, and time; proof links all domain records |

## 9. System architecture

- A Rust storage kernel owns WAL, memory tables, indexes, compaction, snapshots,
  checksums, recovery, and deterministic state-machine application.
- A native typed protocol exposes all semantics. Compatibility protocols map
  into it and return explicit unsupported errors.
- A consensus layer replicates independent shards and records membership as log state.
- A placement controller proposes plans but storage nodes enforce safety locally.
- Background work has durable jobs, checkpoints, quotas, cancellation, and backpressure.
- HelixCore supplies identity, policy, capabilities, audit, deployment health,
  and recovery workflows without entering the storage hot path for every read.

## 10. Agent and automation contract

Every agent works through a named, time-bounded capability lease. A human can
see the lease, current action, last fresh signal, changed records, checks, and
safe stop before approving any effect.

| Agent | Reads | Tools | Drafts | Approval | Forbidden | Checking | Stop or reversal |
|---|---|---|---|---|---|---|---|
| Schema Assistant | Approved schemas, sampled metadata, compatibility report, and policy limits | Schema validator, migration simulator, and disposable test collection | New schema version, migration plan, rollback plan, and risk note | Schema authority activates a version and separately approves a destructive conversion | Apply a schema, read unapproved values, weaken retention, or hide incompatible records | Validate every record class, compare before/after counts and digests, and run rollback rehearsal | Cancel deletes the disposable target; an active migration pauses at a checkpoint and keeps the old schema authoritative |
| Capacity Advisor | Fresh capacity, load, latency, placement, cost, and failure-domain metadata | Forecast, placement simulator, and read-only topology model | Ranked capacity and placement options with assumptions and confidence | Operator approves placement; cutover keeps its separate approval | Move data, add nodes, change collection policy, or present an estimate as a measured fact | Reject stale inputs, replay at least three demand cases, and show policy violations | Withdraw the proposal; any approved movement uses the normal checkpointed migration reversal |
| Incident Guide | Approved health checks, logs, metrics, membership, recent changes, and runbooks | Read-only diagnostics, timeline builder, and evidence exporter | Incident timeline, hypotheses, safe checks, and operator choices | Human approves every state-changing check, failover, restore, or resolution | Force quorum, discard a replica, clear evidence, expose values, or declare resolution | Mark each statement observed, inferred, or unknown; refresh sources before a green claim | Stop all checks immediately; drafts remain evidence and make no cluster change |
| Memory Curator | Source-linked entries inside one approved project, purpose, and retention class | Provenance validator, contradiction linker, expiry preview, and scoped memory writer | Add, supersede, contradict, expire, or remove proposals | Project owner approves policy changes and any cross-project derived view | Read raw secrets, cross tenant or project boundaries, change retention, or turn unsupported text into fact | Require source, confidence, purpose, expiry, and isolation checks for every entry | Revoke the lease; rejected drafts have no effect and approved entries can be tombstoned under policy |
| Recovery Operator | Backup manifests, checksums, target inventory, policy, and recovery drill history | Restore planner, disposable restore namespace, verifier, and cutover preview | Recovery point, target, validation plan, cutover, and rollback steps | Restore authority approves the drill; a separate owner approves live cutover | Restore over live data, skip validation, alter source backup, or purge the 30-day bin | Verify signatures, checksums, schema, logical digest, isolation, and application read checks | Cancel removes the disposable target; failed cutover returns authority to the unchanged source |

No agent may force quorum, discard replicas, skip durable flush, restore over
live data, widen its own grant, or declare an incident resolved. Long operations
show bytes and shards moved, checkpoint, lag, throttle, estimated range,
failures, and safe cancel state.

## 11. Trust, safety, and privacy

Authentication is mutual between nodes and capability-based for clients.
Tenant and collection identity is part of every storage key and log entry.
Encryption keys come from the broker and are not exposed through the data API.
Snapshots are encrypted, signed, and policy-labeled. Admin power is split among
read, schema, placement, backup, restore, delete, and break-glass roles. Durable
user deletion enters a 30-day bin; TTL cache expiry is intentionally final and
labeled non-recoverable. Quorum loss fails closed for strong collections.

## 12. Proof and audit

Proof covers collection contract, accepted write position, batch identity,
replica set, snapshot hashes, membership changes, placement plans, schema
migrations, backups, restores, deletion, access grants, failover, and recovery
drills. High-volume reads produce signed aggregates unless exact per-read audit
is required. Evidence proves Pulse behaviour, not client correctness. Aether is
preferred for external verification; local signed manifests remain mandatory.

## 13. UX system

The main surfaces are Overview, Data, Streams, Queues, Indexes, Collections,
Cluster, Placement, Performance, Backups, Incidents, Evidence, and Recovery.
Simple mode answers “is my data safe and what needs me?” Deep mode reveals
terms, log positions, replica lag, compaction, memory, disk, network, and shard
maps. Health colors require fresh evidence. Deleting a durable collection shows
size, dependants, retention, and recovery date. Selection and movement show an
exact preview. Completion and failed-operation notices persist until read.

## 14. Interoperability and standards

- [RESP2 and RESP3](https://redis.io/docs/latest/develop/reference/protocol-spec/)
  are optional client compatibility adapters, verified from the official
  protocol specification on 2026-07-15. They do not define native Pulse semantics.
- [Raft](https://raft.github.io/index.html) is the default crash-fault consensus
  model for G2, verified from the authors' paper site on 2026-07-15. The actual
  state machine and membership protocol require a checked specification and
  fault-injection tests before production use.

Import and compatibility reports list unsupported commands, different expiry,
transaction, ordering, and persistence behaviour.

## 15. Cross-platform contract

Single-node durable mode, CLI, inspector, snapshot validation, and development
cluster pass on Windows, macOS, and Linux. Production cluster support begins on
Linux after G2, while the same correctness model remains tested everywhere.
Filesystem, locking, clock, networking, and crash tests use native runners on
all three systems. Containers are optional. A user can run one binary without WSL.

## 16. Reliability and performance budgets

Targets apply to published reference hardware and declared collection classes.
Every release records the hardware, workload, dataset, fault model, and result.

| Contract | Numeric target and window | Required proof | Degraded behaviour |
|---|---|---|---|
| Acknowledged durability | Zero acknowledged durable writes lost across at least 1,000,000 writes and 10,000 injected crash points per release | Write receipts, crash schedule, recovered log positions, and state digest | Reject or return `unknown`; never report false success |
| Atomic batches | Zero torn or partly visible batches across 100,000 crash/retry cases per release | Before/after digest and batch-to-log trace | Whole batch is rejected, retried by the same key, or remains unknown |
| Idempotency | Repeating a request for 24 hours returns the original logical outcome in 100% of 1,000,000 duplicate cases | Request key, retained receipt, response hash, and expiry rule | After the declared window, return an explicit expired-key result, not a second silent effect |
| Concurrency | Reference cluster sustains 10,000 connected clients and 100,000 in-flight operations with zero isolation or ordering violations | Load trace, model comparison, queue depth, and tail latency | Admit less work, apply fair backpressure, and keep accepted work ordered |
| Offline local mode | One-node read, write, inspect, backup, restore, and export work for 72 hours with network disabled | Native-OS offline journey report and artifact verification | Remote adapters show unavailable; local durable work continues |
| Quorum and partitions | Strong writes stop within 5 seconds of proven quorum loss; zero known stale reads are labeled linearizable in 1,000 partition campaigns per release | Terms, votes, client receipts, partition schedule, and oracle result | Strong path fails closed; weaker declared classes show lag and semantics |
| Local latency | On the reference host, 1 KiB in-memory read p99 is below 2 ms and durable 1 KiB write p99 below 10 ms over a 30-minute run | Reproducible benchmark, raw histogram, build ID, and host profile | Threshold breach marks the service degraded and enables bounded backpressure |
| Scale | G3 reference test stores 100 TB logical data, 10 billion keys, and 10,000 shards while completing inspection within 10 seconds | Capacity manifest, shard map, query trace, and logical digests | Pagination, sampling, and background work degrade before correctness |
| Node recovery | Metadata RPO is zero; a three-node reference cluster regains one-failure tolerance within 60 seconds of failure detection in 99 of 100 drills | Detection time, replay/checkpoint progress, replica digest, and final health | Continue only within the collection contract; otherwise become read-only or blocked |
| Backup and restore | Every release completes 30 cross-OS restores with matching logical digests; reference 1 TB restore RTO is under 60 minutes | Signed manifest, checksums, timings, application read checks, and cutover result | Failed validation leaves source authoritative and target disposable |
| Cancellation and resume | 100% of 1,000 migration, index, backup, and restore cancellations reach a safe checkpoint within 10 seconds and resume without duplicate effect | Cancel receipt, checkpoint, resource release, and resumed digest | If immediate stop is unsafe, show the exact bounded step and estimated stop time |
| Resource pressure | Backpressure begins by 80% of configured memory or disk queue budget; no accepted operation is dropped in 100 exhaustion runs | Resource curves, admission decisions, client errors, and recovery trace | Reject new work fairly, preserve committed state, and keep health inspectable |
| Subsystem degradation | Loss of index, advisor, console, or one non-voting replica does not stop canonical one-node data service in 100 restart drills | Process lifetime, restart count, canonical read/write checks, and UI state | Affected feature becomes stale/offline; canonical data remains available by contract |

## 17. Success measures

| Outcome | Threshold and time window | Measurement |
|---|---|---|
| Durable truth | Zero lost acknowledged writes in the release suite and in a quarterly 1,000,000-write fault campaign | Independent receipt-to-recovery reconciliation |
| Honest uncertainty | 100% of unknown write outcomes return the original idempotency handle; false success count stays zero each release | Fault-injected client trace review |
| Recovery | 30 of 30 cross-OS restores pass each release; at least 11 of 12 monthly operator drills meet the declared RTO | Signed restore and drill reports |
| Incident understanding | At least 90% of operators identify affected collection, last safe position, and next safe action within 10 minutes in a quarterly 20-person study | Timed study with task-level results |
| Reshard control | At least 99.9% of 10,000 quarterly pause, resume, and cancel trials finish without duplicate ownership; any failure blocks the next gate | Ownership oracle and checkpoint trace |
| Tenant isolation | Zero cross-tenant or cross-project access in 100,000 adversarial policy cases per release | Boundary-test report with grant and denial events |
| Portability | All 30 Windows/macOS/Linux export-import routes match logical digests each release | Clean-machine import and verifier output |
| Accessible operation | All seven signature journeys pass keyboard, screen-reader, 200% text, reduced-motion, and color-independent checks each release | Native accessibility journey report |
| Performance stability | No p99 latency or energy-per-operation regression above 10% against the prior release without a written, approved explanation | Repeated reference benchmark and confidence interval |
| Pilot usefulness | By the end of each 90-day G1 pilot, at least 80% of active teams complete weekly backup verification and fewer than 1 support incident per team-month concerns hidden state | Product telemetry limited to approved metadata plus support classification |

Node count, message count, and raw benchmark peaks are not success measures.

## 18. Delivery plan

| Gate | Build | Tests | Safety | UX | Windows / macOS / Linux | Migration | Operator proof |
|---|---|---|---|---|---|---|---|
| G0 — truthful foundation | Typed collection model, WAL, atomic batch, health truth, crash harness | 10,000 crash points; 100,000 atomic cases; deterministic reference model | No other product critical path; deny-by-default local policy; no false green state | Start local, inspect truth, crash, and recover journeys show durable progress | Native build, crash, filesystem, locking, clock, and fixture suites pass on all three | No production import; only synthetic disposable state; rollback removes the scaffold | Operator can prove one write, one forced crash, exact recovery, and zero silent loss |
| G1 — useful single-player product | One-node types, streams, queues, leases, inspector, backup, restore, export, and supported compatibility subset | 1,000,000 mixed operations; 30 cross-OS restores; all seven local journeys | Split admin roles, 30-day durable-data bin, quotas, and brokered keys | Keyboard and screen-reader complete all seven journeys; slow work and notices stay visible | Signed CLI and inspector packages pass native installer/update/uninstall and 72-hour offline runs | Supported import reports every semantic difference; source remains unchanged until verified cutover | Ten pilot operators complete install, backup, restore, delete, recover, and leave without developer help |
| G2 — trusted team product | Replicated shards, membership, resharding, capability policy, and placement control | Formal model has no safety counterexample; 1,000 partition campaigns; 10,000 reshard trials | Quorum loss fails closed; no universal admin; revocation and residency enforced in protocol and storage | Cluster state shows evidence age, replica lag, unknown outcomes, checkpoints, and safe stop | Three-node development clusters pass the same semantics suite on all three; Linux production pilot only | NATS and prior stores migrate through dual-read validation and reversible cutover | Independent operator runs node-loss, partition, reshard, revoke, and restore drills from signed runbooks |
| G3 — category leader | Tiering, rebuildable indexes, multi-region classes, and 100 TB operations | 10 billion-key scale test; 500 region-loss campaigns; full index delete/rebuild matches 10,000 queries | Semantics and residency cannot weaken on failover; destructive actions require separate authority | Large topology remains understandable; plan, cost, confidence, and effect preview before action | Clients and administration remain equal on all three; cluster data plane support expands only with native proof | At least two external engine adapters pass lossless export and declared-difference import | Published reproducible correctness, latency, cost, energy, and recovery report is independently repeated |
| G4 — frontier network | Owner-controlled federation, placement advisor, and verifiable agent memory | Five-cluster join/leave/revoke drills; 100,000 memory isolation cases; 50 incident replays | No global normal admin; raw data stays local; advisor has no move authority; secret values never enter memory | Owners can explain and revoke every derived view or memory lease from one place | Federation control and offline evidence verification pass on all three systems | A cluster can leave with data, policy, history, identities, and proof and revoke future flows | Independent distributed-systems, privacy, accessibility, and usability reviews approve a live founder-observed drill |

## 19. Current truth and gap

Pulse is an honest scaffold. Its status reports real capabilities as false. It
has no storage engine, WAL, protocol, replication, cluster, or tests. The first
safe slice is HP-F0-01 through HP-F0-03 plus a single durable `put/get/delete`
path and a crash/restart test. No other HelixForge product may depend on Pulse
until G1 is proven.

## 20. Decisions locked for Kimi

| Question | Locked default | Change requires |
|---|---|---|
| Build order | Last catalog product; no critical dependency before G1 | Founder decision |
| Core language | Rust storage kernel | Architecture decision with benchmark proof |
| Source of truth | Native typed model and WAL | Architecture decision |
| Consensus | Raft-style crash-fault model at G2, not before | Formal and architecture review |
| Compatibility | Adapter subset with explicit differences | Product decision |
| First cluster | Three local nodes, synthetic data | G2 safety gate |
| Admin authority | Split roles; no universal normal admin | Security review |
| Delete | Recoverable durable data; explicit non-recoverable ephemeral data | Data-policy decision |

## 21. Definition of category-defining done

- [ ] One-node and clustered modes keep the same tested data meaning.
- [ ] Every collection states and proves durability, consistency, retention, and placement.
- [ ] No accepted write, failover, migration, delete, or restore completes silently.
- [ ] Tenant, capability, and secret boundaries survive adversarial testing.
- [ ] Users can leave with logical data, schemas, policies, history, and proof.
- [ ] Windows, macOS, Linux, accessibility, crash, chaos, recovery, security,
      formal-model, and independent distributed-systems gates pass.
