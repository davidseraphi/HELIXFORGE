# Live Ory Kratos (local)

```powershell
cd C:\Users\divin\PROJECTS\HELIXFORGE
docker compose --profile ory up -d
# wait until http://127.0.0.1:4433/health/ready is 200

$env:KRATOS_PUBLIC_URL = "http://127.0.0.1:4433"
$env:KRATOS_ADMIN_URL = "http://127.0.0.1:4434"
# start auth-adapter / gateway with those env vars
```

## Register + login

```powershell
$body = @{ email = "you@helixforge.local"; password = "password123456" } | ConvertTo-Json
Invoke-RestMethod http://127.0.0.1:8085/v1/ory/register -Method POST -ContentType application/json -Body $body
$login = Invoke-RestMethod http://127.0.0.1:8085/v1/ory/login -Method POST -ContentType application/json -Body $body
$token = $login.data.session_token
Invoke-RestMethod http://127.0.0.1:8080/v1/me -Headers @{ Authorization = "Bearer $token" }
```

## Status

```powershell
Invoke-RestMethod http://127.0.0.1:8085/v1/ory/status
Invoke-RestMethod http://127.0.0.1:8085/v1/auth/health   # mode=kratos when up
```
