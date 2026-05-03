#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
# ]
# ///
"""Context-archiving MCP server."""
from __future__ import annotations

import datetime as dt
import hashlib
import json
import os
import sys
import tarfile
from pathlib import Path
from typing import Optional

from mcp.server.fastmcp import FastMCP
from mcp.types import ToolAnnotations


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

from processkit import entity as _entity  # noqa: E402
from processkit import index as _index  # noqa: E402
from processkit import log as _log  # noqa: E402
from processkit import paths as _paths  # noqa: E402


server = FastMCP("processkit-context-archiving")

TERMINAL_STATES = {"done", "cancelled", "accepted", "superseded", "rejected"}
ACTIVE_STATES = {"backlog", "in-progress", "blocked", "review", "proposed", "active"}


def _project_root() -> Path:
    return _paths.find_project_root()


def _parse_date(value: str | None) -> dt.datetime | None:
    if not value:
        return None
    text = str(value).replace("Z", "+00:00")
    try:
        parsed = dt.datetime.fromisoformat(text)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=dt.timezone.utc)
    return parsed


def _rel_to_root(path: str | Path, root: Path) -> str:
    p = Path(path)
    try:
        return str(p.resolve().relative_to(root.resolve()))
    except Exception:
        return str(p)


def _archive_paths(root: Path, archive_id: str) -> tuple[Path, Path]:
    now = dt.datetime.now(dt.timezone.utc)
    base = root / "context" / "archives" / f"{now:%Y}" / f"{now:%m}"
    return base / f"{archive_id}.tar.gz", base / f"{archive_id}.json"


def _candidate_rows(
    *,
    kind: Optional[str],
    state: Optional[str],
    older_than_days: int,
    limit: int,
) -> list[dict]:
    if older_than_days < 0:
        raise ValueError("older_than_days must be non-negative")
    if state in ACTIVE_STATES:
        raise ValueError(
            f"state {state!r} is active and must not be archived by "
            "the default policy"
        )
    cutoff = dt.datetime.now(dt.timezone.utc) - dt.timedelta(days=older_than_days)
    db = _index.open_db()
    try:
        rows = _index.query_entities(db, kind=kind, state=state, limit=1000)
    finally:
        db.close()

    candidates: list[dict] = []
    for row in rows:
        location = str(row.get("storage_location") or "")
        if location and location != "live":
            continue
        row_state = row.get("state")
        if row_state in ACTIVE_STATES:
            continue
        created = _parse_date(row.get("created"))
        updated = _parse_date(row.get("updated")) or created
        timestamp = updated or created
        if timestamp is None or timestamp > cutoff:
            continue
        path = str(row.get("path") or "")
        if "/context/templates/" in path or path.startswith("context/templates/"):
            continue
        candidates.append(dict(row))
        if len(candidates) >= limit:
            break
    return candidates


