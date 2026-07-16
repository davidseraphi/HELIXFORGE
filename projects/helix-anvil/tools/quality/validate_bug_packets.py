"""Validate every docs/bugs/BUG-* packet — portfolio-canonical Bug OS v4 validator.

Canonical upstream: ~/shared/bug-os/validate_bug_packets.py. Projects copy this to
tools/quality/validate_bug_packets.py and wire it into their Tier-0 gate (drift check or
pre-commit). Lineage: DOCUMENT_OS -> AETHER -> Investment OS -> this upstream.

Rules enforced (blocking unless noted):
- Each BUG-NNNN-<slug>/ packet has the 7 required files, none empty.
- A directory named BUG-* that violates the strict name contract (BUG-NNNN-<lowercase-slug>)
  ERRORS instead of being silently skipped — otherwise it is counted but never validated.
- bug.json validates against docs/quality/bug-packet.schema.json (jsonschema; degrades to
  structural checks when the package is absent).
- bug.json id matches the directory name.
- Lifecycle: 'fixed'/'verified' need reproduction evidence; 'verified' needs regression-gate
  commands.
- An 'ai-behavior' bug cannot be 'verified' unless its regression gate references an eval case
  (a command containing 'ai_eval') — a deterministic test cannot regress a probabilistic class.
- ADVISORY (warns, never blocks): a 'verified' packet SHOULD carry a closure_attestation
  (git-anchored commit_sha + gate_run_ref, or signed-ledger ledger_event_id).

An empty registry is NOT an error (a project may adopt the BOS before filing its first bug).
Exit non-zero iff any blocking issue is found. Warnings never block.

Repo root resolution (in order): env BUG_OS_ROOT; nearest ancestor of CWD that has docs/bugs/;
nearest ancestor of this file that has docs/bugs/; else two parents up (the tools/quality/
install location).
"""

from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path

_REQUIRED_FILES = (
    "bug.json",
    "report.md",
    "repro.md",
    "root_cause.md",
    "fix_plan.md",
    "regression_tests.md",
    "evidence.md",
)
_BUG_DIR_RE = r"^BUG-\d{4}-[a-z0-9]+(?:-[a-z0-9]+)*$"


def _has_bugs_dir(path: Path) -> bool:
    return (path / "docs" / "bugs").is_dir()


def _repo_root() -> Path:
    env = os.environ.get("BUG_OS_ROOT")
    if env:
        return Path(env).resolve()
    for base in (Path.cwd(), Path(__file__).resolve()):
        for candidate in (base, *base.parents):
            if _has_bugs_dir(candidate):
                return candidate
    # Fallback: the canonical tools/quality/ install location (root is two parents up).
    return Path(__file__).resolve().parents[2]


def validate_bug_packets(repo_root: Path | None = None) -> tuple[list[str], list[str]]:
    """Return ``(errors, warnings)`` for every docs/bugs/BUG-* packet.

    An empty registry is NOT an error. Errors block; warnings are advisory.
    """
    root = repo_root or _repo_root()
    errors: list[str] = []
    warnings: list[str] = []

    bugs_dir = root / "docs" / "bugs"
    if not bugs_dir.is_dir():
        return (["docs/bugs/ does not exist"], warnings)

    schema_path = root / "docs" / "quality" / "bug-packet.schema.json"
    validator = None
    if schema_path.is_file():
        try:
            from jsonschema import Draft202012Validator

            schema = json.loads(schema_path.read_text(encoding="utf-8"))
            validator = Draft202012Validator(schema)
        except ImportError:
            warnings.append("jsonschema not installed; structural checks only")
        except Exception as exc:  # noqa: BLE001 - report any schema-load failure, don't crash
            errors.append(f"bug-packet.schema.json unusable: {exc}")

    # A directory that looks like a packet (BUG-*) but violates the strict naming contract
    # must ERROR, not be silently skipped — otherwise it is counted as a packet yet never
    # validated (a mis-named real packet would slip through the gate reported as "validated").
    for p in sorted(bugs_dir.iterdir()):
        if p.is_dir() and p.name.startswith("BUG-") and not re.match(_BUG_DIR_RE, p.name):
            errors.append(
                f"{p.name} has an invalid packet name "
                "(must match BUG-NNNN-<lowercase-slug>); rename it or it is never validated"
            )

    packets = sorted(
        p.name for p in bugs_dir.iterdir() if p.is_dir() and re.match(_BUG_DIR_RE, p.name)
    )

    for name in packets:
        pdir = bugs_dir / name
        for fname in _REQUIRED_FILES:
            fpath = pdir / fname
            if not fpath.is_file():
                errors.append(f"{name} is missing {fname}")
            elif fname.endswith(".md") and not fpath.read_text(encoding="utf-8").strip():
                errors.append(f"{name}/{fname} is empty")

        meta_path = pdir / "bug.json"
        if not meta_path.is_file():
            continue
        try:
            meta = json.loads(meta_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            errors.append(f"{name}/bug.json is not valid JSON: {exc}")
            continue

        if validator is not None:
            for err in sorted(validator.iter_errors(meta), key=lambda e: e.path):
                loc = "/".join(str(p) for p in err.path) or "(root)"
                errors.append(f"{name}/bug.json schema: {loc}: {err.message}")

        if meta.get("id") != name:
            errors.append(f"{name}/bug.json id must match the directory name")

        status = meta.get("status")
        repro_ev = (meta.get("reproduction") or {}).get("evidence") or []
        gate_cmds = (meta.get("regression_gate") or {}).get("commands") or []

        # Cross-field lifecycle rules (the BOS evidence standard).
        if status in ("fixed", "verified") and not repro_ev:
            errors.append(f"{name} cannot be {status} without reproduction evidence")
        if status == "verified" and not gate_cmds:
            errors.append(f"{name} cannot be verified without regression-gate commands")

        # An AI-behavior bug needs an EVAL-CASE regression gate (not just a deterministic test).
        if meta.get("bug_class") == "ai-behavior" and status == "verified":
            if not any("ai_eval" in c for c in gate_cmds):
                errors.append(
                    f"{name} is an ai-behavior bug verified without an ai_eval regression gate "
                    "(BUG_OPERATING_SYSTEM.md — eval-gated closure)"
                )

        # Advisory: closure should be anchored to proof (a commit or a signed-ledger event).
        if status == "verified" and not meta.get("closure_attestation"):
            warnings.append(
                f"{name} is verified without a closure_attestation "
                "(bind closure to the fix commit_sha + gate run, or a signed run-ledger event "
                "— BUG_OPERATING_SYSTEM.md)"
            )

    return (errors, warnings)


def main() -> int:
    root = _repo_root()
    errors, warnings = validate_bug_packets(root)
    for w in warnings:
        print(f"[bug-packets] WARN: {w}")
    for e in errors:
        print(f"[bug-packets] ERROR: {e}", file=sys.stderr)
    bugs_dir = root / "docs" / "bugs"
    n = (
        sum(1 for p in bugs_dir.iterdir() if p.is_dir() and re.match(_BUG_DIR_RE, p.name))
        if bugs_dir.is_dir()
        else 0
    )
    if errors:
        print(f"[bug-packets] {len(errors)} error(s) across {n} packet(s)", file=sys.stderr)
        return 1
    print(f"[bug-packets] validated {n} bug packet(s); {len(warnings)} warning(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
