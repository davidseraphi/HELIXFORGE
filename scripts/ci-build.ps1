# CI build wrapper — Windows (MSVC)
# Usage: powershell -File scripts/ci-build.ps1
$ErrorActionPreference = "Stop"
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"
cargo build --workspace
