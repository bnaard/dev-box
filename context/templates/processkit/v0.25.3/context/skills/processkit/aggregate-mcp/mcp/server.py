#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
#   "httpx>=0.27",
#   "sqlite-vec>=0.1.0",
# ]
# ///
"""Aggregate processkit MCP server.

Compatibility MCP surface for the processkit gateway.

When the shared gateway registry internals are available, this module
uses them to discover and register tools while preserving the aggregate
server name and public tool names. Until then, it falls back to the
original eager per-skill MCP server importer. This avoids eager-starting
many stdio processes in clients such as Codex while keeping the
granular servers available for clients that prefer them.
"""
from __future__ import annotations

import importlib
import importlib.util
import os
import sys
from pathlib import Path
from typing import Any, Iterable


def _find_lib() -> Path:
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for c in (here / "src" / "lib", here / "_lib",
                  here / "context" / "skills" / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

server = FastMCP("processkit-aggregate-mcp")

_SELF_SKILL = "aggregate-mcp"
_EXCLUDED_SKILLS = {_SELF_SKILL, "processkit-gateway"}
_GATEWAY_REGISTRY_MODULE = "processkit.gateway.registry"
_GATEWAY_COLLECTOR = "collect_gateway_tools"
_TOOLS: list[dict[str, Any]] = []
_SOURCE_SERVER_COUNT = 0
_REGISTRY_BACKEND = "eager_import"
_CANONICAL_GATEWAY_AVAILABLE = False


def _skills_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if parent.name == "skills":
            return parent
    raise RuntimeError("skills root not found")


def _iter_server_paths() -> list[Path]:
    root = _skills_root()
    paths = []
    for candidate in root.glob("processkit/*/mcp/server.py"):
        if candidate.parents[1].name in _EXCLUDED_SKILLS:
            continue
        paths.append(candidate)
    return sorted(paths, key=lambda p: p.parents[1].name)


def _import_server(path: Path):
    skill = path.parents[1].name
    module_name = f"processkit_aggregate_{skill.replace('-', '_')}"
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot import MCP server at {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return skill, module


def _relative_server_path(path: Path) -> str:
    try:
        return path.relative_to(_skills_root()).as_posix()
    except ValueError:
        return path.as_posix()


def _annotation_metadata(annotations: Any) -> dict[str, Any] | None:
    if annotations is None:
        return None

    if hasattr(annotations, "model_dump"):
        data = annotations.model_dump(exclude_none=True)
    elif isinstance(annotations, dict):
        data = annotations
    else:
        data = {
            key: getattr(annotations, key)
            for key in (
                "title",
                "readOnlyHint",
                "destructiveHint",
                "idempotentHint",
                "openWorldHint",
            )
            if hasattr(annotations, key)
        }

    return {
        str(key): value
        for key, value in data.items()
        if isinstance(value, (str, int, float, bool, list, dict, type(None)))
    }


def _collect_source_tools() -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    for path in _iter_server_paths():
        skill, module = _import_server(path)
        source_server = getattr(module, "server", None)
        manager = getattr(source_server, "_tool_manager", None)
        tools = getattr(manager, "_tools", {}) if manager else {}
        for original_name, tool in sorted(tools.items()):
            entries.append({
                "source_skill": skill,
                "source_server_path": _relative_server_path(path),
                "source_tool": original_name,
                "tool": tool,
            })
    return entries


def _gateway_collector():
    try:
        registry = importlib.import_module(_GATEWAY_REGISTRY_MODULE)
    except ModuleNotFoundError as exc:
        if exc.name in {_GATEWAY_REGISTRY_MODULE, "processkit.gateway"}:
            return None
        raise
    return getattr(registry, _GATEWAY_COLLECTOR, None)


def _collect_registry_tools() -> tuple[list[dict[str, Any]], str, bool]:
    collector = _gateway_collector()
    if collector is None:
        return _collect_source_tools(), _REGISTRY_BACKEND, False

    entries = collector(
        skills_root=_skills_root(),
        exclude_skills=_EXCLUDED_SKILLS,
    )
    return list(entries), "gateway_registry", True


def _collision_groups(
    entries: Iterable[dict[str, Any]],
) -> dict[str, list[str]]:
    groups: dict[str, list[str]] = {}
    for entry in entries:
        source_tool = entry["source_tool"]
        groups.setdefault(source_tool, []).append(entry["source_skill"])
    return groups


def _registered_name(
    entry: dict[str, Any],
    used_names: set[str],
    duplicate_index: int,
) -> str:
    original_name = entry["source_tool"]
    if duplicate_index == 0 and original_name not in used_names:
        return original_name

    prefix = entry["source_skill"].replace("-", "_")
    candidate = f"{prefix}__{original_name}"
    if candidate not in used_names:
        return candidate

    suffix = 2
    while f"{candidate}_{suffix}" in used_names:
        suffix += 1
    return f"{candidate}_{suffix}"


def _register_tools() -> None:
    global _CANONICAL_GATEWAY_AVAILABLE, _REGISTRY_BACKEND
    global _SOURCE_SERVER_COUNT

    entries, _REGISTRY_BACKEND, _CANONICAL_GATEWAY_AVAILABLE = (
        _collect_registry_tools()
    )
    _SOURCE_SERVER_COUNT = len({
        entry["source_server_path"] for entry in entries
    })
    collision_groups = _collision_groups(entries)
    duplicate_seen: dict[str, int] = {}
    registered_names: set[str] = set()

    for entry in entries:
        original_name = entry["source_tool"]
        duplicate_index = duplicate_seen.get(original_name, 0)
        duplicate_seen[original_name] = duplicate_index + 1

        registered_name = _registered_name(
            entry,
            registered_names,
            duplicate_index,
        )
        registered_names.add(registered_name)

        tool = entry["tool"]
        annotations = getattr(tool, "annotations", None)
        description = getattr(tool, "description", None)
        server.tool(
            name=registered_name,
            description=description,
            annotations=annotations,
        )(tool.fn)

        collision_status = "unique"
        collision_sources = collision_groups[original_name]
        if len(collision_sources) > 1:
            collision_status = (
                "primary" if duplicate_index == 0 else "namespaced_duplicate"
            )

        _TOOLS.append({
            "name": registered_name,
            "source_skill": entry["source_skill"],
            "source_server_path": entry["source_server_path"],
            "source_tool": original_name,
            "annotations": _annotation_metadata(annotations),
            "collision_status": collision_status,
            "collision_sources": collision_sources,
            "deduplicated": registered_name != original_name,
        })


_register_tools()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_aggregate_tools() -> dict:
    """List tools exposed by the aggregate processkit MCP server."""
    return {
        "count": len(_TOOLS),
        "source_server_count": _SOURCE_SERVER_COUNT,
        "runtime": {
            "import_mode": (
                "gateway_registry"
                if _CANONICAL_GATEWAY_AVAILABLE
                else "eager"
            ),
            "lazy_daemon": False,
            "registry_backend": _REGISTRY_BACKEND,
            "canonical_gateway_available": _CANONICAL_GATEWAY_AVAILABLE,
        },
        "tools": _TOOLS,
    }


if __name__ == "__main__":
    server.run()