def _archive_id(kind: Optional[str], state: Optional[str]) -> str:
    now = dt.datetime.now(dt.timezone.utc)
    parts = ["ARCHIVE", f"{now:%Y%m%d_%H%M%S}"]
    if kind:
        parts.append(kind.lower())
    if state:
        parts.append(state.replace("-", "_"))
    return "-".join(parts)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def describe_archive_policy() -> dict:
    """Return the default hot/warm/cold context archive policy."""
    return {
        "version": "0.2.0",
        "write_path_enabled": True,
        "terminal_states": sorted(TERMINAL_STATES),
        "active_states_never_archived": sorted(ACTIVE_STATES),
        "defaults": {
            "WorkItem": {"states": ["done", "cancelled"], "older_than_days": 30},
            "DecisionRecord": {
                "states": ["accepted", "superseded", "rejected"],
                "older_than_days": 90,
            },
            "LogEntry": {"older_than_days": 90, "group_by": "month"},
        },
        "note": (
            "create_archive() writes a tar.gz plus manifest, removes hot "
            "payload files, reindexes archive manifests, and logs an event."
        ),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def plan_archive(
    kind: Optional[str] = None,
    state: Optional[str] = "done",
    older_than_days: int = 30,
    limit: int = 50,
) -> dict:
    """Return archive candidates from the live index without moving files."""
    try:
        candidates = _candidate_rows(
            kind=kind,
            state=state,
            older_than_days=older_than_days,
            limit=limit,
        )
    except ValueError as e:
        return {"error": str(e)}

    return {
        "write_path_enabled": True,
        "kind": kind,
        "state": state,
        "older_than_days": older_than_days,
        "candidate_count": len(candidates),
        "candidates": candidates,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=False,
    openWorldHint=False,
))
def create_archive(
    kind: Optional[str] = None,
    state: Optional[str] = "done",
    older_than_days: int = 30,
    limit: int = 50,
    dry_run: bool = True,
) -> dict:
    """Package archive candidates into cold storage and remove hot files."""
    root = _project_root()
    try:
        candidates = _candidate_rows(
            kind=kind,
            state=state,
            older_than_days=older_than_days,
            limit=limit,
        )
    except ValueError as e:
        return {"error": str(e)}
    if dry_run:
        return {
            "dry_run": True,
            "candidate_count": len(candidates),
            "candidates": candidates,
        }
    if not candidates:
        return {"ok": True, "archived": 0, "message": "no candidates"}

    archive_id = _archive_id(kind, state)
    tar_path, manifest_path = _archive_paths(root, archive_id)
    tar_path.parent.mkdir(parents=True, exist_ok=True)

    manifest_entities: list[dict] = []
    with tarfile.open(tar_path, "w:gz") as tar:
        for row in candidates:
            path = Path(str(row.get("path") or ""))
            if not path.is_absolute():
                path = root / path
            if not path.is_file():
                continue
            ent = _entity.load(path)
            member = _rel_to_root(path, root)
            text = path.read_text(encoding="utf-8")
            digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
            tar.add(path, arcname=member)
            storage_location = f"{_rel_to_root(tar_path, root)}::{member}"
            manifest_entities.append({
                "id": ent.id,
                "kind": ent.kind,
                "apiVersion": ent.apiVersion,
                "metadata": ent.metadata,
                "spec": ent.spec,
                "labels": ent.labels,
                "original_path": member,
                "member_path": member,
                "storage_location": storage_location,
                "sha256": digest,
                "body_index": ent.body[:1200],
            })

    if not manifest_entities:
        tar_path.unlink(missing_ok=True)
        return {"ok": True, "archived": 0, "message": "no readable candidates"}

    tar_digest = hashlib.sha256(tar_path.read_bytes()).hexdigest()
    manifest = {
        "archive_id": archive_id,
        "created_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "archive_path": _rel_to_root(tar_path, root),
        "sha256": tar_digest,
        "kind": kind,
        "state": state,
        "older_than_days": older_than_days,
        "entities": manifest_entities,
    }
    manifest_path.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    removed: list[str] = []
    for item in manifest_entities:
        p = root / item["original_path"]
        if p.is_file():
            p.unlink()
            removed.append(item["original_path"])

    db = _index.open_db()
    try:
        stats = _index.reindex(root, db)
    finally:
        db.close()

    log_id = _log.log_side_effect(
        "Archive",
        archive_id,
        "context_archive.created",
        f"Archived {len(removed)} context entities into {archive_id}",
        root=root,
        actor="processkit-context-archiving",
        details={
            "archive_path": manifest["archive_path"],
            "manifest_path": _rel_to_root(manifest_path, root),
            "entity_ids": [item["id"] for item in manifest_entities],
        },
    )
    return {
        "ok": True,
        "archive_id": archive_id,
        "archive_path": manifest["archive_path"],
        "manifest_path": _rel_to_root(manifest_path, root),
        "sha256": tar_digest,
        "archived": len(removed),
        "removed": removed,
        "log_id": log_id,
        "reindex": {
            "entities": stats.entities,
            "events": stats.events,
            "errors": stats.errors,
        },
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def extract_archive_payload(entity_id: str) -> dict:
    """Return the archived Markdown payload for an entity."""
    root = _project_root()
    db = _index.open_db()
    try:
        row, candidates = _index.resolve_entity(db, entity_id)
    finally:
        db.close()
    if candidates:
        return {"error": f"ambiguous ID {entity_id!r}; candidates: {candidates}"}
    if not row:
        return {"error": f"entity not found: {entity_id!r}"}
    location = row.get("storage_location")
    if not location:
        return {"error": f"entity {row['id']!r} is not archived"}
    if "::" not in location:
        return {"error": f"unsupported storage_location: {location!r}"}
    archive_rel, member = location.split("::", 1)
    archive_path = root / archive_rel
    if not archive_path.is_file():
        return {"error": f"archive file not found: {archive_rel}"}
    with tarfile.open(archive_path, "r:gz") as tar:
        extracted = tar.extractfile(member)
        if extracted is None:
            return {"error": f"member not found in archive: {member}"}
        text = extracted.read().decode("utf-8")
    return {
        "id": row["id"],
        "archive_path": archive_rel,
        "member_path": member,
        "text": text,
    }


if __name__ == "__main__":
    server.run(transport="stdio")
