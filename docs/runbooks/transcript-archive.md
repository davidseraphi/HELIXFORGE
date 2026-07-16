# Transcript archive (HELIXFORGE)

## Where sessions go

Machine-level archive (not in git):

```
C:\Users\divin\TRANSCRIPTS\HELIXFORGE\
  grok\      # Grok Build / CLI
  claude\    # Claude Code (when used with this cwd)
  codex\     # Codex
  cursor\    # Cursor
  …
```

Archiver source: `C:\scripts\ai\transcript-archiver\archive_transcripts.py`  
Windows task: **AI Transcript Archiver** (every 5 minutes + logon).

## How filing works

1. Tool writes local session files under `~/.grok`, `~/.claude`, etc.
2. Archiver maps **cwd / project slug** → project bucket name.
3. For this repo: `C:\Users\divin\PROJECTS\HELIXFORGE` → **`HELIXFORGE`**.

No per-project install is required beyond working inside that path. Other tools
(Claude/Codex/Cursor) land under the same `HELIXFORGE\` tree once they have
sessions tied to this workspace.

## Operator checks

```powershell
# Confirms mapping
py -3.13 -c "import sys; sys.path.insert(0, r'C:\scripts\ai\transcript-archiver'); from archive_transcripts import project_from_cwd; print(project_from_cwd(r'C:\Users\divin\PROJECTS\HELIXFORGE'))"
# → HELIXFORGE

# Task present
Get-ScheduledTask -TaskName 'AI Transcript Archiver'

# Repair task if missing
powershell -NoProfile -ExecutionPolicy Bypass -File C:\scripts\ai\transcript-archiver\install_task.ps1

# Manual archive (errors if another run is active — wait and retry)
py -3.13 C:\scripts\ai\transcript-archiver\archive_transcripts.py --verbose
```

## Limits

- Cloud-only or encrypted-without-local-export chats are **not** captured.
- Live Grok session files update continuously; archiver appends/rewrites
  safely (never shortens an archived copy).
