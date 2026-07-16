# HelixCollab threat model (sovereign)

## Claims

1. With **client_e2ee + sealed CRDT**, cluster operators cannot read document body or Yjs ops.
2. Metadata (ACL graph, sizes, times, classification) is visible to operators — by design.
3. Break-glass is **threshold recovery** only; no silent admin decrypt.
4. Agent endpoints refuse HC1; selection must be unsealed in the client sandbox.

## Server sees

- Tenant / user / document identifiers
- Versions, sizes, attachment object keys (not plaintext if sealed)
- ACL principals and permissions
- Classification labels, residency claims
- Device **public** keys
- Sealed blob lengths and JetStream subjects
- Audit hash-chain events (actions, not content)

## Server never sees

- Passphrases, private keys, unwrapped DEKs
- HC1 plaintext (bodies, sealed CRDT, sealed presence, sealed comments when enabled)
- Attachment bytes when `client_sealed=true` (ciphertext at object store)

## Compromise scenarios

| Asset compromised | Impact |
|-------------------|--------|
| Single laptop | Docs unlocked on that device; revoke device key + rotate DEK shares |
| Postgres + MinIO | Metadata + ciphertext; no plaintext without keys/escrow |
| NATS | Fan-out of opaque envelopes + presence ids |
| Operator with shell | Same as cluster; cannot forge audit chain without HMAC key |

## Horizons

- **A** device keys, export backpack, classification policy, durable sealed CRDT
- **B** spaces, offline protocol, attachments meta, client agent
- **C** MLS stub, threshold ceremonies, residency proofs, federation receipts

See `/v1/sovereign/threat-model` and `/v1/sovereign/capabilities`.
