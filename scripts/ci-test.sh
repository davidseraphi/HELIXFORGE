#!/usr/bin/env bash
# CI test wrapper — POSIX (Linux/macOS)
set -euo pipefail
cargo test --workspace --all-features
