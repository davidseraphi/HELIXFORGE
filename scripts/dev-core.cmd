@echo off
REM Start HelixCore services in separate CMD windows (user-owned).
set ROOT=%~dp0..
cd /d "%ROOT%"
set HELIX_ENV=local
set HELIX_LOCAL_DEV_UNSAFE=1

start "HelixCore gateway" cmd /k "cd /d %ROOT% && set HELIX_ENV=local && set HELIX_LOCAL_DEV_UNSAFE=1 && cargo run -p gateway"
start "HelixCore agent-hub" cmd /k "cd /d %ROOT% && set HELIX_ENV=local && set HELIX_LOCAL_DEV_UNSAFE=1 && cargo run -p agent_hub"
start "HelixCore vault" cmd /k "cd /d %ROOT% && set HELIX_ENV=local && set HELIX_LOCAL_DEV_UNSAFE=1 && cargo run -p vault_service"
start "HelixCore billing" cmd /k "cd /d %ROOT% && set HELIX_ENV=local && set HELIX_LOCAL_DEV_UNSAFE=1 && cargo run -p billing_service"
start "HelixCore observability" cmd /k "cd /d %ROOT% && set HELIX_ENV=local && set HELIX_LOCAL_DEV_UNSAFE=1 && cargo run -p observability_service"
start "HelixCore auth-adapter" cmd /k "cd /d %ROOT% && set HELIX_ENV=local && set HELIX_LOCAL_DEV_UNSAFE=1 && cargo run -p auth_adapter"

echo Started 6 HelixCore windows.
echo Gateway: http://127.0.0.1:8080/healthz
echo Catalog: http://127.0.0.1:8080/v1/catalog
