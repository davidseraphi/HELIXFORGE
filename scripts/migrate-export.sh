#!/usr/bin/env bash
# Export HelixCore Postgres + MinIO state to a portable backup directory.
# Usage: ./scripts/migrate-export.sh [OUT_DIR]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TS="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="${1:-$HOME/helixforge-backups/$TS}"
mkdir -p "$OUT_DIR"

echo "[export] backup dir: $OUT_DIR"

DUMP_FILE="$OUT_DIR/helixforge.sql"

if docker compose exec -T postgres pg_isready -U helix -d helixforge >/dev/null 2>&1; then
  docker compose exec -T postgres pg_dump -U helix -d helixforge --no-owner --format=plain > "$DUMP_FILE"
  echo "[export] postgres dump: $DUMP_FILE"
  DUMP_HASH="$(sha256sum "$DUMP_FILE" | awk '{print $1}')"
else
  echo "[export] WARN: postgres container not ready"
  DUMP_FILE=""
  DUMP_HASH=""
fi

# MinIO mirror via ephemeral mc container
MINIO_DIR="$OUT_DIR/minio-mirror"
mkdir -p "$MINIO_DIR"
MINIO_ENDPOINT="${MINIO_ENDPOINT:-http://127.0.0.1:9000}"
MINIO_OK=false
for bucket in helixforge helix-collab helix-code; do
  if docker run --rm --network host --entrypoint /bin/sh \
       -v "$MINIO_DIR:/mirror" minio/mc:latest \
       -c "mc alias set local $MINIO_ENDPOINT helixminio helixminio_secret >/dev/null 2>&1 && mc mirror --overwrite local/$bucket /mirror/$bucket" >/dev/null 2>&1; then
    MINIO_OK=true
  fi
done
if [[ "$MINIO_OK" == "true" ]]; then
  echo "[export] minio mirror: $MINIO_DIR"
  MINIO_HASH="$(find "$MINIO_DIR" -type f -exec sha256sum {} \; | sha256sum | awk '{print $1}')"
else
  echo "[export] WARN: minio not reachable"
  MINIO_DIR=""
  MINIO_HASH=""
fi

COMMIT="$(git rev-parse HEAD 2>/dev/null || echo unknown)"

BACKUP_SET="[]"
if [[ -n "$DUMP_HASH" ]]; then
  BACKUP_SET="[{\"kind\":\"postgres_dump\",\"path\":\"helixforge.sql\",\"sha256\":\"$DUMP_HASH\"}]"
fi
if [[ -n "$MINIO_HASH" ]]; then
  if [[ "$BACKUP_SET" == "[]" ]]; then
    BACKUP_SET="[{\"kind\":\"minio_mirror\",\"path\":\"minio-mirror\",\"sha256\":\"$MINIO_HASH\"}]"
  else
    BACKUP_SET="${BACKUP_SET%]}, {\"kind\":\"minio_mirror\",\"path\":\"minio-mirror\",\"sha256\":\"$MINIO_HASH\"}]"
  fi
fi

cat > "$OUT_DIR/backup-manifest.json" <<EOF
{
  "timestamp": "$(date -Iseconds)",
  "git_commit": "$COMMIT",
  "database_url_host": "postgres://helix:****@127.0.0.1:55432/helixforge",
  "minio_endpoint": "http://127.0.0.1:9000",
  "backup_set": $BACKUP_SET,
  "note": "Secrets live outside this backup. Restore them separately from your key directory."
}
EOF

echo "[export] manifest: $OUT_DIR/backup-manifest.json"
echo "[export] done."
