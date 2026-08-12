"""Regression tests for database-backed MCP result-size controls."""
from __future__ import annotations

import sqlite3
import sys
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(_REPO_ROOT / "context" / "skills" / "_lib"))

from processkit import index  # noqa: E402


def _db() -> sqlite3.Connection:
    db = sqlite3.connect(":memory:")
    db.row_factory = sqlite3.Row
    db.executescript(index.SCHEMA_DDL)
    return db


def test_query_entities_clamps_large_limit():
    db = _db()
    try:
        for i in range(520):
            db.execute(
                """
                INSERT INTO entities (
                    id, kind, api_version, path, created, spec_json
                ) VALUES (?, 'WorkItem', 'processkit.projectious.work/v2',
                    ?, ?, '{}')
                """,
                (
                    f"BACK-test-{i:04d}",
                    f"/tmp/{i}.md",
                    f"2026-01-01T00:{i:04d}",
                ),
            )
        rows = index.query_entities(db, kind="WorkItem", limit=10_000)
        assert len(rows) == index.MAX_ENTITY_RESULT_LIMIT
    finally:
        db.close()


def test_unfiltered_query_events_is_tightly_capped_and_truncated():
    db = _db()
    try:
        long_summary = "x" * (index.EVENT_SUMMARY_PREVIEW_CHARS + 20)
        for i in range(130):
            db.execute(
                """
                INSERT INTO events (
                    id, timestamp, event_type, actor, summary, path
                ) VALUES (?, ?, 'test.event', 'system', ?, ?)
                """,
                (
                    f"LOG-test-{i:04d}",
                    f"2026-01-01T00:{i:04d}",
                    long_summary,
                    f"/tmp/{i}.md",
                ),
            )
        rows = index.query_events(db, limit=10_000)
        assert len(rows) == index.MAX_UNFILTERED_EVENT_RESULT_LIMIT
        assert len(rows[0]["summary"]) <= index.EVENT_SUMMARY_PREVIEW_CHARS
        assert rows[0]["summary"].endswith("...")
    finally:
        db.close()


def test_filtered_query_events_allows_larger_bounded_window():
    db = _db()
    try:
        for i in range(230):
            db.execute(
                """
                INSERT INTO events (
                    id, timestamp, event_type, actor, summary, path
                ) VALUES (?, ?, 'release.audit', 'system', 'summary', ?)
                """,
                (
                    f"LOG-release-{i:04d}",
                    f"2026-01-01T00:{i:04d}",
                    f"/tmp/{i}.md",
                ),
            )
        rows = index.query_events(
            db,
            event_type="release.audit",
            limit=10_000,
        )
        assert len(rows) == index.MAX_EVENT_RESULT_LIMIT
    finally:
        db.close()


def test_list_errors_is_bounded():
    db = _db()
    try:
        for i in range(650):
            db.execute(
                "INSERT INTO errors (path, message) VALUES (?, 'bad')",
                (f"/tmp/{i:04d}.md",),
            )
        rows = index.list_errors(db, limit=10_000)
        assert len(rows) == index.MAX_ERROR_RESULT_LIMIT
    finally:
        db.close()
