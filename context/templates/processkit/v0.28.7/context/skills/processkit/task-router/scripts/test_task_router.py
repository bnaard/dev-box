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


def test_phase1_groups_applies_v1_penalty_to_actor_group():
    """BACK-20260509_1318-WarmOak: v1 actor group is down-weighted with
    a configurable penalty, and the penalty is surfaced via trace[]."""
    server = _load_server()

    trace: list[str] = []
    scored = server._phase1_groups(
        "create a new actor profile",
        scoring_mode="token",
        trace=trace,
    )

    by_name = {name: score for score, name, _ in scored}
    assert "actor" in by_name
    # Actor-group score should be at most penalty * 1.0 (substring match
    # would otherwise yield 1.0).
    assert by_name["actor"] <= server._v1_penalty() + 1e-9
    # Trace surfaces the penalty and the v2 successor hint.
    assert any("v1-entity penalty" in line for line in trace)
    assert any("team-manager" in line for line in trace)


def test_route_task_surfaces_subagent_dispatch_fields():
    """BACK-20260509_1317-WildPanda P2: route_task always returns
    `recommended_team_member_slug` and `recommended_model_class` keys
    so dispatchers can pass the recommended TeamMember slug as the
    sub-agent's identity (compliance contract sub-agent-dispatch clause).
    """
    server = _load_server()

    out = server.route_task(
        "create a work item for the login bug", scoring_mode="token"
    )

    assert "error" not in out
    assert "recommended_team_member_slug" in out
    assert "recommended_model_class" in out
    # workitem group → fast class (deterministic group→class mapping)
    assert out["recommended_model_class"] == "fast"


def test_recommend_team_member_resolves_pm_for_workitem_group():
    """When a TeamMember with `default_role: ROLE-product-manager` is
    active in the roster (e.g. Cora), the workitem group routes to that
    slug. Active slug presence is repo-state dependent: assert structure
    only when the helper resolves a non-None slug.
    """
    server = _load_server()

    slug = server._recommend_team_member("workitem")
    if slug is not None:
        # If resolved, must be a string slug (matches an active TeamMember).
        assert isinstance(slug, str) and slug
        members = {tm["slug"] for tm in server._list_active_team_members()}
        assert slug in members


def test_skill_finder_fallback_applies_v1_penalty_for_v1_skill():
    server = _load_server()

    trace: list[str] = []
    fb = server._skill_finder_fallback(
        "manage the release process state machine", trace=trace
    )
    if fb is None:
        # Fallback table changes over time; only assert when a match exists.
        return
    if any(
        line.startswith("matched skill 'process-management'")
        or line.startswith("matched skill 'state-machine-management'")
        for line in trace
    ):
        assert any("v1-entity penalty" in line for line in trace)
