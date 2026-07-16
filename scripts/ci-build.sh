#!/usr/bin/env bash
# CI build wrapper — POSIX (Linux/macOS)
set -euo pipefail
cargo build --workspace
