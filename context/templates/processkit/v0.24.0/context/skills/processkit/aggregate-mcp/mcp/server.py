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

Imports every per-skill processkit MCP server and re-exports its tools
through one FastMCP instance. This avoids eager-starting many stdio
processes in clients such as Codex while keeping the granular servers
available for clients that prefer them.
"""
from __future__ import annotations

import importlib.util
import os
import sys
from pathlib import Path
from typing import Any


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
_TOOLS: list[dict[str, Any]] = []


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
        if candidate.parents[1].name == _SELF_SKILL:
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


def _register_tools() -> None:
    seen: set[str] = set()
    for path in _iter_server_paths():
        skill, module = _import_server(path)
        source_server = getattr(module, "server", None)
        manager = getattr(source_server, "_tool_manager", None)
        tools = getattr(manager, "_tools", {}) if manager else {}
        for original_name, tool in sorted(tools.items()):
            registered_name = original_name
            if registered_name in seen:
                registered_name = f"{skill.replace('-', '_')}__{original_name}"
            seen.add(registered_name)

            annotations = getattr(tool, "annotations", None)
            description = getattr(tool, "description", None)
            server.tool(
                name=registered_name,
                description=description,
                annotations=annotations,
            )(tool.fn)
            _TOOLS.append({
                "name": registered_name,
                "source_skill": skill,
                "source_tool": original_name,
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
    return {"count": len(_TOOLS), "tools": _TOOLS}


if __name__ == "__main__":
    server.run()
