# HelixEdu

**Order:** 6 · **Tier:** standard

Adaptive AI learning & certification platform

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.helix-edu.*`
- Objects: MinIO bucket `helix-helix-edu`

## Local development

```bash
# from monorepo root
cargo run -p helix_edu_api
cd projects/helix-edu/web && pnpm dev
```

## HelixCore dependencies

| Service | Use |
|---------|-----|
| gateway | Public API edge |
| auth-adapter | Identity & sessions (Ory) |
| agent-hub | AI agents |
| vault-service | Secrets |
| billing-service | Usage metering |
| observability-service | Metrics / audit |

## Domain modules

Durable learning domain lives in `helix_db` (`EduRepo`) + routes in `backend/src/main.rs`.

| Method | Path | Notes |
|--------|------|--------|
| GET/POST | `/v1/courses` | List / create courses |
| GET | `/v1/courses/{id}` | Fetch one course (excludes soft-deleted) |
| PATCH | `/v1/courses/{id}` | Update title/description/level/metadata |
| POST | `/v1/courses/{id}/publish` | Set status `published` |
| POST | `/v1/courses/{id}/unpublish` | Set status `draft` |
| POST | `/v1/courses/{id}/delete` | Soft-delete course |
| POST | `/v1/courses/{id}/restore` | Restore soft-deleted course |
| GET | `/v1/courses/{id}/enrollments` | Enrollments for a course |
| GET/POST | `/v1/enrollments` | List / enroll (learner = caller) |
| GET | `/v1/enrollments/{id}` | Fetch one enrollment |
| POST | `/v1/enrollments/{id}/progress` | `{ "progress_pct": 0..100 }` |
| POST | `/v1/enrollments/{id}/withdraw` | Withdraw enrollment |
| GET | `/v1/domain/status` | `phase: wave2_w4` + capability planes |

Auth (local): header `x-helix-dev-user: you@example.com`.
