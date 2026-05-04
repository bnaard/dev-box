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
        "record a decision about an architectural choice",
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

    assert out["routing_basis"] in {
        "needs_llm_confirm",
        "skill_finder_trigger_table",
    }
    assert out["llm_escalation"]["model_class"] == "fast"
    assert out["llm_escalation"]["status"] in {
        "not_configured",
        "configured_but_not_invoked",
    }


def test_route_task_uses_command_signals_for_release_requests():
    server = _load_server()

    out = server.route_task(
        "commit, push, and make a patch release",
        scoring_mode="token",
    )

    assert out["route_type"] == "command"
    assert out["command"] == "pk-release-audit"
    assert out["skill"] == "release-audit"


def test_route_task_accepts_intent_signals():
    server = _load_server()

    out = server.route_task(
        "finish the package",
        intent_signals=["patch release", "release audit"],
        scoring_mode="token",
    )

    assert out["command"] == "pk-release-audit"
    assert out["intent_signals"] == ["patch release", "release audit"]
