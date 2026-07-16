#!/usr/bin/env bash
# Restore HelixCore Postgres + MinIO state from a backup directory.
# Usage: ./scripts/migrate-restore.sh <BACKUP_DIR> [PROJECT_NAME]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BACKUP_DIR="${1:-}"
PROJECT_NAME="${2:-${PROJECT_NAME:-helixforge}}"
if [[ -z "$BACKUP_DIR" ]]; then
  echo "Usage: $0 <BACKUP_DIR> [PROJECT_NAME]"
  exit 1
fi

if [[ "$PROJECT_NAME" != "helixforge" ]]; then
  echo "[restore] using isolated compose project: $PROJECT_NAME"
fi

export COMPOSE_PROJECT_NAME="$PROJECT_NAME"

# Use alternate host ports for isolated restore so the main project can keep running.
if [[ "$PROJECT_NAME" != "helixforge" ]]; then
  export HELIX_POSTGRES_PORT="${HELIX_POSTGRES_PORT:-55433}"
  export HELIX_NATS_PORT="${HELIX_NATS_PORT:-4223}"
  export HELIX_NATS_MONITOR_PORT="${HELIX_NATS_MONITOR_PORT:-8223}"
  export HELIX_MINIO_PORT="${HELIX_MINIO_PORT:-9002}"
  export HELIX_MINIO_CONSOLE_PORT="${HELIX_MINIO_CONSOLE_PORT:-9003}"
fi

DUMP_FILE="$BACKUP_DIR/helixforge.sql"
MINIO_DIR="$BACKUP_DIR/minio-mirror"

if [[ ! -f "$DUMP_FILE" ]]; then
  echo "[restore] ERROR: missing $DUMP_FILE"
  exit 1
fi

echo "[restore] stopping and wiping existing project data..."
docker compose -f docker-compose.yml down -v || true

echo "[restore] starting fresh infrastructure..."
docker compose -f docker-compose.yml up -d postgres nats minio minio-init

# Wait for Postgres
PG_HOST_PORT="${HELIX_POSTGRES_PORT:-55432}"
PG_READY=false
for i in $(seq 1 30); do
  if docker compose -f docker-compose.yml exec -T postgres pg_isready -U helix -d helixforge >/dev/null 2>&1; then
    echo "[restore] postgres ready"
    PG_READY=true
    break
  fi
  sleep 2
done
if [[ "$PG_READY" != "true" ]]; then
  echo "[restore] ERROR: postgres did not become ready"
  exit 1
fi

echo "[restore] restoring postgres..."
docker compose -f docker-compose.yml exec -T postgres psql -U helix -d helixforge < "$DUMP_FILE"

echo "[restore] restoring minio..."
if [[ -d "$MINIO_DIR" ]]; then
  MINIO_ENDPOINT="http://127.0.0.1:${HELIX_MINIO_PORT:-9000}"
  for bucket in helixforge helix-collab helix-code; do
    if [[ -d "$MINIO_DIR/$bucket" ]]; then
      docker run --rm --network host --entrypoint /bin/sh \
        -v "$MINIO_DIR:/source" minio/mc:latest \
        -c "mc alias set restore $MINIO_ENDPOINT helixminio helixminio_secret >/dev/null 2>&1 && mc mirror --overwrite /source/$bucket restore/$bucket" >/dev/null 2>&1 || true
    fi
  done
fi

echo "[restore] verification..."
docker compose -f docker-compose.yml exec -T postgres pg_isready -U helix -d helixforge >/dev/null 2>&1 || { echo "ERROR: postgres not ready after restore"; exit 1; }

echo "[restore] done."
echo ""
echo "Verify with: DATABASE_URL=postgres://helix:helix@127.0.0.1:$PG_HOST_PORT/helixforge cargo test -p helix_db"
