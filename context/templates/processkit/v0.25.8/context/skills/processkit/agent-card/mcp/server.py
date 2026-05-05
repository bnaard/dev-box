#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""processkit agent-card MCP server."""
from __future__ import annotations

import hashlib
import json
import os
import sys
from pathlib import Path
from typing import Any

import yaml


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

from processkit import entity, index, log, paths  # noqa: E402

server = FastMCP("processkit-agent-card")


def _load_entity(id: str, kind: str) -> entity.Entity | None:
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind=kind)
    finally:
        db.close()
    if row and row.get("path"):
        return entity.load(row["path"])
    return None


def _parse_body(body: str) -> dict[str, Any]:
    text = (body or "").strip()
    if not text:
        return {}
    try:
        data = json.loads(text)
    except json.JSONDecodeError:
        data = yaml.safe_load(text)
    return data if isinstance(data, dict) else {}


def _project_path(root: Path, output_path: str) -> Path:
    path = Path(output_path)
    if not path.is_absolute():
        path = root / path
    resolved = path.resolve()
    if root.resolve() not in resolved.parents and resolved != root.resolve():
        raise ValueError("output_path must stay inside the project root")
    return resolved


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def project_agent_card(
    artifact_id: str,
    actor_id: str | None = None,
    output_path: str = ".well-known/agent-card.json",
    write: bool = True,
) -> dict:
    """Project an agent-card Artifact into canonical JSON."""
    root = paths.find_project_root()
    artifact = _load_entity(artifact_id, "Artifact")
    if artifact is None:
        return {"error": f"Artifact not found: {artifact_id!r}"}
    if artifact.spec.get("kind") != "agent-card":
        return {"error": f"Artifact {artifact.id!r} is not kind='agent-card'"}

    card = artifact.spec.get("card")
    if not isinstance(card, dict):
        card = _parse_body(artifact.body)
    if not isinstance(card, dict) or not card:
        return {"error": "agent-card Artifact must provide spec.card or YAML/JSON body"}

    card = dict(card)
    card.setdefault("id", artifact.id)
    card.setdefault("name", artifact.spec.get("name") or artifact.id)

    if actor_id:
        actor = _load_entity(actor_id, "Actor")
        if actor is None:
            return {"error": f"Actor not found: {actor_id!r}"}
        endpoints = actor.spec.get("endpoints")
        if endpoints:
            card.setdefault("interfaces", endpoints)
        card.setdefault("actor_id", actor.id)

    payload = json.dumps(card, sort_keys=True, indent=2) + "\n"
    checksum = hashlib.sha256(payload.encode("utf-8")).hexdigest()
    projected_path = _project_path(root, output_path)

    if write:
        projected_path.parent.mkdir(parents=True, exist_ok=True)
        projected_path.write_text(payload, encoding="utf-8")
        log.log_side_effect(
            "Artifact", artifact.id, "agent_card.projected",
            f"Projected agent card {artifact.id!r} to {output_path!r}",
            root=root,
            actor=actor_id or artifact.id,
            details={"output_path": output_path, "sha256": checksum},
        )

    return {
        "card": card,
        "checksum": checksum,
        "output_path": str(projected_path),
        "written": bool(write),
    }


if __name__ == "__main__":
    server.run(transport="stdio")
