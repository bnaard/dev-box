#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit workitem-management MCP server.

Tools:

    create_workitem(title, type?, priority?, assignee?, description?,
                    parent?, scope?, labels?)
        -> {id, path, state}

    transition_workitem(id, to_state, note?)
        -> {ok, from_state, to_state}

    query_workitems(state?, type?, assignee?, limit?)
        -> [workitems]

    link_workitems(from_id, to_id, relation)
        -> {ok}

    get_workitem(id)
        -> workitem | null
"""
from __future__ import annotations

import datetime as _dt
import os
import sys
from pathlib import Path


def _find_lib() -> Path:
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for c in (here / "src" / "lib", here / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import config, entity, ids, index, paths, log, schema, state_machine  # noqa: E402

server = FastMCP("processkit-workitem-management")

_VALID_PRIORITIES = {"critical", "high", "medium", "low"}
_VALID_RELATIONS = {"blocks", "blocked_by", "parent", "children", "related_decisions"}


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _load_workitem(root: Path, id: str) -> entity.Entity | None:
    wi_dir = paths.context_dir("WorkItem", root)
    candidate = wi_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    # Try the index; resolve_entity handles partial IDs and word-pair lookups.
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="WorkItem")
        if row and row.get("path"):
            return entity.load(row["path"])
    finally:
        db.close()
    return None


def _write_index_and_archive_if_terminal(
    ent: entity.Entity,
    *,
    root: Path,
    to_state: str,
) -> tuple[Path, bool]:
    old_path = ent.path
    target = paths.entity_path(
        "WorkItem",
        ent.id,
        str(ent.metadata.get("created") or ""),
        root,
        state=to_state,
    )
    archived = old_path is not None and old_path.resolve() != target.resolve()
    if archived:
        ent.write(target)
        if old_path and old_path.exists():
            old_path.unlink()
    else:
        ent.write()

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()
    return ent.path or target, archived


def _valid_types() -> set[str]:
    values = schema.known_values("WorkItem", "known_types")
    if values:
        return set(values)
    return {"task", "story", "bug", "epic", "spike", "chore"}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def create_workitem(
    title: str,
    type: str = "task",
    priority: str = "medium",
    assignee: str | None = None,
    description: str | None = None,
    parent: str | None = None,
    scope: str | None = None,
    labels: dict | None = None,
    slug_summary: str | None = None,
) -> dict:
    """Create a new WorkItem in the project's context/workitems/ directory.

    Validates ``type``, ``priority``, and the initial state from the
    workitem state machine. Returns ``{id, path, state}``. Prerequisite:
    call find_skill(task_description) or confirm you are already
    operating within a named processkit skill before using this tool.
    1% rule: call route_task first; commit in the same turn — deferred writes are dropped.
    """
    valid_types = _valid_types()
    if type not in valid_types:
        return {"error": f"invalid type {type!r}; must be one of {sorted(valid_types)}"}
    if priority not in _VALID_PRIORITIES:
        return {"error": f"invalid priority {priority!r}"}
    summary_errors = ids.validate_slug_summary(slug_summary)
    if summary_errors:
        return {"error": "invalid slug_summary", "details": summary_errors}

    root = paths.find_project_root()
    cfg = config.load_config(root)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "WorkItem")
    finally:
        db.close()

    new_id = ids.generate_id(
        "WorkItem",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=(slug_summary or title) if cfg.id_slug else None,
        existing=existing,
    )

    initial_state = "backlog"
    try:
        sm = state_machine.load("workitem")
        initial_state = sm.initial
    except state_machine.StateMachineError:
        pass

    spec = {
        "title": title,
        "state": initial_state,
        "type": type,
        "priority": priority,
    }
    if assignee:
        spec["assignee"] = assignee
    if description:
        spec["description"] = description
    if parent:
        spec["parent"] = parent
    if scope:
        spec["scope"] = scope

    errors = schema.validate_spec("WorkItem", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("WorkItem", new_id, spec, labels=labels)
    target = paths.entity_path("WorkItem", new_id, None, root)
    ent.write(target)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "WorkItem", new_id, "workitem.created",
        f"Created WorkItem {new_id!r}: {title!r}",
        root=root,
        actor=new_id,
    )
    return {"id": new_id, "path": str(target), "state": initial_state}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def create_process_instance(
    title: str,
    process_definition_artifact: str,
    steps: list[str],
    priority: str = "medium",
    scope: str | None = None,
    assignee: str | None = None,
    description: str | None = None,
    slug_summary: str | None = None,
) -> dict:
    """Create a process-instance WorkItem and child process-step items.

    ``process_definition_artifact`` must reference the Artifact that
    describes the reusable process. ``steps`` are created as ordered
    child WorkItems. Prerequisite: call find_skill(task_description) or
    confirm you are already operating within a named processkit skill
    before using this tool. 1% rule: call route_task first; commit in
    the same turn — deferred writes are dropped.
    """
    if not process_definition_artifact.startswith("ART-"):
        return {"error": "process_definition_artifact must be an ART-* id"}
    if not steps:
        return {"error": "steps must contain at least one process step"}

    created_parent = create_workitem(
        title=title,
        type="process-instance",
        priority=priority,
        assignee=assignee,
        description=description,
        scope=scope,
        slug_summary=slug_summary or title,
    )
    if "error" in created_parent:
        return created_parent

    root = paths.find_project_root()
    parent = _load_workitem(root, created_parent["id"])
    if parent is None:
        return {"error": f"created parent {created_parent['id']!r} could not be reloaded"}
    parent.spec["process_definition_artifact"] = process_definition_artifact

    children: list[str] = []
    child_results: list[dict] = []
    for idx, step_title in enumerate(steps, start=1):
        created_step = create_workitem(
            title=step_title,
            type="process-step",
            priority=priority,
            assignee=assignee,
            parent=parent.id,
            scope=scope,
            slug_summary=step_title,
        )
        if "error" in created_step:
            return {
                "error": "failed to create process step",
                "details": created_step,
                "created_parent": created_parent,
                "created_steps": child_results,
            }
        child = _load_workitem(root, created_step["id"])
        if child is not None:
            child.spec["process_instance"] = parent.id
            child.spec["step_order"] = idx
            child.spec["process_definition_artifact"] = process_definition_artifact
            child.write()
            db = index.open_db()
            try:
                index.upsert_entity(db, child)
            finally:
                db.close()
        children.append(created_step["id"])
        child_results.append(created_step)

    parent.spec["children"] = children
    parent.write()
    db = index.open_db()
    try:
        index.upsert_entity(db, parent)
    finally:
        db.close()

    log.log_side_effect(
        "WorkItem", parent.id, "process.instance.created",
        f"Created process instance {parent.id!r} from {process_definition_artifact!r}",
        root=root,
        actor=parent.id,
        details={"steps": children, "process_definition_artifact": process_definition_artifact},
    )
    return {
        "id": parent.id,
        "path": str(parent.path),
        "state": parent.spec.get("state"),
        "children": child_results,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def create_sep_handoff(
    title: str,
    source_actor: str,
    target: str,
    payload: dict,
    priority: str = "high",
    scope: str | None = None,
    description: str | None = None,
) -> dict:
    """Create a human-supervision SEP handoff WorkItem.

    The resulting WorkItem has ``type=sep-handoff`` and stores the
    source actor, target, and structured payload in spec. Prerequisite:
    call find_skill(task_description) or confirm you are already
    operating within a named processkit skill before using this tool.
    1% rule: call route_task first; commit in the same turn — deferred
    writes are dropped.
    """
    if not isinstance(payload, dict) or not payload:
        return {"error": "payload must be a non-empty object"}
    created = create_workitem(
        title=title,
        type="sep-handoff",
        priority=priority,
        description=description,
        scope=scope,
        slug_summary=title,
    )
    if "error" in created:
        return created

    root = paths.find_project_root()
    ent = _load_workitem(root, created["id"])
    if ent is None:
        return {"error": f"created handoff {created['id']!r} could not be reloaded"}
    ent.spec["source_actor"] = source_actor
    ent.spec["target"] = target
    ent.spec["handoff_payload"] = payload
    ent.write()
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "WorkItem", ent.id, "sep.handoff.created",
        f"Created SEP handoff {ent.id!r} for {target!r}",
        root=root,
        actor=source_actor,
        details={"target": target},
    )
    return {"id": ent.id, "path": str(ent.path), "state": ent.spec.get("state")}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def transition_workitem(id: str, to_state: str, note: str | None = None) -> dict:
    """Transition a WorkItem to a new state.

    Validates the transition against the workitem state machine. Sets
    ``spec.started_at`` on first entry to ``in-progress`` and
    ``spec.completed_at`` on entering a terminal state. Prerequisite:
    call find_skill(task_description) or confirm you are already
    operating within a named processkit skill before using this tool.
    1% rule: call route_task first; commit in the same turn — deferred writes are dropped.
    """
    root = paths.find_project_root()
    ent = _load_workitem(root, id)
    if ent is None:
        return {"error": f"workitem {id!r} not found"}
    from_state = ent.spec.get("state")
    try:
        state_machine.validate_transition("workitem", from_state, to_state)
    except state_machine.StateMachineError as e:
        return {"error": str(e)}

    ent.spec["state"] = to_state
    if to_state == "in-progress" and not ent.spec.get("started_at"):
        ent.spec["started_at"] = _now_iso()
    try:
        sm = state_machine.load("workitem")
        if sm.is_terminal(to_state):
            ent.spec["completed_at"] = _now_iso()
    except state_machine.StateMachineError:
        pass

    if note:
        ent.body = (ent.body or "") + f"\n\n## Transition note ({_now_iso()})\n\n{note}\n"

    try:
        sm = state_machine.load("workitem")
        should_archive = sm.is_terminal(to_state)
    except state_machine.StateMachineError:
        should_archive = False

    if should_archive:
        final_path, archived = _write_index_and_archive_if_terminal(
            ent,
            root=root,
            to_state=to_state,
        )
    else:
        ent.write()
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
        final_path = ent.path
        archived = False

    log.log_side_effect(
        "WorkItem", id, "workitem.transitioned",
        f"Transitioned WorkItem {id!r} from {from_state!r} to {to_state!r}",
        root=root,
        actor=id,
    )
    if archived:
        log.log_side_effect(
            "WorkItem", id, "workitem.archive-moved",
            f"Archived terminal WorkItem {id!r} to {final_path}",
            root=root,
            actor=id,
            details={"path": str(final_path)},
        )
    return {
        "ok": True,
        "id": id,
        "from_state": from_state,
        "to_state": to_state,
        "path": str(final_path) if final_path else None,
        "archived": archived,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_workitems(
    state: str | None = None,
    type: str | None = None,
    assignee: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """Query WorkItems via the index. Filters: state, type, assignee."""
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="WorkItem", state=state, limit=limit * 4)
    finally:
        db.close()
    out = []
    for r in rows:
        full = get_workitem(r["id"])
        if full is None or "error" in full:
            continue
        if type and full.get("type") != type:
            continue
        if assignee and full.get("assignee") != assignee:
            continue
        out.append(full)
        if len(out) >= limit:
            break
    return out


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_workitem(id: str) -> dict:
    """Fetch a WorkItem by ID with its full spec.

    Accepts a full ID, a prefix (missing slug), or a bare word-pair.
    Returns ``{"error": "..."}`` if not found or ambiguous.
    """
    db = index.open_db()
    try:
        row, candidates = index.resolve_entity(db, id, kind="WorkItem")
    finally:
        db.close()
    if candidates:
        return {"error": f"ambiguous ID {id!r}; candidates: {candidates}"}
    if row is None:
        return {"error": f"WorkItem not found: {id!r}"}
    spec = row.get("spec", {})
    return {
        "id": row["id"],
        "title": row.get("title"),
        "state": row.get("state"),
        "type": spec.get("type"),
        "priority": spec.get("priority"),
        "assignee": spec.get("assignee"),
        "scope": spec.get("scope"),
        "parent": spec.get("parent"),
        "blocks": spec.get("blocks", []),
        "blocked_by": spec.get("blocked_by", []),
        "related_decisions": spec.get("related_decisions", []),
        "path": row.get("path"),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def link_workitems(from_id: str, to_id: str, relation: str) -> dict:
    """Add a typed cross-reference between two WorkItems.

    ``relation`` must be one of: blocks, blocked_by, parent, children,
    related_decisions. Bidirectional relations (blocks/blocked_by,
    parent/children) are NOT auto-mirrored — call link_workitems on
    both sides if you want them.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    if relation not in _VALID_RELATIONS:
        return {"error": f"invalid relation {relation!r}; valid: {sorted(_VALID_RELATIONS)}"}
    root = paths.find_project_root()
    ent = _load_workitem(root, from_id)
    if ent is None:
        return {"error": f"workitem {from_id!r} not found"}
    if relation == "parent":
        ent.spec["parent"] = to_id
    else:
        existing = ent.spec.get(relation, []) or []
        if to_id not in existing:
            existing.append(to_id)
        ent.spec[relation] = existing
    ent.write()

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()
    return {"ok": True, "from": from_id, "to": to_id, "relation": relation}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def reload_schemas() -> dict:
    """Clear this server's in-process schema + state-machine caches.

    After a schema or state-machine file on disk is edited, call this
    tool to make the next request re-read from disk. Returns
    ``{"ok": True, "cleared": {"schemas": N, "state_machines": M}}``
    where N/M are the number of cache entries that were holding data
    before the clear.

    Scope: clears caches in THIS server process only. Each MCP server
    runs as a separate child process under the harness, so a schema
    edit that affects multiple servers requires calling
    `reload_schemas` on each.

    Does NOT address PEP 723 dep-header edits — those require a full
    harness restart because the uv-resolved venv is pinned at process
    start.
    """
    return {"ok": True, "cleared": schema.reload_caches()}


if __name__ == "__main__":
    server.run(transport="stdio")
