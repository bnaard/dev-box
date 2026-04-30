"""Tests for context-archiving cold-tier archive and extraction."""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import yaml


_REPO_ROOT = Path(__file__).resolve().parents[5]
_SERVER = (
    _REPO_ROOT
    / "context"
    / "skills"
    / "processkit"
    / "context-archiving"
    / "mcp"
    / "server.py"
)
_LIB = _REPO_ROOT / "context" / "skills" / "_lib"
sys.path.insert(0, str(_LIB))


def _load_server():
    spec = importlib.util.spec_from_file_location("context_archiving_server", _SERVER)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _write_entity(path: Path, *, entity_id: str, state: str, body: str = "") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    data = {
        "apiVersion": "processkit.projectious.work/v1",
        "kind": "WorkItem",
        "metadata": {
            "id": entity_id,
            "created": "2026-01-01T00:00:00+00:00",
        },
        "spec": {
            "title": f"Archived fixture {entity_id}",
            "state": state,
            "type": "task",
            "priority": "low",
            "description": "archive fixture",
        },
    }
    text = "---\n" + yaml.safe_dump(data, sort_keys=False) + "---\n" + body
    path.write_text(text, encoding="utf-8")


def _write_log_entry(path: Path, *, entity_id: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    data = {
        "apiVersion": "processkit.projectious.work/v1",
        "kind": "LogEntry",
        "metadata": {
            "id": entity_id,
            "created": "2026-01-01T00:00:00+00:00",
        },
        "spec": {
            "timestamp": "2026-01-01T00:00:00+00:00",
            "event_type": "workitem.transitioned",
            "actor": "tester",
            "subject": "BACK-old-done",
            "subject_kind": "WorkItem",
            "summary": "Moved old work item to done",
            "details": {"state": "done"},
        },
    }
    text = "---\n" + yaml.safe_dump(data, sort_keys=False) + "---\nlog body"
    path.write_text(text, encoding="utf-8")


def test_create_archive_removes_hot_file_and_keeps_index_row(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    (tmp_path / "AGENTS.md").write_text("# test\n", encoding="utf-8")
    hot = tmp_path / "context" / "workitems" / "BACK-old-done.md"
    active = tmp_path / "context" / "workitems" / "BACK-active.md"
    _write_entity(hot, entity_id="BACK-old-done", state="done", body="cold body")
    _write_entity(active, entity_id="BACK-active", state="in-progress")

    from processkit import index

    db = index.open_db()
    try:
        index.reindex(tmp_path, db)
    finally:
        db.close()

    server = _load_server()
    dry = server.create_archive(kind="WorkItem", state="done", dry_run=True)
    assert dry["candidate_count"] == 1

    out = server.create_archive(
        kind="WorkItem",
        state="done",
        older_than_days=30,
        dry_run=False,
    )

    assert out["ok"] is True
    assert out["archived"] == 1
    assert not hot.exists()
    assert active.exists()
    assert (tmp_path / out["archive_path"]).is_file()
    assert (tmp_path / out["manifest_path"]).is_file()

    db = index.open_db()
    try:
        row = index.get_entity(db, "BACK-old-done")
    finally:
        db.close()

    assert row is not None
    assert row["state"] == "done"
    assert row["storage_location"].endswith("::context/workitems/BACK-old-done.md")

    payload = server.extract_archive_payload("BACK-old-done")
    assert payload["id"] == "BACK-old-done"
    assert "cold body" in payload["text"]


def test_create_archive_rejects_active_state(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    (tmp_path / "AGENTS.md").write_text("# test\n", encoding="utf-8")
    server = _load_server()

    out = server.create_archive(kind="WorkItem", state="in-progress")

    assert "active" in out["error"]


def test_archived_log_entry_remains_event_queryable(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    (tmp_path / "AGENTS.md").write_text("# test\n", encoding="utf-8")
    log_path = tmp_path / "context" / "logs" / "LOG-old.md"
    _write_log_entry(log_path, entity_id="LOG-old")

    from processkit import index

    db = index.open_db()
    try:
        index.reindex(tmp_path, db)
    finally:
        db.close()

    server = _load_server()
    out = server.create_archive(
        kind="LogEntry",
        state=None,
        older_than_days=30,
        dry_run=False,
    )

    assert out["ok"] is True
    assert out["archived"] == 1
    assert not log_path.exists()

    db = index.open_db()
    try:
        events = index.query_events(db, subject="BACK-old-done")
    finally:
        db.close()

    assert [event["id"] for event in events] == ["LOG-old"]
