# HelixForge plain-English glossary

This glossary defines specialist terms used by the product sheets. A product UI
must normally use the plain meaning, not require the user to know the term.

| Term | Plain meaning |
|---|---|
| Access policy | A rule that says who may do what, to which item, for what reason, and for how long. |
| Adapter | A small boundary that translates between HelixForge and another tool, provider, device, or standard. |
| Agent | Software that can plan or act with named tools. It has no authority beyond its current lease. |
| Artifact | A file or other output made by work, such as a report, build, dataset, or image. |
| Atomic write | A group of changes that all become real together or do not become real at all. |
| Audit event | A protected record of an important action, decision, grant, denial, or change. |
| Backpressure | The system slows or rejects new work before memory, disk, or another limit causes wider failure. |
| Baseline | The named result or normal state used for a fair comparison. |
| Capability | One exact action a person or process may request, such as reading one repository. |
| Capability lease | Temporary permission for one process, action, scope, purpose, and limit. |
| Checkpoint | A durable safe point from which long work can resume after a pause or crash. |
| CI | Automatic build and test work run against a change. It means continuous integration. |
| Consistency | The promise about which version of shared data a reader is allowed to see. |
| Consensus | A method that helps several machines agree on one ordered state after failures. |
| CRDT | A data structure that can merge approved changes made on different devices without one central editor. |
| DAP | Debug Adapter Protocol. It lets an editor talk to different debuggers through one common message format. |
| Data residency | A rule about the physical or legal region in which data may be stored or processed. |
| Digital twin | A versioned model of a real system used to understand, simulate, and compare behaviour. It is not the real system. |
| Domain engine | The real rules and calculations that make a product different from a generic data form. |
| Evidence | Source material, observations, checks, approvals, and outputs that support or limit a result. |
| Federation | Independent systems cooperate through narrow agreements while each owner keeps its own control. |
| Formal model | A precise mathematical description used to test whether a system can enter unsafe or impossible states. |
| `fsync` | An operating-system request to push written data toward durable storage before reporting success. |
| Gate | A set of fresh checks that must pass before work may move to a higher maturity level. |
| Hardware-in-the-loop | A test where software controls or reads real or representative hardware inside a safe test setup. |
| Idempotency | Repeating the same request with the same identity has the same logical result instead of creating duplicates. |
| Immutable | Not changed in place. A correction becomes a new linked version. |
| Intermediate representation | A common internal program form used between a source language and different execution targets. |
| Lease | Permission or ownership that expires unless it is renewed. |
| Linearizable | A strong data promise: once a write succeeds, later strong reads behave as if operations happened in one real-time order. |
| LSP | Language Server Protocol. It lets an editor ask different language tools for completion, symbols, errors, and navigation. |
| Migration | A controlled change from an old data or contract shape to a new one. |
| Model | A calculation or representation of a system. Its output is not automatically true. |
| Multi-tenant | One installation serves several separate owners while keeping their data and authority apart. |
| Outbox | Durable records of external messages that still need to be sent after the main database change commits. |
| Provenance | Where an item came from, what changed it, who or what acted, and which version was used. |
| QIR | Quantum Intermediate Representation. It is a common compiler form for hybrid quantum and classical programs. |
| Quorum | The minimum number of cluster members that must agree before a protected operation can continue. |
| Recovery point objective (RPO) | The maximum amount of recent data the declared recovery plan may lose. |
| Recovery time objective (RTO) | The target time for restoring the declared service after failure. |
| Reproducible | Another allowed person or machine can rebuild or rerun the work from its recorded inputs and method. |
| RO-Crate | An open package format that keeps research files together with structured descriptions and relationships. |
| Sandbox | An isolated place with strict file, network, process, time, and resource limits. |
| Semantic | About meaning, not only text or file shape. |
| Snapshot | A consistent saved view of system state at a named point. |
| Tombstone | A durable marker that an item was deleted, often kept so identity, replication, and recovery stay correct. |
| Tenant | One person or organization whose data and authority must be separated from other owners. |
| Vector index | An index that finds items with similar numeric representations. Similar does not mean correct or equivalent. |
| Washout | A planned period in a study used to reduce the effect of an earlier treatment or condition. |
| WebAuthn | A web standard for authentication with public-key credentials controlled through a browser and authenticator. |
| Workload identity | A stable identity for one approved program or process, separate from the human who started it. |

