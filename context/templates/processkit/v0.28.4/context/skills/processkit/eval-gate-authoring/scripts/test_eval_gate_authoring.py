#!/usr/bin/env python3
"""Regression tests for eval-gate-authoring MCP helpers."""
from __future__ import annotations

import importlib.util
import os
import shutil
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[5]
SERVER_PATH = (
    REPO_ROOT
    / "context"
    / "skills"
    / "processkit"
    / "eval-gate-authoring"
    / "mcp"
    / "server.py"
)


def _load_server():
    spec = importlib.util.spec_from_file_location("eval_gate_server", SERVER_PATH)
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


def test_codify_eval_validates_gate_before_writing_artifact() -> None:
    server = _load_server()
    original_validate = server.schema.validate_spec

    def fail_gate_only(kind: str, spec: dict) -> list[str]:
        if kind == "Gate":
            return ["forced gate schema failure"]
        return original_validate(kind, spec)

    old_cwd = Path.cwd()
    with tempfile.TemporaryDirectory() as tmpdir:
        project = Path(tmpdir)
        _prepare_project(project)
        os.chdir(project)
        server.schema.validate_spec = fail_gate_only
        try:
            result = server.codify_eval(
                category_id="CAT-test",
                name="release judge",
                eval_kind="llm-as-judge",
                description="Evaluate whether release evidence is sufficient.",
                judge="MODEL-test-judge",
            )
        finally:
            server.schema.validate_spec = original_validate
            os.chdir(old_cwd)

        assert result["error"] == "gate schema validation failed"
        assert not (project / "context" / "artifacts").exists()
        assert not (project / "context" / "gates").exists()


if __name__ == "__main__":
    test_codify_eval_validates_gate_before_writing_artifact()
