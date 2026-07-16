# Bugs registry

The packet is the authority; this file is the quick index. One row per material bug.

| ID | Severity | Priority | Status | Surface | Title |
|---|---|---|---|---|---|
| _(none yet — zero packets is a clean pass)_ | | | | | |

## Gardening rule

Every quality batch starts by reading this file, then the packet for every S0/S1/P0/P1 item.
Do not continue broad feature work while an S0 or unmitigated S1 bug is newly reproduced.
Validate with `python tools/quality/validate_bug_packets.py` (also runs inside the Tier-0 gate).
File new bugs with the `/file-bug` skill.
