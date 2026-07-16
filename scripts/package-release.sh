#!/usr/bin/env bash
# Build a HelixForge release package for the current platform.
# Usage: ./scripts/package-release.sh [-s|--skip-build]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SKIP_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    -s|--skip-build) SKIP_BUILD=1; shift ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

VERSION="$(grep '^version' services/gateway/Cargo.toml | head -1 | cut -d'"' -f2)"
RAW_OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$RAW_OS" in
  linux*) OS="linux" ;;
  darwin*) OS="macos" ;;
  msys*|mingw*|cygwin*) OS="windows" ;;
  *) OS="$RAW_OS" ;;
esac
ARCH="$(uname -m)"
[[ "$ARCH" == "x86_64" ]] && ARCH="amd64"
PKG="helixforge-${VERSION}-${OS}-${ARCH}"
OUT="target/release/packages"
mkdir -p "$OUT"
STAGE="$(mktemp -d)/$PKG"
mkdir -p "$STAGE/bin" "$STAGE/scripts" "$STAGE/deploy/local"

if [[ $SKIP_BUILD -eq 0 ]]; then
  echo "[package] building release binaries..."
  cargo build --release --workspace
fi

# Copy service binaries
for crate in gateway agent_hub vault_service billing_service observability_service auth_adapter; do
  bin="target/release/$crate"
  [[ -f "$bin" ]] && cp "$bin" "$STAGE/bin/"
done

# Copy product API binaries
for bin in target/release/helix_*_api; do
  [[ -f "$bin" ]] && cp "$bin" "$STAGE/bin/"
done

cp docker-compose.yml "$STAGE/"
cp -R deploy/local/* "$STAGE/deploy/local/"
cp scripts/install.sh "$STAGE/scripts/"
cp scripts/migrate-export.sh "$STAGE/scripts/"
cp scripts/migrate-restore.sh "$STAGE/scripts/"
cp README.md "$STAGE/" 2>/dev/null || true

cat > "$STAGE/INSTALL.md" <<EOF
# HelixForge ${VERSION} install

1. Load secrets from your key directory (not included in this package).
2. Run the installer for your platform:
   - Linux/macOS: ./scripts/install.sh
   - Windows:     .\scripts\install.ps1
3. Start core services with scripts/dev-core.* or run individual binaries from bin/.
EOF

TAR="${OUT}/${PKG}.tar.gz"
 tar -czf "$TAR" -C "$(dirname "$STAGE")" "$PKG"

# SHA-256 manifest
sha256sum "$TAR" | awk '{print $1}' > "${TAR}.sha256"

echo "[package] created $TAR"
echo "[package] sha256: $(cat "${TAR}.sha256")"
