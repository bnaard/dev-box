#!/usr/bin/env python3
"""Regression tests for gate-management MCP helpers."""
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
    / "gate-management"
    / "mcp"
    / "server.py"
)


def _load_server():
    spec = importlib.util.spec_from_file_location("gate_server", SERVER_PATH)
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


def test_update_gate_repairs_unobserved_gate_and_guards_history() -> None:
    server = _load_server()
    old_cwd = Path.cwd()
    with tempfile.TemporaryDirectory() as tmpdir:
        project = Path(tmpdir)
        _prepare_project(project)
        os.chdir(project)
        try:
            created = server.create_gate(
                name="release-check",
                description="Original release gate.",
                kind="manual",
                validator="Original validator.",
            )
            repaired = server.update_gate(
                created["id"],
                validator="Updated validator.",
                evidence_required=True,
            )
            evaluated = server.evaluate_gate(
                created["id"],
                "passed",
                actor="ACTOR-test-agent",
                evidence="ART-test-evidence",
            )
            blocked = server.update_gate(created["id"], blocking=False)
            forced = server.update_gate(
                created["id"],
                blocking=False,
                force=True,
            )
        finally:
            os.chdir(old_cwd)

        assert "error" not in repaired, repaired
        assert repaired["updated"] == ["evidence_required", "validator"]
        assert "error" not in evaluated, evaluated
        assert "evaluation history" in blocked["error"]
        assert "error" not in forced, forced
        assert forced["force"] is True

        data = yaml.safe_load(
            Path(created["path"]).read_text(encoding="utf-8").split("---", 2)[1]
        )
        assert data["spec"]["validator"] == "Updated validator."
        assert data["spec"]["blocking"] is False


if __name__ == "__main__":
    test_update_gate_repairs_unobserved_gate_and_guards_history()
