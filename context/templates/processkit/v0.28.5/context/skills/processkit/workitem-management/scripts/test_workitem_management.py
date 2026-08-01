#!/usr/bin/env python3
"""Regression tests for workitem-management MCP helpers."""
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
    / "workitem-management"
    / "mcp"
    / "server.py"
)


def _load_server():
    spec = importlib.util.spec_from_file_location("workitem_server", SERVER_PATH)
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


def test_create_process_instance_persists_definition_for_short_titles() -> None:
    server = _load_server()
    old_cwd = Path.cwd()
    with tempfile.TemporaryDirectory() as tmpdir:
        project = Path(tmpdir)
        _prepare_project(project)
        os.chdir(project)
        try:
            result = server.create_process_instance(
                title="Release",
                process_definition_artifact="ART-test-process-definition",
                steps=["Plan", "Ship"],
            )
        finally:
            os.chdir(old_cwd)

        assert "error" not in result, result
        assert result["process_definition_artifact"] == "ART-test-process-definition"
        assert len(result["children"]) == 2

        parent_path = Path(result["path"])
        data = yaml.safe_load(parent_path.read_text(encoding="utf-8").split("---", 2)[1])
        assert data["spec"]["type"] == "process-instance"
        assert (
            data["spec"]["process_definition_artifact"]
            == "ART-test-process-definition"
        )

        os.chdir(project)
        try:
            fetched = server.get_workitem(result["id"])
        finally:
            os.chdir(old_cwd)
        assert fetched["process_definition_artifact"] == "ART-test-process-definition"
        assert (
            fetched["spec"]["process_definition_artifact"]
            == "ART-test-process-definition"
        )


def test_update_workitem_can_repair_missing_process_definition() -> None:
    server = _load_server()
    old_cwd = Path.cwd()
    with tempfile.TemporaryDirectory() as tmpdir:
        project = Path(tmpdir)
        _prepare_project(project)
        os.chdir(project)
        try:
            created = server.create_workitem(
                title="Repair stranded process instance",
                type="process-instance",
            )
            result = server.update_workitem(
                created["id"],
                process_definition_artifact="ART-repair-process-definition",
            )
        finally:
            os.chdir(old_cwd)

        assert "error" not in result, result
        assert result["updated"] == ["process_definition_artifact"]
        assert (
            result["spec"]["process_definition_artifact"]
            == "ART-repair-process-definition"
        )

        data = yaml.safe_load(
            Path(created["path"]).read_text(encoding="utf-8").split("---", 2)[1]
        )
        assert (
            data["spec"]["process_definition_artifact"]
            == "ART-repair-process-definition"
        )


if __name__ == "__main__":
    test_create_process_instance_persists_definition_for_short_titles()
    test_update_workitem_can_repair_missing_process_definition()
