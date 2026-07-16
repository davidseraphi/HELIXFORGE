#!/usr/bin/env bash
# HelixForge local installer for macOS/Linux.
# Usage: ./scripts/install.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# --- Secrets ---
KEYS="${HOME}/.keys/helixforge/.env.local"
if [[ -f "$KEYS" ]]; then
  echo "[install] loading secrets from $KEYS"
  set -a
  # shellcheck source=/dev/null
  source "$KEYS"
  set +a
else
  echo "[install] no secrets file at $KEYS — continuing with defaults"
fi

# --- Defaults ---
export HELIX_ENV="${HELIX_ENV:-local}"
export HELIX_LOCAL_DEV_UNSAFE="${HELIX_LOCAL_DEV_UNSAFE:-1}"
export HELIX_ALLOW_DEV_HEADERS="${HELIX_ALLOW_DEV_HEADERS:-1}"
export HELIX_DEV_PLATFORM="${HELIX_DEV_PLATFORM:-1}"
export DATABASE_URL="${DATABASE_URL:-postgres://helix:helix@127.0.0.1:55432/helixforge}"
export NATS_URL="${NATS_URL:-nats://127.0.0.1:4222}"
export MINIO_ENDPOINT="${MINIO_ENDPOINT:-http://127.0.0.1:9000}"
export HELIX_VAULT_MASTER_KEY="${HELIX_VAULT_MASTER_KEY:-local-dev-vault-master-key-not-for-prod}"
export HELIX_AUDIT_HMAC_SECRET="${HELIX_AUDIT_HMAC_SECRET:-local-audit-hmac-dev-only}"
export HELIX_WEBHOOK_ALLOW_UNSIGNED="${HELIX_WEBHOOK_ALLOW_UNSIGNED:-1}"

# --- Prerequisites ---
fail() { echo "[install] ERROR: $1" >&2; exit 1; }

check_cmd() { command -v "$1" >/dev/null 2>&1; }

check_cmd docker || fail "Docker is required. Install Docker Desktop or docker-ce."
check_cmd cargo || fail "Rust/Cargo is required. Install from https://rustup.rs"
check_cmd pnpm || fail "pnpm is required. Install: npm install -g pnpm"
check_cmd node || fail "Node.js is required. Install Node 20+."

if docker compose version >/dev/null 2>&1 || docker-compose version >/dev/null 2>&1; then
  COMPOSE="docker compose"
else
  fail "Docker Compose is required."
fi

# --- Infrastructure ---
echo "[install] starting backing services..."
$COMPOSE up -d postgres nats minio minio-init

# Wait for Postgres
PG_READY=false
for i in $(seq 1 30); do
  if docker compose exec -T postgres pg_isready -U helix -d helixforge >/dev/null 2>&1; then
    echo "[install] postgres ready"
    PG_READY=true
    break
  fi
  sleep 2
done
if [[ "$PG_READY" != "true" ]]; then
  fail "postgres did not become ready"
fi

# --- Build ---
echo "[install] building Rust workspace (this may take several minutes)..."
cargo build --workspace

echo "[install] installing JS dependencies and building console..."
pnpm install
pnpm --filter @helixforge/console build

echo "[install] done."
echo ""
echo "Next steps:"
echo "  - Start core services: ./scripts/dev-core.sh  (or scripts/dev-core.ps1 on Windows)"
echo "  - Or run gateway directly: cargo run -p gateway"
echo "  - Gateway health: curl http://127.0.0.1:8080/healthz"
