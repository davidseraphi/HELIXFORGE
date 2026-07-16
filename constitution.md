# HelixForge Constitution

Immutable principles. Change only via explicit ADR + founder ratification.

1. **Sovereignty** — the platform must be fully self-hostable; no mandatory SaaS.
2. **One core** — products must not reimplement HelixCore capabilities.
3. **Zero-trust** — every request has a principal; residency is enforced.
4. **Tamper-evident audit** — security events are hash-chained and append-only.
5. **Secrets outside the tree** — credentials live in `.keys/`, never the repo.
6. **Verify before claiming** — external tool/version claims are checked, not guessed.
7. **Fail loud** — degraded dependencies surface health checks; do not silent-fake prod auth.
