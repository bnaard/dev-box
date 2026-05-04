#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit note-management MCP server."""
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

from processkit import config, entity, ids, index, log, paths, schema, state_machine  # noqa: E402

server = FastMCP("processkit-note-management")

_VALID_NOTE_TYPES = {"fleeting", "insight", "reference", "question"}
_VALID_INJECTION_MODES = {"interrupt", "ambient", "next-cycle"}
_HOOK_INBOX_STATES = ("inbox", "claimed", "done", "failed")


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _review_due(days: int = 7) -> str:
    return (_dt.date.today() + _dt.timedelta(days=days)).isoformat()


def _load_note(root: Path, id: str) -> entity.Entity | None:
    note_dir = paths.context_dir("Note", root)
    candidate = note_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="Note")
        if row and row.get("path"):
            return entity.load(row["path"])
    finally:
        db.close()
    return None


def _write_and_index(ent: entity.Entity) -> None:
    ent.write()
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()


def _safe_relative_dir(value: str) -> Path:
    path = Path(value)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError("base_dir must be a relative path without '..'")
    return path


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def prepare_hook_inbox_dirs(base_dir: str = "tasks") -> dict:
    """Create the canonical hook-adapter drop directories.

    Creates ``tasks/inbox/``, ``tasks/claimed/``, ``tasks/done/``, and
    ``tasks/failed/`` by default. Drop adapters can use this layout as
    their hand-off boundary before calling ``capture_inbox_item``.
    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    try:
        rel_base = _safe_relative_dir(base_dir)
    except ValueError as exc:
        return {"error": str(exc)}
    root = paths.find_project_root()
    created: dict[str, str] = {}
    for state in _HOOK_INBOX_STATES:
        target = root / rel_base / state
        target.mkdir(parents=True, exist_ok=True)
        created[state] = str(target)
    return {"ok": True, "base_dir": str(root / rel_base), "dirs": created}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def create_note(
    title: str,
    body: str,
    type: str = "fleeting",
    tags: list[str] | None = None,
    source: str | None = None,
    review_due: str | None = None,
    slug_summary: str | None = None,
) -> dict:
    """Create a Note entity."""
    if type not in _VALID_NOTE_TYPES:
        return {"error": f"invalid type {type!r}; must be one of {sorted(_VALID_NOTE_TYPES)}"}
    summary_errors = ids.validate_slug_summary(slug_summary)
    if summary_errors:
        return {"error": "invalid slug_summary", "details": summary_errors}

    root = paths.find_project_root()
    cfg = config.load_config(root)
    db = index.open_db()
    try:
        existing = index.existing_ids(db, "Note")
    finally:
        db.close()

    new_id = ids.generate_id(
        "Note",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=(slug_summary or title) if cfg.id_slug else None,
        existing=existing,
    )
    try:
        initial_state = state_machine.load("note").initial
    except state_machine.StateMachineError:
        initial_state = "captured"

    spec: dict = {
        "title": title,
        "body": body,
        "type": type,
        "state": initial_state,
        "review_due": review_due or _review_due(),
    }
    if tags:
        spec["tags"] = tags
    if source:
        spec["source"] = source

    errors = schema.validate_spec("Note", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("Note", new_id, spec, body=body)
    target = paths.entity_path("Note", new_id, None, root)
    ent.write(target)
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Note", new_id, "note.created",
        f"Created Note {new_id!r}: {title!r}",
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
def capture_inbox_item(
    title: str,
    body: str,
    injection_mode: str = "ambient",
    channel: str | None = None,
    source: str | None = None,
    target_workitem: str | None = None,
    tags: list[str] | None = None,
) -> dict:
    """Capture a hook-inbox item as a Note."""
    if injection_mode not in _VALID_INJECTION_MODES:
        return {
            "error": (
                f"invalid injection_mode {injection_mode!r}; must be one of "
                f"{sorted(_VALID_INJECTION_MODES)}"
            )
        }
    created = create_note(
        title=title,
        body=body,
        type="fleeting",
        tags=tags,
        source=source,
        slug_summary=title,
    )
    if "error" in created:
        return created

    root = paths.find_project_root()
    ent = _load_note(root, created["id"])
    if ent is None:
        return {"error": f"created note {created['id']!r} could not be reloaded"}
    ent.spec["inbox"] = {
        "status": "captured",
        "injection_mode": injection_mode,
        "channel": channel,
        "captured_at": _now_iso(),
    }
    if target_workitem:
        ent.spec["inbox"]["target_workitem"] = target_workitem
    _write_and_index(ent)

    log.log_side_effect(
        "Note", ent.id, "inbox.captured",
        f"Captured inbox item {ent.id!r} as {injection_mode!r}",
        root=root,
        actor=ent.id,
        details={"injection_mode": injection_mode, "target_workitem": target_workitem},
    )
    return {"id": ent.id, "path": str(ent.path), "inbox": ent.spec["inbox"]}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def claim_inbox_item(id: str, actor: str) -> dict:
    """Mark a captured inbox item as claimed by an actor."""
    root = paths.find_project_root()
    ent = _load_note(root, id)
    if ent is None:
        return {"error": f"note {id!r} not found"}
    inbox = ent.spec.setdefault("inbox", {})
    inbox["status"] = "claimed"
    inbox["claimed_by"] = actor
    inbox["claimed_at"] = _now_iso()
    _write_and_index(ent)
    log.log_side_effect(
        "Note", ent.id, "inbox.claimed",
        f"Claimed inbox item {ent.id!r}",
        root=root,
        actor=actor,
    )
    return {"ok": True, "id": ent.id, "inbox": inbox}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def complete_inbox_item(id: str, actor: str, result: dict | None = None) -> dict:
    """Mark a hook-inbox Note as completed."""
    root = paths.find_project_root()
    ent = _load_note(root, id)
    if ent is None:
        return {"error": f"note {id!r} not found"}
    inbox = ent.spec.setdefault("inbox", {})
    inbox["status"] = "completed"
    inbox["completed_at"] = _now_iso()
    if result:
        inbox["result"] = result
    _write_and_index(ent)
    log.log_side_effect(
        "Note", ent.id, "inbox.completed",
        f"Completed inbox item {ent.id!r}",
        root=root,
        actor=actor,
        details=result,
    )
    return {"ok": True, "id": ent.id, "inbox": inbox}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def fail_inbox_item(id: str, actor: str, error: str) -> dict:
    """Mark a hook-inbox Note as failed."""
    root = paths.find_project_root()
    ent = _load_note(root, id)
    if ent is None:
        return {"error": f"note {id!r} not found"}
    inbox = ent.spec.setdefault("inbox", {})
    inbox["status"] = "failed"
    inbox["failed_at"] = _now_iso()
    inbox["error"] = error
    _write_and_index(ent)
    log.log_side_effect(
        "Note", ent.id, "inbox.failed",
        f"Failed inbox item {ent.id!r}: {error}",
        root=root,
        actor=actor,
    )
    return {"ok": True, "id": ent.id, "inbox": inbox}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def reload_schemas() -> dict:
    """Clear this server's in-process schema + state-machine caches."""
    return {"ok": True, "cleared": schema.reload_caches()}


if __name__ == "__main__":
    server.run(transport="stdio")
