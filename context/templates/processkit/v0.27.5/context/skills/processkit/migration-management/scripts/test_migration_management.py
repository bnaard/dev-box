"""Focused tests for migration-management MCP behavior.

Run with:

    uv run --with mcp --with pyyaml --with jsonschema \
      context/skills/processkit/migration-management/scripts/test_migration_management.py
"""
from __future__ import annotations

import importlib.util
import os
import tempfile
from pathlib import Path


SKILL_ROOT = Path(__file__).resolve().parents[1]
SERVER_PATH = SKILL_ROOT / "mcp" / "server.py"


def _load_server():
    spec = importlib.util.spec_from_file_location(
        "processkit_migration_management_server",
        SERVER_PATH,
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _write_migration(root: Path, bucket: str, mid: str, state: str) -> None:
    path = root / "context" / "migrations" / bucket / f"{mid}.md"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        f"""---
apiVersion: processkit.projectious.work/v2
kind: Migration
metadata:
  id: {mid}
  created: 2026-05-12T00:00:00Z
spec:
  source: test
  state: {state}
  from_version: "1"
  to_version: "2"
  summary: "{state} migration"
---
# {mid}
""",
        encoding="utf-8",
    )


def test_list_migrations_default_returns_only_active_states() -> None:
    server = _load_server()
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "AGENTS.md").write_text("# Test repo\n", encoding="utf-8")
        _write_migration(root, "pending", "MIG-PENDING", "pending")
        _write_migration(root, "in-progress", "MIG-INPROGRESS", "in-progress")
        _write_migration(root, "applied", "MIG-APPLIED", "applied")
        _write_migration(root, "applied", "MIG-REJECTED", "rejected")

        old_cwd = Path.cwd()
        try:
            os.chdir(root)
            default_ids = {m["id"] for m in server.list_migrations()}
            applied_ids = {
                m["id"] for m in server.list_migrations(state="applied")
            }
            rejected_ids = {
                m["id"] for m in server.list_migrations(state="rejected")
            }
        finally:
            os.chdir(old_cwd)

    assert default_ids == {"MIG-PENDING", "MIG-INPROGRESS"}
    assert applied_ids == {"MIG-APPLIED"}
    assert rejected_ids == {"MIG-REJECTED"}


def test_list_migrations_ignores_non_entity_briefings() -> None:
    server = _load_server()
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "AGENTS.md").write_text("# Test repo\n", encoding="utf-8")
        pending = root / "context" / "migrations" / "pending"
        pending.mkdir(parents=True, exist_ok=True)
        (pending / "disabled-harness-state.md").write_text(
            "# Disabled harness state\n\nHuman briefing only.\n",
            encoding="utf-8",
        )

        old_cwd = Path.cwd()
        try:
            os.chdir(root)
            migrations = server.list_migrations(state="pending")
        finally:
            os.chdir(old_cwd)

    assert migrations == []


def test_list_migrations_empty_pending_returns_empty_list() -> None:
    server = _load_server()
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "AGENTS.md").write_text("# Test repo\n", encoding="utf-8")
        (root / "context" / "migrations" / "pending").mkdir(
            parents=True,
            exist_ok=True,
        )

        old_cwd = Path.cwd()
        try:
            os.chdir(root)
            migrations = server.list_migrations(state="pending")
        finally:
            os.chdir(old_cwd)

    assert migrations == []


def test_draft_migration_creates_pending_schema_valid_entity() -> None:
    server = _load_server()
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "AGENTS.md").write_text("# Test repo\n", encoding="utf-8")
        old_cwd = Path.cwd()
        try:
            os.chdir(root)
            result = server.draft_migration(
                kind="data-fix",
                summary="Normalize legacy Binding filenames",
                affected_files=[{
                    "path": "context/bindings/BIND-legacy.md",
                    "classification": "changed-locally-only",
                }],
                related_decisions=["DEC-test"],
            )
        finally:
            os.chdir(old_cwd)

        assert result["state"] == "pending"
        path = Path(result["path"])
        assert path.is_file()
        text = path.read_text(encoding="utf-8")
        assert "kind: data-fix" in text
        assert "state: pending" in text
        assert "related_decisions:" in text


if __name__ == "__main__":
    test_list_migrations_default_returns_only_active_states()
    test_list_migrations_ignores_non_entity_briefings()
    test_list_migrations_empty_pending_returns_empty_list()
    test_draft_migration_creates_pending_schema_valid_entity()
    print("All tests passed.")
