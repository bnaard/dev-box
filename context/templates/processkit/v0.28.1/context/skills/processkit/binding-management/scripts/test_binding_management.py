#!/usr/bin/env python3
"""Regression tests for binding-management MCP helpers."""
from __future__ import annotations

import importlib.util
import os
import shutil
import tempfile
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[5]
SERVER_PATH = (
    REPO_ROOT
    / "context"
    / "skills"
    / "processkit"
    / "binding-management"
    / "mcp"
    / "server.py"
)


def _load_server():
    spec = importlib.util.spec_from_file_location("binding_server", SERVER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load {SERVER_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _prepare_project(root: Path) -> None:
    (root / "AGENTS.md").write_text("# test project\n", encoding="utf-8")
    shutil.copytree(REPO_ROOT / "context" / "schemas", root / "context" / "schemas")
    shutil.copytree(
        REPO_ROOT / "context" / "state-machines",
        root / "context" / "state-machines",
    )


def test_update_binding_repairs_time_window_conditions() -> None:
    server = _load_server()
    old_cwd = Path.cwd()
    with tempfile.TemporaryDirectory() as tmpdir:
        project = Path(tmpdir)
        _prepare_project(project)
        os.chdir(project)
        try:
            created = server.create_binding(
                type="time-window",
                subject="BACK-test-process",
                target="ACTOR-test-agent",
                conditions={},
            )
            result = server.update_binding(
                created["id"],
                conditions={"recurrence_rule": "ART-test-schedule-rule"},
            )
        finally:
            os.chdir(old_cwd)

        assert "error" not in result, result
        assert result["updated"] == ["conditions"]
        assert result["spec"]["conditions"]["recurrence_rule"] == (
            "ART-test-schedule-rule"
        )

        data = yaml.safe_load(
            Path(created["path"]).read_text(encoding="utf-8").split("---", 2)[1]
        )
        assert data["spec"]["conditions"]["recurrence_rule"] == (
            "ART-test-schedule-rule"
        )


if __name__ == "__main__":
    test_update_binding_repairs_time_window_conditions()
