# HelixPulse — vision

**Slug:** `helix-pulse` · **Order:** 21 · **Port:** 8121  
**Priority:** Build **last** in the HelixForge portfolio (after Core + products 1–20).

## Problem

Redis-class systems are the default for rate limits, caches, sessions, and streams — but they are:

- Weak on **multi-tenant sovereignty** (shared process, weak isolation)
- Weak on **residency / purpose / audit** out of the box
- Either **vendor-gated** or “just another data plane” you must operate forever
- Not integrated with Helix identity, billing, or governance

HelixPulse is HelixForge’s answer: a **sovereign distributed memory plane** designed for the same zero-trust spine as the other forges.

## North star

A self-hosted, multi-region-capable **cluster data plane** that products use for:

- Shared rate limiting & short-TTL counters  
- Encrypted ephemeral / semi-durable KV  
- Streams / pub-sub (where NATS is not enough at the edge)  
- Agent scratchpads & workflow checkpoints  
- Optional Redis-protocol **compatibility gateway** for migration  

Not: “faster Redis clone for benchmarks.”

## Non-goals (especially for v0–v1)

- Replacing Postgres or MinIO as system of record  
- Full Redis command surface day one  
- Replacing NATS as the HelixCore event bus  
- Building cluster consensus before product demand exists  

## Differentiated principles

1. **Tenant-first keys** — every key namespaced by tenant; cross-tenant impossible by API  
2. **Residency** — shards / replicas honor `residency_region`  
3. **Envelope crypto** — values sealable with Vault DEKs / purpose tags  
4. **Audit by default** — security-sensitive ops on the hash chain  
5. **Mesh native** — mTLS, no flat “trust the VPC”  
6. **Metered** — usage events to Helix billing  

## Phased roadmap (do not reorder)

| Phase | When | Scope |
|-------|------|--------|
| **P0 Scaffold** | Now | Catalog entry, API shell, vision, deferred flag |
| **P1 Embedded** | After products 1–20 | Single-node in-process/library KV + TTL + INCR for Core rate limit |
| **P2 Protocol** | Demand-driven | Subset RESP (GET/SET/DEL/EXPIRE/INCR/PUBLISH) |
| **P3 Cluster** | Last major | Shard map, replication, failover, multi-AZ |
| **P4 Multi-region** | Sovereign deploy | Cross-region, legal hold aware eviction |

**Cluster (P3+) is explicitly last.** Do not start Raft/shard work while Core or product forges are incomplete.

## Relationship to Core today

| Need | Use until Pulse ships |
|------|------------------------|
| Rate limit (single node) | In-process limiter in `service_kit` |
| Shared rate limit | NATS KV or Postgres |
| Messaging | NATS JetStream |
| Secrets | Vault |
| Durable state | Postgres |

## Success criteria (FULL for this product)

- [ ] Multi-node cluster deploys via Helm with secrets fail-closed  
- [ ] Tenant isolation tests (cross-tenant get fails)  
- [ ] Residency fail-closed  
- [ ] Integration: gateway rate limit backend = Pulse  
- [ ] Optional RESP subset greened against a subset of redis-rb/ioredis tests  
- [ ] Independent Kimi (or peer) review: COMPLETE  

## Name

**HelixPulse** — “pulse” of the platform: live counters, caches, and cluster heartbeat.  
Avoid “HelixRedis” branding; compatibility is a gateway, not identity.
