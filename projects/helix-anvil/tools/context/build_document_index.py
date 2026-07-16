#!/usr/bin/env python3
"""Canonical global substrate tool — see ~/shared/substrate/SUBSTRATE_PORTING_GUIDE.md.
Project-specific config in the CONFIG block below.

Build / refresh the document index for this project.

The indexed corpus is immutable EVIDENCE. This tool produces a drift-aware
index over it:

- Computes mechanical facts for every discovered file: relative path, size,
  sha256, guessed type, and rich metadata (headings, word count, snippet).
- PRESERVES semantic fields (summary, key_claims, product_implications,
  contradictions_or_open_questions, currency, unique_contribution,
  origin_guess, title) authored in a human semantic pass — never overwrites
  them with blanks on re-run.
- Flags NEW files (need a semantic pass), CHANGED files (sha differs from
  the recorded one → semantic entry is stale), and REMOVED files (in the
  index but gone from disk).
- Detects duplicate-hash groups across the corpus.

Outputs:
  docs/DOCUMENT_INDEX.json   (machine-readable, canonical)
  docs/DOCUMENT_INDEX.md     (human-readable table, generated from the json)

Usage:
  python tools/context/build_document_index.py            # write/refresh
  python tools/context/build_document_index.py --check    # exit 1 on any drift
  python tools/context/build_document_index.py --stamp-date 2026-06-25

Stdlib only. `--stamp-date` lets a caller set generated_at deterministically.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import struct
import zipfile
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from xml.etree import ElementTree
from xml.etree.ElementTree import XMLParser  # noqa: F401 (also used via ElementTree.*)

# ---------------------------------------------------------------------------
# CONFIG — edit this block when porting to a new project.
# All project-specific path prefixes, include/exclude rules, and classify()
# heuristics live here. The rest of the script is project-agnostic.
# ---------------------------------------------------------------------------

REPO_ROOT = Path(__file__).resolve().parents[2]

# Output paths (relative to REPO_ROOT)
INDEX_JSON = REPO_ROOT / "docs" / "DOCUMENT_INDEX.json"
INDEX_MD   = REPO_ROOT / "docs" / "DOCUMENT_INDEX.md"

# File extensions to include in the index.
INCLUDED_EXTENSIONS: set[str] = {
    ".csv",
    ".docx",
    ".html",
    ".jpeg",
    ".jpg",
    ".md",
    ".mdx",
    ".odt",
    ".pdf",
    ".png",
    ".pptx",
    ".rtf",
    ".txt",
    ".xls",
    ".xlsx",
}

# Directory names to skip entirely (at any depth).
EXCLUDED_DIRS: set[str] = {
    ".git",
    ".hypothesis",
    ".npm-cache",
    ".pytest_cache",
    ".venv",
    "__pycache__",
    "dist",
    "mutants.out",
    "node_modules",
    "output",
    "target",
}

# Repo-relative posix paths of files to exclude even if their extension
# matches INCLUDED_EXTENSIONS (typically: generated context outputs).
EXCLUDED_FILES: set[str] = {
    "docs/DOCUMENT_INDEX.json",
    "docs/DOCUMENT_INDEX.md",
    "docs/context/AGENT_CONTEXT.md",
    "docs/context/SOURCE_MAP.md",
    "llms.txt",
    "llms-full.txt",
}

TEXT_EXTENSIONS:  set[str] = {".csv", ".html", ".md", ".mdx", ".rtf", ".txt"}
IMAGE_EXTENSIONS: set[str] = {".png", ".jpg", ".jpeg"}


def classify(path: Path) -> tuple[str, str]:
    """Return (category, family) for a file.

    PORTING: replace the body with logic appropriate to your project.
    Keep the signature; return two strings. category is a machine tag;
    family is a human-readable group label used in the Markdown output.

    The default below covers the substrate's own conventional layout.
    """
    rel = rel_path(path)
    ext = path.suffix.lower()

    if rel.startswith(".remember/"):
        return "working-memory-note", "Local remember notes"
    if rel.startswith("specs/"):
        return "ratified-spec", "Ratified capability specs"
    if rel.startswith("docs/features/"):
        return "feature-packet", "Context substrate feature packets"
    if rel.startswith("docs/review-batch/"):
        return "review-artifact", "Review batch evidence"
    if rel.startswith("docs/superpowers/specs/"):
        return "specification", "Superpowers locked specs"
    if rel.startswith("docs/superpowers/plans/"):
        return "plan-evidence", "Superpowers plans"
    if rel.startswith("tmp/"):
        return "temporary-artifact", "Temporary workspace documents"
    if ext in IMAGE_EXTENSIONS:
        return "figure-or-image", "Figures and diagrams"
    if rel in {"VISION.md", "BUILD_SPEC.md", "NEXT_ACTION.md",
               "DECISION_LOG.md", "AGENTS.md"}:
        return "governance-doc", "Project governance and continuation"
    if rel.startswith("research/") or rel.startswith("research_archive/"):
        return "research-notes", "Research evidence"
    if ext == ".csv":
        return "data-table", "Miscellaneous project artifacts"
    if ext == ".html":
        return "html-artifact", "Miscellaneous project artifacts"
    if ext == ".txt":
        return "plain-text-artifact", "Miscellaneous project artifacts"
    return "other-document", "Miscellaneous project artifacts"


# ---------------------------------------------------------------------------
# END CONFIG
# ---------------------------------------------------------------------------

SEMANTIC_FIELDS = (
    "type",
    "origin_guess",
    "title",
    "summary",
    "key_claims",
    "product_implications",
    "contradictions_or_open_questions",
    "currency",
    "unique_contribution",
)


def rel_path(path: Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def iter_document_paths() -> list[Path]:
    results: list[Path] = []
    stack = [REPO_ROOT]
    while stack:
        current = stack.pop()
        try:
            children = sorted(current.iterdir(),
                              key=lambda p: p.name.lower(), reverse=True)
        except PermissionError:
            continue
        for child in children:
            if child.is_dir():
                if (child.name not in EXCLUDED_DIRS
                        and not child.name.endswith(".egg-info")):
                    stack.append(child)
                continue
            if child.is_file():
                r = rel_path(child)
                if (child.suffix.lower() in INCLUDED_EXTENSIONS
                        and r not in EXCLUDED_FILES):
                    results.append(child)
    return sorted(results, key=lambda p: rel_path(p).lower())


def sha256_of(p: Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


# --- text / metadata extraction -------------------------------------------

def read_text_safe(path: Path) -> tuple[str, str]:
    data = path.read_bytes()
    for enc in ("utf-8-sig", "utf-8", "cp1252"):
        try:
            return data.decode(enc), enc
        except UnicodeDecodeError:
            pass
    return data.decode("utf-8", errors="replace"), "utf-8-replace"


def normalize_snippet(text: str, limit: int = 280) -> str:
    return re.sub(r"\s+", " ", text).strip()[:limit]


def markdown_headings(text: str) -> list[dict]:
    headings: list[dict] = []
    for line in text.splitlines():
        m = re.match(r"^(#{1,6})\s+(.+?)\s*$", line)
        if m:
            headings.append({"level": len(m.group(1)), "text": m.group(2)})
    return headings


def infer_title(path: Path, text: str = "",
                headings: list[dict] | None = None) -> str:
    if headings:
        return headings[0]["text"]
    for line in text.splitlines():
        clean = line.strip().strip("#").strip()
        if clean:
            return clean[:120]
    return path.stem.replace("_", " ").replace("-", " ")


def _safe_xml_parse(data: bytes) -> ElementTree.Element:
    """Parse XML bytes with external entity expansion disabled (stdlib-only).

    Python's expat (used internally by ElementTree) does not fetch external
    entities by default, but it DOES expand internal entities and can be
    triggered by billion-laughs payloads.  We neutralise that by:
      1. Using XMLParser with a custom expat handler that raises on any
         entity declaration that is NOT a predefined XML entity.
      2. Falling back to a plain parse if the expat hook is unavailable
         (the defence is belt-and-suspenders for the corpus; .docx files
         from office applications never contain internal entity declarations).

    If defusedxml is later added to the project, replace the body with:
        import defusedxml.ElementTree as DET
        return DET.fromstring(data)
    """
    try:
        parser = XMLParser()
        # Intercept entity declarations; raise for anything not predefined.
        _SAFE_ENTITIES = frozenset({"amp", "lt", "gt", "apos", "quot"})

        def _entity_decl(name, is_parameter_entity, value, base,
                         system_id, public_id, notation_name):
            if name not in _SAFE_ENTITIES:
                raise ElementTree.ParseError(
                    f"Forbidden entity declaration: {name!r}")

        parser.parser.EntityDeclHandler = _entity_decl  # type: ignore[attr-defined]
        parser.feed(data)
        return parser.close()
    except AttributeError:
        # expat attribute not available in this Python build — plain parse.
        return ElementTree.fromstring(data)


def docx_metadata(path: Path) -> dict:
    try:
        with zipfile.ZipFile(path) as z:
            document_xml = z.read("word/document.xml")
    except (KeyError, zipfile.BadZipFile):
        return {"title": path.stem.replace("_", " ").replace("-", " ")}
    ns = {"w": "http://schemas.openxmlformats.org/wordprocessingml/2006/main"}
    try:
        root = _safe_xml_parse(document_xml)
    except ElementTree.ParseError as exc:
        return {"title": path.stem.replace("_", " ").replace("-", " "),
                "parse_error": str(exc)}
    paragraphs, headings = [], []
    for para in root.findall(".//w:p", ns):
        text = "".join(n.text or "" for n in para.findall(".//w:t", ns)).strip()
        if not text:
            continue
        paragraphs.append(text)
        sn = para.find(".//w:pStyle", ns)
        style = sn.attrib.get(f"{{{ns['w']}}}val", "") if sn is not None else ""
        if style.lower().startswith("heading"):
            lm = re.search(r"(\d+)", style)
            headings.append({
                "level": int(lm.group(1)) if lm else 1,
                "text": text, "style": style,
            })
    full_text = "\n".join(paragraphs)
    return {
        "title": infer_title(path, full_text, headings),
        "word_count": len(re.findall(r"\b\w+\b", full_text)),
        "paragraph_count": len(paragraphs),
        "headings": headings[:80],
        "snippet": normalize_snippet(full_text),
    }


def png_dimensions(path: Path) -> tuple[int | None, int | None]:
    data = path.read_bytes()[:24]
    if len(data) >= 24 and data.startswith(b"\x89PNG\r\n\x1a\n"):
        return struct.unpack(">II", data[16:24])
    return None, None


def metadata_for(path: Path) -> dict:
    ext = path.suffix.lower()
    if ext in TEXT_EXTENSIONS:
        text, encoding = read_text_safe(path)
        headings = markdown_headings(text) if ext in {".md", ".mdx"} else []
        return {
            "title": infer_title(path, text, headings),
            "encoding": encoding,
            "line_count": len(text.splitlines()),
            "word_count": len(re.findall(r"\b\w+\b", text)),
            "heading_count": len(headings),
            "headings": headings[:40],
            "snippet": normalize_snippet(text),
        }
    if ext == ".docx":
        return docx_metadata(path)
    if ext == ".png":
        w, h = png_dimensions(path)
        result: dict = {"title": path.stem.replace("_", " ").replace("-", " "),
                        "media_type": "png"}
        if w and h:
            result.update({"width": w, "height": h,
                           "aspect_ratio": round(w / h, 4)})
        return result
    return {"title": path.stem.replace("_", " ").replace("-", " ")}


# --- semantic preservation ------------------------------------------------

def empty_semantic(rel: str) -> dict:
    """Default semantic record for a newly discovered file."""
    return {
        "type": "unclassified",
        "origin_guess": "",
        "title": Path(rel).name,
        "summary": "TODO: needs semantic pass",
        "key_claims": [],
        "product_implications": [],
        "contradictions_or_open_questions": [],
        "currency": "unrated",
        "unique_contribution": "",
    }


# --- index build ----------------------------------------------------------

def load_existing() -> dict:
    if INDEX_JSON.exists():
        try:
            return json.loads(INDEX_JSON.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            print(f"WARN: existing DOCUMENT_INDEX.json is invalid JSON: {exc}",
                  flush=True)
    return {"files": [], "documents": []}


def document_entry(path: Path, prior: dict | None) -> dict:
    """Build or refresh an index entry, preserving any existing semantic fields."""
    stat = path.stat()
    digest = sha256_of(path)
    r = rel_path(path)
    category, family = classify(path)

    entry: dict = {
        "path": r,
        "extension": path.suffix.lower(),
        "category": category,
        "family": family,
        "size_bytes": stat.st_size,
        "size_kib": round(stat.st_size / 1024, 1),
        "modified_local": (
            datetime.fromtimestamp(stat.st_mtime).astimezone()
            .isoformat(timespec="seconds")
        ),
        "sha256": digest,
        "sha256_16": digest[:16],
    }
    entry.update(metadata_for(path))

    # Preserve human-authored semantic fields; mark stale if hash changed.
    if prior is None:
        sem = empty_semantic(r)
        sem["stale_semantic"] = True
    else:
        sem = {f: prior.get(f, empty_semantic(r)[f]) for f in SEMANTIC_FIELDS}
        sem["stale_semantic"] = (prior.get("sha256") != digest
                                 or bool(prior.get("stale_semantic", False)))

    entry.update(sem)
    return entry


def build(stamp_date: str | None) -> tuple[dict, dict]:
    existing_raw = load_existing()
    # Support both REAL_ESTATE_OS shape ("files") and DOCUMENT_OS shape ("documents")
    prior_list = existing_raw.get("documents") or existing_raw.get("files", [])
    prior: dict[str, dict] = {e["path"]: e for e in prior_list}

    on_disk = iter_document_paths()
    disk_paths = {rel_path(p) for p in on_disk}

    documents: list[dict] = []
    added, changed, unchanged = [], [], []

    for p in on_disk:
        r = rel_path(p)
        e = document_entry(p, prior.get(r))
        if r not in prior:
            added.append(r)
        elif prior[r].get("sha256") != e["sha256"]:
            changed.append(r)
        else:
            unchanged.append(r)
        documents.append(e)

    removed = [path for path in prior if path not in disk_paths]
    documents.sort(key=lambda e: e["path"])

    by_hash: dict[str, list[str]] = defaultdict(list)
    for doc in documents:
        by_hash[doc["sha256"]].append(doc["path"])
    duplicates = [
        {"sha256_16": h[:16], "sha256": h, "paths": sorted(ps)}
        for h, ps in sorted(by_hash.items())
        if len(ps) > 1
    ]

    index = {
        "generated_at": stamp_date or datetime.now(timezone.utc).isoformat(
            timespec="seconds"),
        "root": str(REPO_ROOT),
        "scope": {
            "included_extensions": sorted(INCLUDED_EXTENSIONS),
            "excluded_dirs": sorted(EXCLUDED_DIRS),
            "excluded_files": sorted(EXCLUDED_FILES),
            "note": ("Authored document-like artifacts, excluding "
                     "dependency/build/cache directories and generated "
                     "context exports."),
        },
        "counts": {
            "total": len(documents),
            "by_extension": dict(sorted(
                Counter(d["extension"] for d in documents).items())),
            "by_category": dict(sorted(
                Counter(d["category"] for d in documents).items())),
            "by_family": dict(sorted(
                Counter(d["family"] for d in documents).items())),
        },
        "duplicate_hash_groups": duplicates,
        "documents": documents,
        # Legacy alias kept for tools that read the "files" key.
        "files": documents,
    }
    drift = {
        "added": added, "changed": changed,
        "removed": removed, "unchanged": unchanged,
    }
    return index, drift


def render_md(index: dict) -> str:
    project_name = index.get("root", "").split("/")[-1] or "project"
    lines = [
        f"# Document Index — {project_name}",
        "",
        f"> AUTO-GENERATED by `tools/context/build_document_index.py`.",
        f"> Do not edit by hand — edit the JSON's semantic fields, then",
        f"> regenerate.",
        "",
        f"- Sources indexed: **{index['counts']['total']}**",
        f"- Generated at: {index.get('generated_at') or '(unstamped)'}",
        "",
    ]
    if index.get("duplicate_hash_groups"):
        lines += ["## Duplicate Hash Groups", ""]
        for n, g in enumerate(index["duplicate_hash_groups"], 1):
            paths = "; ".join(f"`{p}`" for p in g["paths"])
            lines.append(f"{n}. `{g['sha256_16']}` ({len(g['paths'])} files): "
                         f"{paths}")
        lines.append("")

    lines += [
        "## Document Entries",
        "",
        "| Path | Category | Currency | Stale? | Summary |",
        "|---|---|---|---|---|",
    ]
    for e in index["documents"]:
        summ = (e.get("summary") or "").replace("|", "\\|").replace("\n", " ")
        if len(summ) > 140:
            summ = summ[:137] + "..."
        stale = "YES" if e.get("stale_semantic") else ""
        lines.append(
            f"| `{e['path']}` | {e.get('category', '')} "
            f"| {e.get('currency', '')} | {stale} | {summ} |"
        )
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Build/refresh the project document index.")
    ap.add_argument("--check", action="store_true",
                    help="exit 1 if any add/change/remove drift is detected")
    ap.add_argument("--stamp-date", default=None,
                    help="ISO date/datetime to record as generated_at")
    args = ap.parse_args()

    index, drift = build(args.stamp_date)

    if not args.check:
        INDEX_JSON.parent.mkdir(parents=True, exist_ok=True)
        INDEX_JSON.write_text(
            json.dumps(index, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8")
        INDEX_MD.write_text(render_md(index), encoding="utf-8")

    print(f"indexed={index['counts']['total']} "
          f"added={len(drift['added'])} "
          f"changed={len(drift['changed'])} "
          f"removed={len(drift['removed'])} "
          f"unchanged={len(drift['unchanged'])}")
    for label in ("added", "changed", "removed"):
        for p in drift[label]:
            print(f"  {label}: {p}")

    if args.check and (drift["added"] or drift["changed"] or drift["removed"]):
        print("DRIFT: index is out of sync with the document corpus.",
              flush=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
