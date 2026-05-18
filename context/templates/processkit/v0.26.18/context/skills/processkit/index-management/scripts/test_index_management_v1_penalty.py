"""Tests for v1-entity penalty on semantic_search_entities and hybrid_search_entities.

Covers BACK-20260510_0344-MightyWolf (follow-up of BACK-20260509_1318-WarmOak).

Run with:

    uv run --with pyyaml --with pytest --with mcp \
        pytest src/context/skills/processkit/index-management/scripts/test_index_management_v1_penalty.py -v
"""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from unittest.mock import patch

_REPO_ROOT = Path(__file__).resolve().parents[6]
_SERVER_PATH = (
    _REPO_ROOT
    / "src"
    / "context"
    / "skills"
    / "processkit"
    / "index-management"
    / "mcp"
    / "server.py"
)


def _load_server():
    lib_path = _REPO_ROOT / "src" / "context" / "skills" / "_lib"
    if str(lib_path) not in sys.path:
        sys.path.insert(0, str(lib_path))
    spec = importlib.util.spec_from_file_location("index_mgmt_server", _SERVER_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# ---------------------------------------------------------------------------
# Unit-level tests for helper functions (no DB, no filesystem)
# ---------------------------------------------------------------------------


def test_is_v1_true_for_v1_suffix():
    srv = _load_server()
    assert srv._is_v1("processkit.projectious.work/v1") is True


def test_is_v1_false_for_v2_suffix():
    srv = _load_server()
    assert srv._is_v1("processkit.projectious.work/v2") is False


def test_is_v1_false_for_empty_string():
    srv = _load_server()
    assert srv._is_v1("") is False


def test_v1_finding_known_kind_returns_superseded():
    srv = _load_server()
    finding_id, successor = srv._v1_finding("Actor")
    assert finding_id == "v1.entity-superseded"
    assert successor == "TeamMember"


def test_v1_finding_unknown_kind_returns_still_v1():
    srv = _load_server()
    finding_id, successor = srv._v1_finding("WorkItem")
    assert finding_id == "v1.entity-still-v1"
    assert successor is None


def test_annotate_row_v2_entity_no_penalty():
    srv = _load_server()
    row: dict = {"id": "BACK-001", "kind": "WorkItem"}
    result = srv._annotate_row(
        row, "processkit.projectious.work/v2", penalty=0.3, apply_score=False
    )
    assert result["v1_penalty_applied"] is False
    assert result["v1_successor_hint"] is None
    assert "v1_trace" not in result


def test_annotate_row_v1_superseded_entity_sets_penalty():
    srv = _load_server()
    row: dict = {"id": "ACT-001", "kind": "Actor"}
    result = srv._annotate_row(
        row,
        "processkit.projectious.work/v1",
        penalty=0.3,
        apply_score=True,
        base_score=1.0,
    )
    assert result["v1_penalty_applied"] is True
    assert result["v1_successor_hint"] == "TeamMember"
    assert result["v1_adjusted_score"] == pytest_approx(0.3)
    assert "v1-entity penalty" in result.get("v1_trace", "")


def test_annotate_row_v1_still_v1_no_score_penalty():
    """v1 entities with no successor are flagged but not score-penalised."""
    srv = _load_server()
    row: dict = {"id": "WI-001", "kind": "WorkItem"}
    result = srv._annotate_row(
        row,
        "processkit.projectious.work/v1",
        penalty=0.3,
        apply_score=True,
        base_score=0.8,
    )
    # No successor → no penalty applied to score
    assert result["v1_penalty_applied"] is False
    # Score is preserved unchanged
    assert result["v1_adjusted_score"] == pytest_approx(0.8)
    assert result["v1_finding_id"] == "v1.entity-still-v1"


def pytest_approx(val):
    """Inline approx helper to avoid pytest import at module level."""
    import pytest
    return pytest.approx(val, rel=1e-6)


# ---------------------------------------------------------------------------
# Integration tests: patch _open() so no DB or filesystem needed
# ---------------------------------------------------------------------------


def _make_entity_row(
    entity_id: str,
    kind: str,
    api_version: str,
    *,
    distance: float | None = None,
    hybrid_score: float | None = None,
) -> dict:
    row: dict = {"id": entity_id, "kind": kind, "title": f"Title {entity_id}"}
    if distance is not None:
        row["distance"] = distance
    if hybrid_score is not None:
        row["hybrid_score"] = hybrid_score
    return row


class _FakeDB:
    """Minimal DB double used by integration tests."""

    def __init__(self, api_version_map: dict[str, str]) -> None:
        self._map = api_version_map

    def execute(self, sql: str, params=()) -> "_FakeDB":
        self._rows = [
            {"id": eid, "api_version": av}
            for eid, av in self._map.items()
            if eid in params
        ]
        return self

    def fetchall(self):
        return [_FakeRow(r) for r in getattr(self, "_rows", [])]

    def close(self) -> None:
        pass


class _FakeRow(dict):
    """Dict subclass that also supports row["key"] access from sqlite3.Row."""

    def __getitem__(self, key):
        if isinstance(key, int):
            return list(self.values())[key]
        return super().__getitem__(key)


def _patch_open(srv, fake_db: _FakeDB):
    """Return a context manager that patches srv._open to return fake_db."""
    from unittest.mock import patch as _patch, MagicMock

    root_mock = MagicMock()
    return _patch.object(srv, "_open", return_value=(root_mock, fake_db))


# -- semantic_search_entities ------------------------------------------------


def test_semantic_search_v1_superseded_entity_is_penalised():
    """A v1 Actor in semantic results gets score multiplied by penalty."""
    import pytest

    srv = _load_server()

    v1_id = "ACT-20260101_0000-OldActor-some-old-actor"
    v2_id = "TM-20260101_0001-NewMember-new-member"
    fake_db = _FakeDB(
        {
            v1_id: "processkit.projectious.work/v1",
            v2_id: "processkit.projectious.work/v2",
        }
    )

    sem_rows = [
        _make_entity_row(v1_id, "Actor", "", distance=0.1),  # distance small → high score
        _make_entity_row(v2_id, "TeamMember", "", distance=0.5),
    ]

    with (
        patch.object(srv, "_open", return_value=(None, fake_db)),
        patch.object(srv.index, "semantic_search_entities", return_value=sem_rows),
        patch.object(srv, "_v1_penalty", return_value=0.3),
    ):
        result = srv.semantic_search_entities("actor")

    ids = [r["id"] for r in result]
    # v2 entity should now rank above the penalised v1 entity
    assert ids.index(v2_id) < ids.index(v1_id)

    v1_result = next(r for r in result if r["id"] == v1_id)
    assert v1_result["v1_penalty_applied"] is True
    assert v1_result["v1_successor_hint"] == "TeamMember"
    assert "v1_adjusted_score" in v1_result

    v2_result = next(r for r in result if r["id"] == v2_id)
    assert v2_result["v1_penalty_applied"] is False


def test_semantic_search_empty_list_returns_empty():
    srv = _load_server()
    fake_db = _FakeDB({})

    with (
        patch.object(srv, "_open", return_value=(None, fake_db)),
        patch.object(srv.index, "semantic_search_entities", return_value=[]),
    ):
        result = srv.semantic_search_entities("nothing")

    assert result == []


def test_semantic_search_no_v1_entities_all_pass_through():
    """When no v1 entities are present, all results pass through unpenalised."""
    srv = _load_server()

    v2a = "WI-001"
    v2b = "WI-002"
    fake_db = _FakeDB(
        {
            v2a: "processkit.projectious.work/v2",
            v2b: "processkit.projectious.work/v2",
        }
    )

    sem_rows = [
        _make_entity_row(v2a, "WorkItem", "", distance=0.1),
        _make_entity_row(v2b, "WorkItem", "", distance=0.2),
    ]

    with (
        patch.object(srv, "_open", return_value=(None, fake_db)),
        patch.object(srv.index, "semantic_search_entities", return_value=sem_rows),
        patch.object(srv, "_v1_penalty", return_value=0.3),
    ):
        result = srv.semantic_search_entities("workitem")

    assert all(r["v1_penalty_applied"] is False for r in result)
    # Lower distance → higher base score → should remain first
    assert result[0]["id"] == v2a


def test_semantic_search_v1_adjusted_score_equals_base_times_penalty():
    """v1_adjusted_score is exactly base_score * penalty for superseded kinds."""
    import pytest

    srv = _load_server()

    v1_id = "ACT-20260101_0000-OldActor-old"
    fake_db = _FakeDB({v1_id: "processkit.projectious.work/v1"})
    distance = 1.0  # base_score = 1/(1+1.0) = 0.5
    expected_score = 0.5 * 0.3

    with (
        patch.object(srv, "_open", return_value=(None, fake_db)),
        patch.object(
            srv.index,
            "semantic_search_entities",
            return_value=[_make_entity_row(v1_id, "Actor", "", distance=distance)],
        ),
        patch.object(srv, "_v1_penalty", return_value=0.3),
    ):
        result = srv.semantic_search_entities("actor")

    assert len(result) == 1
    assert result[0]["v1_adjusted_score"] == pytest.approx(expected_score, rel=1e-6)


# -- hybrid_search_entities --------------------------------------------------


def test_hybrid_search_v1_superseded_entity_is_penalised():
    """A v1 Actor in hybrid results gets its hybrid_score multiplied by penalty."""
    import pytest

    srv = _load_server()

    v1_id = "ACT-20260101_0000-OldActor-hybrid"
    v2_id = "TM-20260101_0001-NewMember-hybrid"
    fake_db = _FakeDB(
        {
            v1_id: "processkit.projectious.work/v1",
            v2_id: "processkit.projectious.work/v2",
        }
    )

    hybrid_rows = [
        # v1 Actor has a higher raw hybrid_score but should drop after penalty
        _make_entity_row(v1_id, "Actor", "", hybrid_score=0.9),
        _make_entity_row(v2_id, "TeamMember", "", hybrid_score=0.5),
    ]

    with (
        patch.object(srv, "_open", return_value=(None, fake_db)),
        patch.object(srv.index, "hybrid_search_entities", return_value=hybrid_rows),
        patch.object(srv, "_v1_penalty", return_value=0.3),
    ):
        result = srv.hybrid_search_entities("actor")

    ids = [r["id"] for r in result]
    # v2 (adjusted 0.5) should outrank penalised v1 (0.9*0.3=0.27)
    assert ids.index(v2_id) < ids.index(v1_id)

    v1_result = next(r for r in result if r["id"] == v1_id)
    assert v1_result["v1_penalty_applied"] is True
    assert v1_result["v1_adjusted_score"] == pytest.approx(0.9 * 0.3, rel=1e-6)

    v2_result = next(r for r in result if r["id"] == v2_id)
    assert v2_result["v1_penalty_applied"] is False
    assert v2_result["v1_adjusted_score"] == pytest.approx(0.5, rel=1e-6)


def test_hybrid_search_empty_list_returns_empty():
    srv = _load_server()
    fake_db = _FakeDB({})

    with (
        patch.object(srv, "_open", return_value=(None, fake_db)),
        patch.object(srv.index, "hybrid_search_entities", return_value=[]),
    ):
        result = srv.hybrid_search_entities("nothing")

    assert result == []


def test_hybrid_search_no_v1_entities_preserve_order():
    """When no v1 entities are present, original RRF order is preserved."""
    srv = _load_server()

    v2a = "WI-100"
    v2b = "WI-101"
    fake_db = _FakeDB(
        {
            v2a: "processkit.projectious.work/v2",
            v2b: "processkit.projectious.work/v2",
        }
    )

    hybrid_rows = [
        _make_entity_row(v2a, "WorkItem", "", hybrid_score=0.8),
        _make_entity_row(v2b, "WorkItem", "", hybrid_score=0.3),
    ]

    with (
        patch.object(srv, "_open", return_value=(None, fake_db)),
        patch.object(srv.index, "hybrid_search_entities", return_value=hybrid_rows),
        patch.object(srv, "_v1_penalty", return_value=0.3),
    ):
        result = srv.hybrid_search_entities("workitem")

    assert result[0]["id"] == v2a
    assert result[1]["id"] == v2b
    assert all(r["v1_penalty_applied"] is False for r in result)


def test_hybrid_search_v1_trace_surfaced():
    """v1_trace must be present and mention the penalty for superseded entities."""
    srv = _load_server()

    v1_id = "ACT-20260101_0000-OldActor-trace"
    fake_db = _FakeDB({v1_id: "processkit.projectious.work/v1"})

    with (
        patch.object(srv, "_open", return_value=(None, fake_db)),
        patch.object(
            srv.index,
            "hybrid_search_entities",
            return_value=[_make_entity_row(v1_id, "Actor", "", hybrid_score=0.5)],
        ),
        patch.object(srv, "_v1_penalty", return_value=0.3),
    ):
        result = srv.hybrid_search_entities("actor")

    assert len(result) == 1
    trace = result[0].get("v1_trace", "")
    assert "v1-entity penalty" in trace
    assert "TeamMember" in trace
