#!/usr/bin/env python3
"""Scaffold HelixForge monorepo structure (idempotent)."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PRODUCTS = [
    ("helix-collab", "HelixCollab", "Real-time collaborative workspace", 1, "standard"),
    ("helix-code", "HelixCode", "AI-native collaborative IDE", 2, "standard"),
    ("helix-flow", "HelixFlow", "Agentic automation & workflow engine", 3, "standard"),
    ("helix-insights", "HelixInsights", "Predictive analytics & decision OS", 4, "standard"),
    ("helix-commerce", "HelixCommerce", "AI e-commerce & digital marketplace builder", 5, "standard"),
    ("helix-edu", "HelixEdu", "Adaptive AI learning & certification platform", 6, "standard"),
    ("helix-capital", "HelixCapital", "AI financial operating system", 7, "standard"),
    ("helix-well", "HelixWell", "AI personal & team wellness platform", 8, "standard"),
    ("helix-network", "HelixNetwork", "AI professional networking & opportunity engine", 9, "standard"),
    ("helix-forge-studio", "HelixForge Studio", "No-code/low-code AI app & internal tool builder", 10, "standard"),
    ("helix-synthbio", "HelixSynthBio", "Synthetic biology design & virtual wet-lab", 11, "frontier"),
    ("helix-lex-prime", "HelixLexPrime", "Autonomous legal & regulatory intelligence", 12, "frontier"),
    ("helix-cura-prime", "HelixCuraPrime", "Enterprise clinical AI platform", 13, "frontier"),
    ("helix-terra-prime", "HelixTerraPrime", "Precision agriculture & climate-smart farming OS", 14, "frontier"),
    ("helix-climate-prime", "HelixClimatePrime", "Planetary-scale climate risk modeling & net-zero orchestration", 15, "frontier"),
    ("helix-orbit-prime", "HelixOrbitPrime", "Commercial space operations & satellite intelligence", 16, "frontier"),
    ("helix-quantum-forge", "HelixQuantumForge", "Hybrid quantum-classical computing platform", 17, "frontier"),
    ("helix-vita-prime", "HelixVitaPrime", "Precision medicine & longevity research platform", 18, "frontier"),
    ("helix-grid-prime", "HelixGridPrime", "Autonomous smart energy systems & renewable optimization", 19, "frontier"),
    ("helix-nova-labs", "HelixNovaLabs", "Open scientific discovery accelerator", 20, "frontier"),
]

CORE_CRATES = [
    "shared-core",
    "agent-framework",
    "vault-client",
    "auth-client",
    "nats-client",
    "audit-log",
    "billing-client",
    "observability",
    "service-kit",
]

CORE_SERVICES = [
    ("gateway", 8080, "API gateway / BFF edge"),
    ("agent-hub", 8081, "Agent orchestration hub"),
    ("vault-service", 8082, "Secrets & envelope encryption"),
    ("billing-service", 8083, "Usage metering & billing"),
    ("observability-service", 8084, "Metrics, traces, audit sink"),
    ("auth-adapter", 8085, "Ory Kratos/Hydra adapter"),
]

DIRS = [
    "crates",
    "services",
    "projects",
    "apps/console",
    "apps/console/src/app",
    "apps/console/src/components",
    "packages/ui",
    "packages/sdk-ts",
    "packages/config-eslint",
    "packages/config-typescript",
    "infra/terraform/modules/network",
    "infra/terraform/modules/kubernetes",
    "infra/terraform/modules/postgres",
    "infra/terraform/modules/nats",
    "infra/terraform/modules/minio",
    "infra/terraform/modules/ory",
    "infra/terraform/environments/dev",
    "infra/terraform/environments/staging",
    "infra/terraform/environments/prod",
    "infra/helm/helix-core",
    "infra/helm/helix-core/templates",
    "infra/helm/charts",
    "infra/argocd/applications",
    "infra/argocd/projects",
    "infra/docker",
    "deploy/local",
    "docs/adr",
    "docs/architecture",
    "docs/features/_template",
    "docs/features/000-helix-core-bootstrap",
    "docs/bugs/_template",
    "docs/quality",
    "docs/runbooks",
    "docs/api",
    "schemas",
    "tools/context",
    "tools/quality",
    "scripts",
    "tests/integration",
    "tests/e2e",
    ".github/workflows",
    ".github/ISSUE_TEMPLATE",
]


def write(path: Path, content: str, *, force: bool = False) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() and not force:
        return
    path.write_text(content, encoding="utf-8", newline="\n")


def main() -> None:
    for d in DIRS:
        (ROOT / d).mkdir(parents=True, exist_ok=True)

    members: list[str] = []
    for crate in CORE_CRATES:
        members.append(f"crates/{crate}")
        crate_dir = ROOT / "crates" / crate
        crate_dir.mkdir(parents=True, exist_ok=True)
        (crate_dir / "src").mkdir(exist_ok=True)
        write(
            crate_dir / "Cargo.toml",
            f'''[package]
name = "{crate.replace("-", "_")}"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
description = "HelixForge shared crate: {crate}"

[dependencies]
''',
        )
        write(crate_dir / "src" / "lib.rs", f"//! HelixForge shared crate: `{crate}`.\n\n")

    for name, port, desc in CORE_SERVICES:
        members.append(f"services/{name}")
        svc = ROOT / "services" / name
        (svc / "src").mkdir(parents=True, exist_ok=True)
        write(
            svc / "Cargo.toml",
            f'''[package]
name = "{name.replace("-", "_")}"
version = "0.1.0"
edition.workspace = true
license.workspace = true
description = "{desc}"

[[bin]]
name = "{name.replace("-", "_")}"
path = "src/main.rs"

[dependencies]
''',
        )
        write(
            svc / "src" / "main.rs",
            f'''//! {desc}
//! Default port: {port}

fn main() {{
    println!("{{name}} service starting on port {{port}}");
}}
'''.replace("{name}", name).replace("{port}", str(port)),
        )

    for slug, title, desc, order, tier in PRODUCTS:
        members.append(f"projects/{slug}/backend")
        base = ROOT / "projects" / slug
        for sub in [
            "backend/src",
            "backend/tests",
            "web/src/app",
            "web/src/components",
            "web/public",
            "docs",
            "migrations",
        ]:
            (base / sub).mkdir(parents=True, exist_ok=True)

        write(
            base / "README.md",
            f"""# {title}

