# HelixForge — Claude entrypoint

@AGENTS.md

## Claude notes

- Prefer extending `service_kit` / product domain modules over new ad-hoc services.
- Secrets only under `~/Desktop/.keys/helixforge/`.
- Do not background Next.js or `cargo run` servers for the user — emit CMD recipes.
