# Windows MSVC toolchain (HelixForge)

## Why

The monorepo targets `x86_64-pc-windows-msvc` (see `.cargo/config.toml`). Crates like `ring` and `windows-sys` require the MSVC linker and Windows SDK. The GNU host toolchain (`x86_64-pc-windows-gnu`) often fails with missing `gcc`/`dlltool`.

## Setup

```powershell
# Install MSVC target
rustup target add x86_64-pc-windows-msvc

# Prefer MSVC for this repo (already set in .cargo/config.toml)
# Optional: install Visual Studio Build Tools with "Desktop development with C++"

# Always use MSVC when the default host is gnu:
rustup run stable-x86_64-pc-windows-msvc cargo test --workspace
rustup run stable-x86_64-pc-windows-msvc cargo run -p gateway
```

## Env for local smoke after fail-closed auth

```powershell
$env:HELIX_ENV = "local"
$env:HELIX_ALLOW_DEV_HEADERS = "1"
$env:HELIX_DEV_PLATFORM = "1"   # only for platform APIs / ops@ smokes
# optional break-glass:
# $env:HELIX_ALLOW_AUDIT_REHASH = "1"
# $env:HELIX_WEBHOOK_ALLOW_UNSIGNED = "1"  # local webhooks only
```

## Verify

```powershell
rustup run stable-x86_64-pc-windows-msvc cargo --version
rustup run stable-x86_64-pc-windows-msvc cargo test -p vault_client -p auth_client
```