**Order:** {order} · **Tier:** {tier}

{desc}

## Architecture

- Backend: Rust (Axum) — reuses HelixCore via `service-kit`, `auth-client`, `nats-client`, `agent-framework`
- Frontend: Next.js 15 App Router
- Data: PostgreSQL (Citus/Timescale via HelixCore)
- Events: NATS JetStream subjects `helix.{slug}.*`
- Objects: MinIO bucket `helix-{slug}`

## Local development

```bash
# from monorepo root
cargo run -p {slug.replace("-", "_")}_api
cd projects/{slug}/web && pnpm dev
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

See `backend/src/domain/` for hexagonal domain logic.
""",
        )

        crate_name = f"{slug.replace('-', '_')}_api"
        write(
            base / "backend" / "Cargo.toml",
            f'''[package]
name = "{crate_name}"
version = "0.1.0"
edition.workspace = true
license.workspace = true
description = "{title} API — {desc}"

[[bin]]
name = "{crate_name}"
path = "src/main.rs"

[dependencies]
''',
        )
        write(
            base / "backend" / "src" / "main.rs",
            f'''//! {title} API service.
//! {desc}

fn main() {{
    println!("{title} API — scaffold ready");
}}
''',
        )
        write(
            base / "web" / "package.json",
            json.dumps(
                {
                    "name": f"@helixforge/{slug}-web",
                    "version": "0.1.0",
                    "private": True,
                    "scripts": {
                        "dev": "next dev",
                        "build": "next build",
                        "start": "next start",
                        "lint": "next lint",
                        "typecheck": "tsc --noEmit",
                    },
                    "dependencies": {
                        "next": "^15.1.0",
                        "react": "^19.0.0",
                        "react-dom": "^19.0.0",
                    },
                    "devDependencies": {
                        "@types/node": "^22.10.0",
                        "@types/react": "^19.0.0",
                        "@types/react-dom": "^19.0.0",
                        "typescript": "^5.7.0",
                    },
                },
                indent=2,
            )
            + "\n",
        )

    # Workspace members file for reference
    write(
        ROOT / "scripts" / "workspace_members.json",
        json.dumps({"members": members, "products": [
            {"slug": s, "title": t, "description": d, "order": o, "tier": ti}
            for s, t, d, o, ti in PRODUCTS
        ]}, indent=2)
        + "\n",
        force=True,
    )
    print(f"Scaffolded {len(members)} workspace members under {ROOT}")


if __name__ == "__main__":
    main()
