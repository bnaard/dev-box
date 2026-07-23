#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
#   "sqlite-vec>=0.1.0",
# ]
# ///
"""processkit index-management MCP server.

Walks the consumer project's ``context/`` directory, parses every entity
file, and writes a SQLite index that other MCP servers (and the agent)
can query.

Tools provided:
    reindex()                              -> stats
    query_entities(kind?, state?, limit?)  -> [entities]
    get_entity(id)                         -> entity | null
    get_entity_by_path(path)               -> entity | {error}
    list_entities(kind?, state?, limit?)   -> [entities]
    search_entities(text, limit?)          -> [entities]
    semantic_status()                      -> {capabilities}
    semantic_search_entities(text, limit?) -> [entities]
    hybrid_search_entities(text, limit?)   -> [entities]
    query_events(event_type?, subject?, actor?, limit?) -> [events]
    list_errors(limit?)                    -> [{path, message}]
    stats()                                -> counts

Run from a processkit checkout (development) or installed by aibox into
``context/skills/index-management/mcp/server.py`` (consumer install,
provider-neutral path).

Lib resolution: walks up from this file to find ``src/lib/processkit/``
(checkout) or ``_lib/processkit/`` (consumer-install layout — resolves
to ``context/skills/_lib/processkit/`` after the walk-up).
"""
from __future__ import annotations

import json
import os
import sys
from pathlib import Path


def _find_lib() -> Path:
    """Locate the processkit shared library directory.

    Search order:
      1. PROCESSKIT_LIB_PATH environment variable
      2. <ancestor>/src/lib/processkit/__init__.py (running from
         a processkit checkout)
      3. <ancestor>/_lib/processkit/__init__.py    (consumer install,
         e.g. <project>/context/skills/_lib/processkit/__init__.py
         when invoked as <project>/context/skills/<name>/mcp/server.py)
    """
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for candidate in (
            here / "src" / "lib",
            here / "_lib",
        ):
            if (candidate / "processkit" / "__init__.py").is_file():
                return candidate
        if here.parent == here:
            raise RuntimeError(
                "could not locate processkit shared library "
                "(set PROCESSKIT_LIB_PATH or run from a processkit checkout)"
            )
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import index, paths  # noqa: E402

server = FastMCP("processkit-index-management")


def _open() -> tuple[Path, "index.sqlite3.Connection"]:  # noqa: F821
    root = paths.find_project_root()
    db = index.open_db()
    return root, db


# ---------------------------------------------------------------------------
# v1-entity penalty / annotation (BACK-20260509_1318-WarmOak / GH #21).
#
# Mirrors the per-kind successor table from
# pk-doctor/scripts/checks/v1_entity_drift.py and shares the
# ``v1_entity_penalty`` knob with task-router (single config primitive).
# Entries with no v2 successor (WorkItem, DecisionRecord, ...) are flagged
# with the ``v1.entity-still-v1`` finding id but receive no penalty.
# ---------------------------------------------------------------------------

_V1_ENTITY_SUCCESSORS: dict[str, str] = {
    "Actor": "TeamMember",
    "Process": "Scope + Gate",
    "StateMachine": "lifecycle metadata on the owning entity",
    "Model": "Artifact(kind=model-spec)",
}

_DEFAULT_V1_PENALTY = 0.3


def _v1_penalty() -> float:
    """Return the configured v1-entity score multiplier.

    Reads ``v1_entity_penalty`` from task-router/mcp/user_config.json so
    routing and entity-read surfaces stay aligned (single config knob).
    Default 0.3.
    """
    cfg_path = (
        paths.find_project_root()
        / "context" / "skills" / "processkit"
        / "task-router" / "mcp" / "user_config.json"
    )
    try:
        cfg = json.loads(cfg_path.read_text(encoding="utf-8"))
    except Exception:
        return _DEFAULT_V1_PENALTY
    val = cfg.get("v1_entity_penalty")
    try:
        penalty = float(val)
    except (TypeError, ValueError):
        return _DEFAULT_V1_PENALTY
    if penalty < 0.0 or penalty > 1.0:
        return _DEFAULT_V1_PENALTY
    return penalty


