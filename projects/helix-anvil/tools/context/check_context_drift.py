#!/usr/bin/env python3
"""Canonical global substrate tool — see ~/shared/substrate/SUBSTRATE_PORTING_GUIDE.md.
Project-specific config in the CONFIG block below.

Master substrate-integrity gate (Tier-0, token-free).

Checks performed (any failure → non-zero exit):
  1. PROJECT_STATE.json validates against schemas/project_state.schema.json
     (full validation if `jsonschema` is installed; minimal structural check
     otherwise — noted in output).
  2. Every canonical_docs / historical_docs path exists on disk.
  3. Every active_feature_packets entry has its directory and all 5 required
     files; directory name equals the declared packet id (the full NNN-slug);
     status.json has all required keys and a valid status value.
  4. next_product_slice.packet_id is registered in active_feature_packets
     and its docs/features/<id>-* directory exists.
  5. NEXT_ACTION.md exists (load-bearing continuation pointer).
  6. Document index is in sync with the corpus (build_document_index --check).
  7. Context pack is fresh vs its inputs (build_context_pack --check).
  8. Ratified specs/ layer (substrate v2.2): every declared capability spec in
     PROJECT_STATE.json.ratified_specs exists and is well-formed (## Requirements
     / ### Requirement: / #### Scenario:); 'done' packets have their behavior-spec
     delta promoted into a registered capability (WARN on gaps; no-op pre-v2.2).

Usage:
  python tools/context/check_context_drift.py            # all checks
  python tools/context/check_context_drift.py --schema   # only check #1
  python tools/context/check_context_drift.py --no-subtools  # skip 6+7

Stdlib only (jsonschema optional for check #1).
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# CONFIG — edit this block when porting to a new project.
# ---------------------------------------------------------------------------

HERE      = Path(__file__).resolve().parent
REPO_ROOT = HERE.parents[1]

# Machine source of truth
STATE  = REPO_ROOT / "PROJECT_STATE.json"
# JSON Schema for PROJECT_STATE.json
SCHEMA = REPO_ROOT / "schemas" / "project_state.schema.json"

# Directory that contains feature packet subdirectories.
PACKETS_DIR = REPO_ROOT / "docs" / "features"

# The 5 files required inside every feature packet directory.
PACKET_FILES = (
    "requirements.md",
    "design.md",
    "tasks.md",
    "acceptance.md",
    "status.json",
)

# Valid values for status.json's "status" key.
VALID_PACKET_STATUSES = {
    "template",
    "planned",
    "active",
    "review-ready",
    "blocked",
    "done",
    "superseded",
    # PORTING: add project-specific statuses here
}

# Required top-level keys in status.json (checked when jsonschema absent).
STATUS_JSON_REQUIRED_KEYS = (
    "id",
    "status",
    "updated_at",
    "owner_model_tier",
    "current_step",
    "allowed_edit_paths",
    "forbidden_edit_paths",
    "verification_commands",
    "review_status",
)

# ---------------------------------------------------------------------------
# END CONFIG
# ---------------------------------------------------------------------------


def fail(msg: str, errors: list[str]) -> None:
    errors.append(msg)
    print(f"  FAIL: {msg}", file=sys.stderr)


def ok(msg: str) -> None:
    print(f"  ok: {msg}")


def warn(msg: str) -> None:
    print(f"  warn: {msg}")


# ---------------------------------------------------------------------------
# Check 1 — schema validation
# ---------------------------------------------------------------------------

def check_schema(errors: list[str]) -> None:
    if not STATE.exists():
        fail("PROJECT_STATE.json is missing", errors)
        return
    try:
        state = json.loads(STATE.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"PROJECT_STATE.json is invalid JSON: {exc}", errors)
        return
    if not SCHEMA.exists():
        fail("schemas/project_state.schema.json is missing", errors)
        return
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    try:
        import jsonschema  # type: ignore
        try:
            jsonschema.validate(instance=state, schema=schema)
            ok("PROJECT_STATE.json validates against schema (jsonschema)")
        except jsonschema.ValidationError as exc:
            path_str = "/".join(str(p) for p in exc.absolute_path)
            fail(f"schema validation: {exc.message} at {path_str!r}", errors)
    except ImportError:
        # Minimal fallback: check required keys only.
        missing = [k for k in schema.get("required", []) if k not in state]
        if missing:
            fail(f"PROJECT_STATE.json missing required keys: {missing}", errors)
        else:
            ok("PROJECT_STATE.json has all required top-level keys "
               "(install `jsonschema` for full validation)")


# ---------------------------------------------------------------------------
# Check 2 — canonical + historical doc paths exist
# ---------------------------------------------------------------------------

def check_docs_exist(errors: list[str]) -> None:
    if not STATE.exists():
        return
    state = json.loads(STATE.read_text(encoding="utf-8"))
    # Support both REAL_ESTATE_OS ("historical_docs") and
    # DOCUMENT_OS ("historical_or_superseded_docs") naming conventions.
    hist_key = ("historical_or_superseded_docs"
                if "historical_or_superseded_docs" in state
                else "historical_docs")
    for group in ("canonical_docs", hist_key):
        for d in state.get(group, []):
            p = REPO_ROOT / d["path"]
            if not p.exists():
                fail(f"{group} path does not exist: {d['path']}", errors)
    ok("canonical_docs / historical_docs paths checked")


# ---------------------------------------------------------------------------
# Check 3 — active feature packets complete
# ---------------------------------------------------------------------------

def _packet_dir(pid: str) -> Path | None:
    """Canonical convention: the packet directory basename equals the id (the
    full NNN-slug, schema pattern ^[0-9]{3}-[a-z0-9-]+$). Exact match, not a
    `{pid}-*` prefix glob — the prefix form silently tolerated id=='000' with
    dir=='000-project-inception' (a schema-pattern violation)."""
    cand = PACKETS_DIR / pid
    return cand if cand.is_dir() else None


def check_packets(errors: list[str]) -> None:
    """Every registered packet must exist, be complete, and be id-named."""
    if not STATE.exists():
        return
    state = json.loads(STATE.read_text(encoding="utf-8"))
    for pkt in state.get("active_feature_packets", []):
        pid  = pkt.get("id", "")
        path = pkt.get("path", "")
        d    = REPO_ROOT / path
        if not d.exists():
            fail(f"feature packet path does not exist: {path}", errors)
            continue
        if d.name != pid:
            fail(f"packet dir '{d.name}' does not match declared id '{pid}' "
                 f"(dir basename must equal the full NNN-slug id)", errors)
        for fname in PACKET_FILES:
            if not (d / fname).exists():
                fail(f"packet {d.name} missing {fname}", errors)
        _check_status_json(d, errors)
    ok("active_feature_packets paths + files checked")


def _check_status_json(pkt_dir: Path, errors: list[str]) -> None:
    status_path = pkt_dir / "status.json"
    if not status_path.exists():
        return  # already flagged by PACKET_FILES loop above
    try:
        status = json.loads(status_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{pkt_dir.name}/status.json is invalid JSON: {exc}", errors)
        return
    for key in STATUS_JSON_REQUIRED_KEYS:
        if key not in status:
            fail(f"{pkt_dir.name}/status.json missing key: {key!r}", errors)
    if status.get("status") not in VALID_PACKET_STATUSES:
        fail(f"{pkt_dir.name}/status.json has unknown status "
             f"{status.get('status')!r}; expected one of "
             f"{sorted(VALID_PACKET_STATUSES)}", errors)


# ---------------------------------------------------------------------------
# Check 4 — next_product_slice registered and on disk
# ---------------------------------------------------------------------------

def check_next_slice(errors: list[str]) -> None:
    if not STATE.exists():
        return
    state = json.loads(STATE.read_text(encoding="utf-8"))
    pid = state.get("next_product_slice", {}).get("packet_id")
    if not pid:
        fail("next_product_slice.packet_id is empty", errors)
        return
    registered = {p.get("id") for p in state.get("active_feature_packets", [])}
    if pid not in registered:
        fail(f"next_product_slice.packet_id '{pid}' is not in "
             f"active_feature_packets {sorted(registered)}", errors)
    if _packet_dir(pid) is None:
        fail(f"next_product_slice '{pid}' has no {PACKETS_DIR.name}/{pid} "
             f"directory", errors)
    else:
        ok(f"next_product_slice -> packet {pid} present")


# ---------------------------------------------------------------------------
# Check 5 — NEXT_ACTION.md exists
# ---------------------------------------------------------------------------

def check_handoff(errors: list[str]) -> None:
    if not (REPO_ROOT / "NEXT_ACTION.md").exists():
        fail("NEXT_ACTION.md is missing (load-bearing continuation pointer)",
             errors)
    else:
        ok("NEXT_ACTION.md present")


# ---------------------------------------------------------------------------
# Check 8 — ratified specs/ layer (substrate v2.2 OpenSpec graft)
# ---------------------------------------------------------------------------

_PLACEHOLDER_TOKENS = ("[NEEDS CLARIFICATION", "<!-- INSTRUCTIONS")


def _is_placeholder_name(name: str) -> bool:
    """A heading name that is still an angle-bracket template placeholder."""
    return bool(re.match(r"^<.+>$", name.strip()))


def _spec_wellformed(text: str) -> list[str]:
    """Return structural problems in a ratified capability spec.

    Expected OpenSpec-native shape:
      ## Requirements
      ### Requirement: <name>      (>= 1; names are the promotion anchor → unique)
      #### Scenario: <name>        (>= 1 per Requirement)
    A block runs until the next '## ' or '### ' heading (a '#### Scenario'
    does NOT end it). Unfilled template placeholders are rejected — a ratified
    spec is CURRENT behavior, not a skeleton.
    """
    problems: list[str] = []
    lines = text.splitlines()
    if not any(re.match(r"^##\s+Requirements\s*$", ln) for ln in lines):
        problems.append("missing '## Requirements' section")
    req_idxs = [i for i, ln in enumerate(lines)
                if re.match(r"^###\s+Requirement:", ln)]
    if not req_idxs:
        problems.append("no '### Requirement:' blocks")
        return problems
    seen_names: dict[str, int] = {}
    for start in req_idxs:
        name = lines[start].split("Requirement:", 1)[1].strip() or "<unnamed>"
        seen_names[name] = seen_names.get(name, 0) + 1
        if _is_placeholder_name(name):
            problems.append(f"Requirement name {name!r} is an unfilled placeholder")
        end = len(lines)
        for j in range(start + 1, len(lines)):
            if re.match(r"^(##|###)\s+\S", lines[j]):
                end = j
                break
        block = lines[start + 1:end]
        scen = [ln for ln in block if re.match(r"^####\s+Scenario:", ln)]
        if not scen:
            problems.append(f"Requirement {name!r} has no '#### Scenario:' block")
        for sl in scen:
            sname = sl.split("Scenario:", 1)[1].strip()
            if _is_placeholder_name(sname):
                problems.append(f"Scenario name {sname!r} is an unfilled placeholder")
    # Requirement names are the promotion match anchor → must be unique.
    for nm, count in seen_names.items():
        if count > 1:
            problems.append(f"duplicate Requirement name {nm!r} ({count}x) — names "
                            f"are the promotion anchor and must be unique")
    # Reject specs that still carry template scaffolding.
    for tok in _PLACEHOLDER_TOKENS:
        if tok in text:
            problems.append(f"contains template marker {tok!r} — not a ratified "
                            f"spec (resolve before registering in ratified_specs)")
    return problems


def check_ratified_specs(errors: list[str]) -> None:
    """Validate the v2.2 ratified specs/ layer + promotion completeness.

    Backward-compatible: a no-op for projects with no ratified_specs and no
    'done' packets (greenfield / pre-v2.2). Declared spec paths are validated
    strictly (exist + confined to repo + well-formed → fail). Promotion: a
    'done' packet whose promotes_to capability is registered MUST appear in that
    spec's promoted_from (→ fail; catches stale/unpromoted specs); an
    unregistered capability is a WARN (migration-friendly).
    """
    if not STATE.exists():
        return
    state = json.loads(STATE.read_text(encoding="utf-8"))
    specs = state.get("ratified_specs", []) or []
    spec_caps = {s.get("capability") for s in specs}
    promoted_from = {s.get("capability"): set(s.get("promoted_from", []) or [])
                     for s in specs}
    repo_root = REPO_ROOT.resolve()

    # 1. Declared specs: confined to repo, exist, well-formed.
    for s in specs:
        rel = s.get("path", "")
        p = REPO_ROOT / rel
        try:
            rp = p.resolve()
        except OSError:
            fail(f"ratified_specs path cannot be resolved: {rel}", errors)
            continue
        if rp != repo_root and repo_root not in rp.parents:
            fail(f"ratified_specs path escapes the repo root: {rel}", errors)
            continue
        if not p.exists():
            fail(f"ratified_specs path does not exist: {rel}", errors)
            continue
        for prob in _spec_wellformed(p.read_text(encoding="utf-8")):
            fail(f"ratified spec {rel}: {prob}", errors)

    # 2. Promotion completeness.
    packets = state.get("active_feature_packets", [])
    done = [p for p in packets if p.get("status") == "done"]
    for p in done:
        pid = p.get("id")
        for cap in p.get("promotes_to", []) or []:
            if cap not in spec_caps:
                warn(f"packet {pid} is done and promotes_to '{cap}' but no "
                     f"ratified_specs entry exists — promote its behavior-spec "
                     f"delta into specs/{cap}/spec.md")
            elif pid not in promoted_from.get(cap, set()):
                fail(f"packet {pid} is done and promotes_to '{cap}' (registered) "
                     f"but ratified_specs['{cap}'].promoted_from omits '{pid}' — "
                     f"the delta was not promoted (stale spec)", errors)

    # 3. Shipped-behavior gate (WARN): behavior shipped but nothing ratified.
    if done and not specs:
        warn(f"{len(done)} packet(s) at status 'done' but ratified_specs is empty "
             f"— substrate v2.2 expects shipped behavior to be ratified into "
             f"specs/<capability>/spec.md (optional until first 'done').")

    if specs:
        ok(f"ratified specs layer checked ({len(specs)} capability spec(s))")


# ---------------------------------------------------------------------------
# Checks 6+7 — delegate to the other context tools
# ---------------------------------------------------------------------------

def run_subcheck(script_name: str, errors: list[str], label: str) -> None:
    script = HERE / script_name
    if not script.exists():
        warn(f"{script_name} not found alongside check_context_drift.py — "
             f"skipping {label} check")
        return
    res = subprocess.run(
        [sys.executable, str(script), "--check"],
        capture_output=True, text=True,
    )
    sys.stdout.write(res.stdout)
    if res.returncode != 0:
        sys.stderr.write(res.stderr)
        fail(f"{label} drift (see {script_name} output)", errors)
    else:
        ok(f"{label} in sync")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def check_bug_packets(errors: list[str]) -> None:
    """Tier-0: validate docs/bugs/BUG-* packets (Bug Operating System)."""
    import importlib.util  # noqa: PLC0415

    vp = REPO_ROOT / "tools" / "quality" / "validate_bug_packets.py"
    if not vp.exists():
        return
    try:
        spec = importlib.util.spec_from_file_location("validate_bug_packets", vp)
        mod = importlib.util.module_from_spec(spec)
        assert spec.loader is not None
        spec.loader.exec_module(mod)
        bug_errors, bug_warnings = mod.validate_bug_packets(REPO_ROOT)
    except Exception as exc:  # noqa: BLE001
        fail(f"bug-packet validation crashed: {exc}", errors)
        return
    for w in bug_warnings:
        warn(f"bug-packets: {w}")
    for e in bug_errors:
        fail(f"bug-packets: {e}", errors)
    if not bug_errors:
        ok(f"bug packets valid ({len(bug_warnings)} warning(s))")

def main() -> int:
    ap = argparse.ArgumentParser(
        description="Substrate integrity gate — validates PROJECT_STATE.json, "
                    "doc paths, feature packets, and context freshness.")
    ap.add_argument("--schema", action="store_true",
                    help="only validate PROJECT_STATE.json against the schema")
    ap.add_argument("--no-subtools", action="store_true",
                    help="skip build_document_index --check and "
                         "build_context_pack --check (useful in CI where "
                         "these run as separate steps)")
    args = ap.parse_args()

    errors: list[str] = []
    proj_name = "project"
    if STATE.exists():
        try:
            _s = json.loads(STATE.read_text(encoding="utf-8"))
            proj_name = _s.get("project", {}).get("name", proj_name)
        except Exception:
            pass

    print(f"== {proj_name} substrate drift check ==")
    print(f"   root: {REPO_ROOT}")

    check_schema(errors)

    if not args.schema:
        check_docs_exist(errors)
        check_packets(errors)
        check_next_slice(errors)
        check_handoff(errors)
        check_bug_packets(errors)
        check_ratified_specs(errors)

        if not args.no_subtools:
            run_subcheck("build_document_index.py", errors, "document index")
            run_subcheck("build_context_pack.py",   errors, "context pack")

    if errors:
        print(f"\nDRIFT/ERRORS: {len(errors)} problem(s). Substrate is not "
              f"clean.", file=sys.stderr)
        for i, e in enumerate(errors, 1):
            print(f"  {i}. {e}", file=sys.stderr)
        return 1

    print("\nAll substrate checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
