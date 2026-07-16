#!/usr/bin/env python3
"""tmp_path smoke tests for check_context_drift's packet-naming convention.

Canonical convention (schema + _template/status.json + porting guide all agree):
a feature packet's `id` is the full ``NNN-slug`` (schema pattern
``^[0-9]{3}-[a-z0-9-]+$``) and the packet DIRECTORY basename equals that id.
So the drift gate must accept ``dir.name == id`` exactly — not the looser
``dir.name.startswith(f"{id}-")`` prefix form, which silently tolerated the
schema-violating ``id == "000"`` / ``dir == "000-project-inception"`` style.

These tests exercise the real filesystem against throwaway tmp_path scaffolds
(per the production-state-isolation rule: never run against a real project).
The module-level path constants are monkeypatched to point at the scaffold.
"""
from __future__ import annotations

import json
from pathlib import Path

import pytest

import check_context_drift as drift


def _write_packet(features_dir: Path, dir_name: str, pkt_id: str,
                  status: str = "planned") -> None:
    """Create a fully valid 5-file packet so the ONLY thing that can fail
    check_packets is the dir-name-vs-id naming check under test."""
    d = features_dir / dir_name
    d.mkdir(parents=True)
    for fname in ("requirements.md", "design.md", "tasks.md", "acceptance.md"):
        (d / fname).write_text("# stub\n", encoding="utf-8")
    status_json = {
        "id": pkt_id,
        "status": status,
        "updated_at": "2026-06-29",
        "owner_model_tier": "sonnet",
        "current_step": "T-000",
        "allowed_edit_paths": [f"docs/features/{dir_name}/**"],
        "forbidden_edit_paths": ["state/**"],
        "verification_commands": [],
        "review_status": {"tier0": "pending", "tier1": "pending",
                          "tier2": "n/a", "tier3": "n/a"},
    }
    (d / "status.json").write_text(json.dumps(status_json), encoding="utf-8")


def _scaffold(tmp_path: Path, monkeypatch,
              packets: list[tuple[str, str, str]]) -> Path:
    """Build repo/docs/features with the given (dir_name, id, status) packets,
    write a minimal PROJECT_STATE.json, and point the module globals at it."""
    repo = tmp_path / "repo"
    features = repo / "docs" / "features"
    features.mkdir(parents=True)
    for dir_name, pkt_id, status in packets:
        _write_packet(features, dir_name, pkt_id, status)

    state = {
        "active_feature_packets": [
            {"id": pkt_id, "path": f"docs/features/{dir_name}",
             "title": f"packet {pkt_id}", "status": status}
            for dir_name, pkt_id, status in packets
        ],
    }
    (repo / "PROJECT_STATE.json").write_text(json.dumps(state), encoding="utf-8")

    monkeypatch.setattr(drift, "REPO_ROOT", repo)
    monkeypatch.setattr(drift, "STATE", repo / "PROJECT_STATE.json")
    monkeypatch.setattr(drift, "PACKETS_DIR", features)
    return repo


def test_check_packets_accepts_both_id_styles(tmp_path, monkeypatch):
    """Smoke test: dir name == id for both a multi-segment slug (DOCUMENT_OS
    style) and a short slug (classic). Both must pass with zero errors."""
    _scaffold(tmp_path, monkeypatch, [
        ("000-current-mvp-stabilization", "000-current-mvp-stabilization", "active"),
        ("001-context-substrate", "001-context-substrate", "planned"),
    ])
    errors: list[str] = []
    drift.check_packets(errors)
    assert errors == [], f"expected no errors for dir==id packets, got: {errors}"


def test_check_packets_flags_dir_id_mismatch(tmp_path, monkeypatch):
    """The schema-violating REAL_ESTATE_OS style (id '000', dir
    '000-project-inception') must now be flagged — dir name != id."""
    _scaffold(tmp_path, monkeypatch, [
        ("000-project-inception", "000", "planned"),
    ])
    errors: list[str] = []
    drift.check_packets(errors)
    assert any("000-project-inception" in e and "000" in e for e in errors), (
        f"expected a dir-name-vs-id mismatch error, got: {errors}")


def test_packet_dir_resolves_exact_name(tmp_path, monkeypatch):
    """_packet_dir resolves the directory whose name equals the id exactly."""
    _scaffold(tmp_path, monkeypatch, [
        ("000-current-mvp-stabilization", "000-current-mvp-stabilization", "active"),
    ])
    found = drift._packet_dir("000-current-mvp-stabilization")
    assert found is not None
    assert found.name == "000-current-mvp-stabilization"


def test_packet_dir_none_when_missing(tmp_path, monkeypatch):
    """_packet_dir returns None when no directory matches the id."""
    _scaffold(tmp_path, monkeypatch, [
        ("001-context-substrate", "001-context-substrate", "planned"),
    ])
    assert drift._packet_dir("999-does-not-exist") is None
