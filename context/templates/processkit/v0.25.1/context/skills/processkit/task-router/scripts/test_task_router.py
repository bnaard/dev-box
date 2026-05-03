"""Tests for task-router scoring and escalation metadata.

Run with:

    uv run --with pyyaml --with pytest --with mcp \
        pytest context/skills/processkit/task-router/scripts/test_task_router.py -v
"""
from __future__ import annotations

import sys
import importlib.util
from pathlib import Path


_REPO_ROOT = Path(__file__).resolve().parents[5]
_SERVER_DIR = _REPO_ROOT / "context" / "skills" / "processkit" / "task-router" / "mcp"
sys.path.insert(0, str(_SERVER_DIR))


def _load_server():
    spec = importlib.util.spec_from_file_location(
        "task_router_server",
        _SERVER_DIR / "server.py",
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_route_task_reports_embedding_scoring_basis():
    server = _load_server()

    out = server.route_task(
        "find the process primitive for an architectural choice",
        scoring_mode="embedding",
    )

    assert "error" not in out
    assert out["scoring_basis"]["group"] == "embedding"
    assert out["candidate_tools"][0]["scoring_basis"] == "embedding"


def test_route_task_rejects_unknown_scoring_mode():
    server = _load_server()

    out = server.route_task("create a backlog item", scoring_mode="bogus")

    assert "scoring_mode must be auto" in out["error"]


def test_route_task_surfaces_escalation_status_for_low_confidence():
    server = _load_server()

    out = server.route_task(
        "task",
        allow_llm_escalation=True,
        scoring_mode="token",
    )

    assert out["routing_basis"] == "needs_llm_confirm"
    assert out["llm_escalation"]["model_class"] == "fast"
    assert out["llm_escalation"]["status"] in {
        "not_configured",
        "configured_but_not_invoked",
    }
