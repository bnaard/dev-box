"""Tests for the pk-doctor MCP wrapper."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path


_REPO_ROOT = Path(__file__).resolve().parents[5]
_SERVER = (
    _REPO_ROOT
    / "context"
    / "skills"
    / "processkit"
    / "pk-doctor"
    / "mcp"
    / "server.py"
)
_LIB = _REPO_ROOT / "context" / "skills" / "_lib"
sys.path.insert(0, str(_LIB))


def _load_server():
    spec = importlib.util.spec_from_file_location("pk_doctor_server", _SERVER)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_run_pk_doctor_forwards_explicit_confirmation(
    tmp_path: Path,
    monkeypatch,
):
    server = _load_server()
    captured: list[str] = []

    monkeypatch.setattr(server.paths, "find_project_root", lambda: tmp_path)

    def fake_run(command, **_kwargs):
        captured.extend(command)
        payload = {
            "findings": [],
            "totals": {"error": 0, "warn": 0, "info": 0},
            "action_totals": {"actionable": 0},
            "exit_code": 0,
        }
        return subprocess.CompletedProcess(command, 0, json.dumps(payload), "")

    monkeypatch.setattr(server.subprocess, "run", fake_run)
    tool = server.server._tool_manager._tools["run_pk_doctor"].fn

    result = tool(
        check="entity_storage_hygiene",
        fix="entity_storage_hygiene",
        yes=True,
    )

    assert result["exit_code"] == 0
    assert "--category=entity_storage_hygiene" in captured
    assert "--fix=entity_storage_hygiene" in captured
    assert "--yes" in captured
