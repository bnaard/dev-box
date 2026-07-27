"""Gateway tool catalog serialization.

The gateway can run in eager mode by importing every backing MCP server and
registering the source tools directly. Daemon mode needs a cheaper startup
path: list tool metadata from a catalog, then import the owning source server
only when a tool is called.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from mcp.types import ToolAnnotations


CATALOG_VERSION = 1


def default_catalog_path(skills_root: Path) -> Path:
    """Return the processkit-gateway catalog path for ``skills_root``."""
    return (
        skills_root
        / "processkit"
        / "processkit-gateway"
        / "mcp"
        / "tool-catalog.json"
    )


def annotations_from_metadata(data: dict[str, Any] | None) -> ToolAnnotations | None:
    """Hydrate serialized MCP annotations."""
    if not data:
        return None
    return ToolAnnotations(**data)


def catalog_payload(tools: list[dict[str, Any]], source_server_count: int) -> dict:
    """Return a JSON-safe catalog payload from registry tool metadata."""
    catalog_tools: list[dict[str, Any]] = []
    for item in tools:
        tool = item["tool"]
        catalog_tools.append({
            "name": item["name"],
            "title": getattr(tool, "title", None),
            "description": getattr(tool, "description", None) or "",
            "parameters": getattr(tool, "parameters", None) or {
                "type": "object",
                "properties": {},
            },
            "output_schema": getattr(tool, "output_schema", None),
            "source_skill": item["source_skill"],
            "source_server_path": item["source_server_path"],
            "source_tool": item["source_tool"],
            "annotations": item["annotations"],
            "permission_class": item["permission_class"],
            "collision_status": item["collision_status"],
            "collision_sources": item["collision_sources"],
            "deduplicated": item["deduplicated"],
        })

    return {
        "catalog_version": CATALOG_VERSION,
        "source_server_count": source_server_count,
        "tools": catalog_tools,
    }


def read_catalog(path: Path) -> dict:
    """Read and validate a gateway catalog."""
    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("catalog_version") != CATALOG_VERSION:
        raise ValueError(
            f"unsupported gateway catalog version: "
            f"{payload.get('catalog_version')!r}"
        )
    tools = payload.get("tools")
    if not isinstance(tools, list):
        raise ValueError("gateway catalog missing tools list")
    return payload


def write_catalog(path: Path, payload: dict) -> None:
    """Write ``payload`` as stable formatted JSON."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
