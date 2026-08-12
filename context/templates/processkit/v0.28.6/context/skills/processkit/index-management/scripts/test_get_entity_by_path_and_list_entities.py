"""Tests for get_entity_by_path and list_entities in index-management MCP server.

Covers BACK-20260510_0751-TallFern (T1.1 and T1.2).

Run with:

    uv run --with pyyaml --with pytest --with mcp --with sqlite-vec \
        pytest context/skills/processkit/index-management/scripts/test_get_entity_by_path_and_list_entities.py -v
"""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from unittest.mock import patch, MagicMock

_REPO_ROOT = Path(__file__).resolve().parents[5]
_SERVER_PATH = (
    _REPO_ROOT
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
    spec = importlib.util.spec_from_file_location("index_mgmt_server_t1", _SERVER_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# ---------------------------------------------------------------------------
# Helpers to build minimal fake DB rows
# ---------------------------------------------------------------------------

def _make_row(id: str, kind: str, state: str, path: str) -> dict:
    return {
        "id": id,
        "kind": kind,
        "state": state,
        "title": f"Test {id}",
        "path": path,
        "created": "2026-01-01T00:00:00+00:00",
        "updated": None,
        "spec": "{}",
        "body": "",
        "api_version": "processkit.projectious.work/v2",
    }


# ---------------------------------------------------------------------------
# Tests for get_entity_by_path
# ---------------------------------------------------------------------------


class TestGetEntityByPath:
    """Tests for get_entity_by_path()."""

    def test_resolves_absolute_path_to_entity(self):
        """A known absolute path returns the entity dict."""
        srv = _load_server()

        wi_path = str(_REPO_ROOT / "context" / "workitems" / "2026" / "05" /
                      "BACK-20260510_0751-TallFern-mcp-gateway-adoption-close-usage-gaps.md")
        fake_row = _make_row(
            "BACK-20260510_0751-TallFern-mcp-gateway-adoption-close-usage-gaps",
            "WorkItem", "in-progress", wi_path,
        )

        fake_db = MagicMock()
        fake_db.execute.return_value.fetchall.return_value = [{"id": fake_row["id"]}]

        with (
            patch.object(srv, "_open", return_value=(_REPO_ROOT, fake_db)),
            patch.object(srv.index, "resolve_entity", return_value=(fake_row, [])),
            patch.object(srv, "_api_versions_for", return_value={fake_row["id"]: "processkit.projectious.work/v2"}),
            patch.object(srv, "_v1_penalty", return_value=0.3),
        ):
            result = srv.get_entity_by_path(wi_path)

        assert result["id"] == fake_row["id"]
        assert result["kind"] == "WorkItem"
        assert result.get("error") is None

    def test_returns_error_for_unknown_path(self):
        """An unrecognized path returns an error dict."""
        srv = _load_server()

        fake_db = MagicMock()
        fake_db.execute.return_value.fetchall.return_value = []

        with (
            patch.object(srv, "_open", return_value=(_REPO_ROOT, fake_db)),
            patch.object(srv.index, "resolve_entity", return_value=(None, [])),
            patch.object(srv, "paths") as mock_paths,
        ):
            mock_paths.find_project_root.return_value = _REPO_ROOT
            result = srv.get_entity_by_path("context/nonexistent/FAKE-entity.md")

        assert "error" in result

    def test_returns_error_for_ambiguous_path(self):
        """A path whose stem resolves to multiple entities returns an error."""
        srv = _load_server()

        fake_db = MagicMock()
        fake_db.execute.return_value.fetchall.return_value = []

        with (
            patch.object(srv, "_open", return_value=(_REPO_ROOT, fake_db)),
            patch.object(srv.index, "resolve_entity", return_value=(None, ["ID-A", "ID-B"])),
            patch.object(srv, "paths") as mock_paths,
        ):
            mock_paths.find_project_root.return_value = _REPO_ROOT
            result = srv.get_entity_by_path("context/workitems/AMBIGUOUS.md")

        assert "error" in result
        assert "ambiguous" in result["error"]

    def test_handles_terminal_subdir_path(self):
        """A path containing done/ subdir is resolved correctly."""
        srv = _load_server()

        dec_path = str(_REPO_ROOT / "context" / "decisions" / "done" /
                       "DEC-20260510_0758-FierceFern-mcp-gateway-adoption-ux-defaults-for.md")
        fake_row = _make_row(
            "DEC-20260510_0758-FierceFern-mcp-gateway-adoption-ux-defaults-for",
            "DecisionRecord", "accepted", dec_path,
        )

        fake_db = MagicMock()
        fake_db.execute.return_value.fetchall.return_value = [{"id": fake_row["id"]}]

        with (
            patch.object(srv, "_open", return_value=(_REPO_ROOT, fake_db)),
            patch.object(srv.index, "resolve_entity", return_value=(fake_row, [])),
            patch.object(srv, "_api_versions_for", return_value={fake_row["id"]: "processkit.projectious.work/v2"}),
            patch.object(srv, "_v1_penalty", return_value=0.3),
        ):
            result = srv.get_entity_by_path(dec_path)

        assert result["id"] == fake_row["id"]
        assert result.get("error") is None


# ---------------------------------------------------------------------------
# Tests for list_entities
# ---------------------------------------------------------------------------


class TestListEntities:
    """Tests for list_entities()."""

    def test_returns_all_when_no_filters(self):
        """No filters returns up to limit rows with v1 annotations."""
        srv = _load_server()

        rows = [
            _make_row("WI-001", "WorkItem", "open", "/p/context/workitems/WI-001.md"),
            _make_row("DEC-001", "DecisionRecord", "accepted", "/p/context/decisions/DEC-001.md"),
        ]

        fake_db = MagicMock()

        with (
            patch.object(srv, "_open", return_value=(_REPO_ROOT, fake_db)),
            patch.object(srv.index, "query_entities", return_value=rows),
            patch.object(srv, "_api_versions_for", return_value={
                "WI-001": "processkit.projectious.work/v2",
                "DEC-001": "processkit.projectious.work/v2",
            }),
            patch.object(srv, "_v1_penalty", return_value=0.3),
        ):
            result = srv.list_entities()

        assert len(result) == 2
        assert all("v1_penalty_applied" in r for r in result)

    def test_filters_by_kind(self):
        """kind= filter is passed through to query_entities."""
        srv = _load_server()

        wi_row = _make_row("WI-001", "WorkItem", "open", "/p/context/workitems/WI-001.md")
        fake_db = MagicMock()

        with (
            patch.object(srv, "_open", return_value=(_REPO_ROOT, fake_db)),
            patch.object(srv.index, "query_entities", return_value=[wi_row]) as mock_qe,
            patch.object(srv, "_api_versions_for", return_value={"WI-001": "processkit.projectious.work/v2"}),
            patch.object(srv, "_v1_penalty", return_value=0.3),
        ):
            result = srv.list_entities(kind="WorkItem", limit=10)

        mock_qe.assert_called_once_with(fake_db, kind="WorkItem", state=None, limit=10)
        assert len(result) == 1

    def test_filters_by_state(self):
        """state= filter is passed through to query_entities."""
        srv = _load_server()

        dec_row = _make_row("DEC-001", "DecisionRecord", "accepted", "/p/context/decisions/DEC-001.md")
        fake_db = MagicMock()

        with (
            patch.object(srv, "_open", return_value=(_REPO_ROOT, fake_db)),
            patch.object(srv.index, "query_entities", return_value=[dec_row]) as mock_qe,
            patch.object(srv, "_api_versions_for", return_value={"DEC-001": "processkit.projectious.work/v2"}),
            patch.object(srv, "_v1_penalty", return_value=0.3),
        ):
            result = srv.list_entities(state="accepted")

        mock_qe.assert_called_once_with(fake_db, kind=None, state="accepted", limit=50)
        assert len(result) == 1

    def test_returns_empty_list_when_no_results(self):
        """Empty DB returns empty list without error."""
        srv = _load_server()

        fake_db = MagicMock()

        with (
            patch.object(srv, "_open", return_value=(_REPO_ROOT, fake_db)),
            patch.object(srv.index, "query_entities", return_value=[]),
        ):
            result = srv.list_entities(kind="TeamMember")

        assert result == []

    def test_v1_penalty_annotated_on_results(self):
        """v1 entities get penalty annotation."""
        srv = _load_server()

        actor_row = _make_row("ACTOR-legacy", "Actor", "active", "/p/context/actors/ACTOR-legacy.md")
        fake_db = MagicMock()

        with (
            patch.object(srv, "_open", return_value=(_REPO_ROOT, fake_db)),
            patch.object(srv.index, "query_entities", return_value=[actor_row]),
            patch.object(srv, "_api_versions_for", return_value={"ACTOR-legacy": "processkit.projectious.work/v1"}),
            patch.object(srv, "_v1_penalty", return_value=0.3),
        ):
            result = srv.list_entities(kind="Actor")

        assert len(result) == 1
        assert result[0]["v1_penalty_applied"] is True
        assert result[0]["v1_successor_hint"] == "TeamMember"
