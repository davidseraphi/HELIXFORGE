# CI test wrapper — Windows (MSVC)
# Usage: powershell -File scripts/ci-test.ps1
$ErrorActionPreference = "Stop"
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"
cargo test --workspace --all-features