def _api_versions_for(
    db: "index.sqlite3.Connection",  # noqa: F821
    ids: list[str],
) -> dict[str, str]:
    """Bulk-fetch ``api_version`` for the given entity IDs."""
    if not ids:
        return {}
    placeholders = ",".join(["?"] * len(ids))
    rows = db.execute(
        f"SELECT id, api_version FROM entities WHERE id IN ({placeholders})",
        ids,
    ).fetchall()
    return {row["id"]: (row["api_version"] or "") for row in rows}


def _is_v1(api_version: str) -> bool:
    return bool(api_version) and api_version.endswith("/v1")


def _v1_finding(kind: str | None) -> tuple[str, str | None]:
    """Return (finding_id, successor_hint) for a v1 entity of ``kind``.

    Mirrors the pk-doctor v1_entity_drift finding-id pattern:
    ``v1.entity-superseded`` when a successor exists, ``v1.entity-still-v1``
    otherwise.
    """
    successor = _V1_ENTITY_SUCCESSORS.get(kind or "")
    if successor:
        return "v1.entity-superseded", successor
    return "v1.entity-still-v1", None


def _annotate_row(
    row: dict,
    api_version: str,
    *,
    penalty: float,
    apply_score: bool = False,
    base_score: float | None = None,
) -> dict:
    """Add v1-penalty annotations to ``row`` in place; return ``row``.

    Always sets ``v1_penalty_applied`` and ``v1_successor_hint``. When
    ``apply_score`` is true and the row is a v1-superseded entity, also
    multiplies ``base_score`` by the penalty into ``v1_adjusted_score``
    and emits a ``v1_trace`` line in the task-router trace format.
    """
    row.setdefault("api_version", api_version)
    if not _is_v1(api_version):
        row["v1_penalty_applied"] = False
        row["v1_successor_hint"] = None
        if apply_score and base_score is not None:
            row["v1_adjusted_score"] = base_score
        return row

    finding_id, successor = _v1_finding(row.get("kind"))
    has_successor = successor is not None
    row["v1_finding_id"] = finding_id
    row["v1_successor_hint"] = successor
    row["v1_penalty_applied"] = bool(has_successor and penalty < 1.0)

    if apply_score and base_score is not None:
        if has_successor and penalty < 1.0:
            row["v1_adjusted_score"] = base_score * penalty
        else:
            row["v1_adjusted_score"] = base_score
    if has_successor and penalty < 1.0:
        trace = (
            f"v1-entity penalty {penalty} applied to {row.get('id')}; "
            f"consider v2 successor: {successor}"
        )
    else:
        trace = f"v1-entity {row.get('id')} flagged: {finding_id}"
    row["v1_trace"] = trace
    return row


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def reindex() -> dict:
    """Rebuild the SQLite index from scratch.

    Walks ``<project-root>/context/`` recursively, parses every Markdown
    file with YAML frontmatter, and replaces the index contents. Returns
    counts of entities, events, and parse errors.
    """
    root, db = _open()
    stats = index.reindex(root, db)
    db.close()
    return {
        "entities": stats.entities,
        "events": stats.events,
        "errors": stats.errors,
        "project_root": str(root),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_entities(
    kind: str | None = None,
    state: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """List entities matching the filters.

    Parameters
    ----------
    kind: optional primitive kind (e.g. "WorkItem", "DecisionRecord").
    state: optional state to filter by.
    limit: maximum rows to return (default 50).

    Each result is annotated with ``v1_penalty_applied`` and
    ``v1_successor_hint`` (BACK-20260509_1318-WarmOak). Results are
    ordered by ``created DESC`` (no score), so the penalty is surfaced
    as annotation only — no re-ranking is applied.
    """
    _, db = _open()
    try:
        rows = index.query_entities(db, kind=kind, state=state, limit=limit)
        if not rows:
            return rows
        penalty = _v1_penalty()
        api_map = _api_versions_for(db, [r["id"] for r in rows])
        return [
            _annotate_row(
                row,
                api_map.get(row["id"], ""),
                penalty=penalty,
                apply_score=False,
            )
            for row in rows
        ]
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_entity(id: str) -> dict:
    """Fetch a single entity by ID.

    Accepts a full ID, a prefix (missing slug), or a bare word-pair.
    Returns ``{"error": "..."}`` if not found or ambiguous.

    Adds ``v1_penalty_applied`` and ``v1_successor_hint`` to the result
    when the entity is a v1 primitive (BACK-20260509_1318-WarmOak).
    """
    _, db = _open()
    try:
        row, candidates = index.resolve_entity(db, id)
        if candidates:
            return {"error": f"ambiguous ID {id!r}; candidates: {candidates}"}
        if row is None:
            return {"error": f"entity not found: {id!r}"}
        api_map = _api_versions_for(db, [row["id"]])
        return _annotate_row(
            row,
            api_map.get(row["id"], ""),
            penalty=_v1_penalty(),
            apply_score=False,
        )
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_entity_by_path(path: str) -> dict:
    """Fetch a single entity by its relative filesystem path.

    Accepts a path relative to the project root, e.g.
    ``context/workitems/2026/05/BACK-20260510_0751-TallFern-...md`` or an
    absolute path.  Dispatches to ``get_entity`` under the hood after
    deriving the entity ID from the index.

    Handles terminal-state subdirectories (``done/``, ``applied/``,
    ``rejected/``) transparently.

    Returns the same shape as ``get_entity``, or ``{"error": "..."}`` if
    the path cannot be resolved to an indexed entity.
    """
    root = paths.find_project_root()
    p = Path(path)
    if not p.is_absolute():
        p = root / p
    try:
        p = p.resolve()
    except Exception:
        pass

    # Query the DB by absolute path string
    _, db = _open()
    try:
        # Try exact path match first
        rows = db.execute(
            "SELECT id FROM entities WHERE path = ?", (str(p),)
        ).fetchall()
        if not rows:
            # Try matching the basename (ID without extension) as the entity ID
            stem = p.stem
            row2, candidates = index.resolve_entity(db, stem)
            if candidates:
                return {"error": f"ambiguous path {path!r}; candidates: {candidates}"}
            if row2 is None:
                return {"error": f"no entity found for path: {path!r}"}
            entity_id = row2["id"]
        elif len(rows) > 1:
            return {"error": f"path {path!r} matched multiple entities"}
        else:
            entity_id = rows[0]["id"]

        # Now fetch full entity with v1 annotations
        row, candidates = index.resolve_entity(db, entity_id)
        if candidates:
            return {"error": f"ambiguous entity ID {entity_id!r}; candidates: {candidates}"}
        if row is None:
            return {"error": f"entity not found: {entity_id!r}"}
        api_map = _api_versions_for(db, [row["id"]])
        return _annotate_row(
            row,
            api_map.get(row["id"], ""),
            penalty=_v1_penalty(),
            apply_score=False,
        )
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_entities(
    kind: str | None = None,
    state: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """Unified entity listing across all kinds.

    A convenience wrapper over ``query_entities`` that surfaces the same
    v1-entity annotations and accepts the same filters.  Prefer this tool
    over the per-kind ``list_*`` tools when you want a single call that
    works across entity types without remembering per-kind tool names.

    Parameters
    ----------
    kind: optional primitive kind (e.g. "WorkItem", "DecisionRecord",
          "TeamMember", "Artifact", "Scope", "Gate", "Role", "Binding").
    state: optional state filter (e.g. "open", "accepted", "in-progress").
    limit: maximum rows to return (default 50).

    Each result is annotated with ``v1_penalty_applied`` and
    ``v1_successor_hint``. Results are ordered by ``created DESC``.
    """
    _, db = _open()
    try:
        rows = index.query_entities(db, kind=kind, state=state, limit=limit)
        if not rows:
            return rows
        penalty = _v1_penalty()
        api_map = _api_versions_for(db, [r["id"] for r in rows])
        return [
            _annotate_row(
                row,
                api_map.get(row["id"], ""),
                penalty=penalty,
                apply_score=False,
            )
            for row in rows
        ]
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def search_entities(text: str, limit: int = 50) -> list[dict]:
    """Full-text search across entity IDs, titles, bodies, and specs.

    Applies the v1-entity penalty (BACK-20260509_1318-WarmOak): results
    are FTS5-ranked, so we synthesise a per-rank score
    ``1.0 / (1 + rank_index)`` and multiply it by ``_v1_penalty()`` for
    v1-superseded entries. Results are then re-sorted on the adjusted
    score. Each row carries ``v1_penalty_applied``, ``v1_successor_hint``,
    and a ``v1_trace`` line analogous to the task-router trace surface.
    """
    _, db = _open()
    try:
        rows = index.search_entities(db, text, limit=limit)
        if not rows:
            return rows
        penalty = _v1_penalty()
        api_map = _api_versions_for(db, [r["id"] for r in rows])
        annotated: list[dict] = []
        for rank_index, row in enumerate(rows):
            base_score = 1.0 / (1 + rank_index)
            annotated.append(
                _annotate_row(
                    row,
                    api_map.get(row["id"], ""),
                    penalty=penalty,
                    apply_score=True,
                    base_score=base_score,
                )
            )
        annotated.sort(
            key=lambda r: r.get("v1_adjusted_score", 0.0),
            reverse=True,
        )
        return annotated
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def semantic_status() -> dict:
    """Return semantic-index capability and row counts."""
    _, db = _open()
    try:
        return index.semantic_status(db)
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def semantic_search_entities(text: str, limit: int = 50) -> list[dict]:
    """Search entities semantically via sqlite-vec when available.

    Returns an empty list when sqlite-vec is not installed or cannot be
    loaded; use hybrid_search_entities for an FTS-backed fallback.

    Applies the v1-entity penalty (BACK-20260509_1318-WarmOak): the raw
    distance from sqlite-vec is converted to a base score
    ``1.0 / (1 + distance)`` and then multiplied by ``_v1_penalty()``
    for v1-superseded entities. Results are re-ranked on the adjusted
    score. Each row carries ``v1_penalty_applied``, ``v1_successor_hint``,
    and ``v1_trace``.
    """
    _, db = _open()
    try:
        rows = index.semantic_search_entities(db, text, limit=limit)
        if not rows:
            return rows
        penalty = _v1_penalty()
        api_map = _api_versions_for(db, [r["id"] for r in rows])
        annotated: list[dict] = []
        for row in rows:
            distance = row.get("distance") or 0.0
            base_score = 1.0 / (1.0 + distance)
            annotated.append(
                _annotate_row(
                    row,
                    api_map.get(row["id"], ""),
                    penalty=penalty,
                    apply_score=True,
                    base_score=base_score,
                )
            )
        annotated.sort(
            key=lambda r: r.get("v1_adjusted_score", 0.0),
            reverse=True,
        )
        return annotated
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def hybrid_search_entities(text: str, limit: int = 50) -> list[dict]:
    """Combine FTS5 and sqlite-vec semantic results with RRF.

    Falls back to FTS5-only search when sqlite-vec is unavailable.

    Applies the v1-entity penalty (BACK-20260509_1318-WarmOak): the RRF
    ``hybrid_score`` is used as the base score and multiplied by
    ``_v1_penalty()`` for v1-superseded entities. Results are re-ranked
    on the adjusted score. Each row carries ``v1_penalty_applied``,
    ``v1_successor_hint``, and ``v1_trace``.
    """
    _, db = _open()
    try:
        rows = index.hybrid_search_entities(db, text, limit=limit)
        if not rows:
            return rows
        penalty = _v1_penalty()
        api_map = _api_versions_for(db, [r["id"] for r in rows])
        annotated: list[dict] = []
        for row in rows:
            base_score = row.get("hybrid_score", 0.0)
            annotated.append(
                _annotate_row(
                    row,
                    api_map.get(row["id"], ""),
                    penalty=penalty,
                    apply_score=True,
                    base_score=base_score,
                )
            )
        annotated.sort(
            key=lambda r: r.get("v1_adjusted_score", 0.0),
            reverse=True,
        )
        return annotated
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_events(
    event_type: str | None = None,
    subject: str | None = None,
    actor: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """Query the LogEntry events table."""
    _, db = _open()
    try:
        return index.query_events(
            db,
            event_type=event_type,
            subject=subject,
            actor=actor,
            limit=limit,
        )
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_errors(limit: int = 100) -> list[dict]:
    """Return files that failed to parse during the last reindex."""
    _, db = _open()
    try:
        return index.list_errors(db, limit=limit)
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def stats() -> dict:
    """Return counts of indexed entities/events/errors."""
    _, db = _open()
    try:
        rows = {
            "entities": db.execute("SELECT COUNT(*) FROM entities").fetchone()[0],
            "events": db.execute("SELECT COUNT(*) FROM events").fetchone()[0],
            "errors": db.execute("SELECT COUNT(*) FROM errors").fetchone()[0],
        }
        return rows
    finally:
        db.close()


if __name__ == "__main__":
    server.run(transport="stdio")
