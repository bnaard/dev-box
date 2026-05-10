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

Two import modes are available:

- ``eager`` (default): every per-skill MCP server is imported at
  startup. Backward-compatible behaviour.
- ``lazy_catalog``: the cached gateway tool catalog is loaded at
  startup, tool surface is announced from catalog metadata, and the
  owning per-skill module is imported only when a tool is first
  called. Opt in with ``PROCESSKIT_MCP_LAZY=1`` or
  ``PROCESSKIT_MCP_MODE=lazy_catalog``. Override the catalog source
  via ``PROCESSKIT_MCP_CATALOG_PATH``.

Both modes preserve ``list_aggregate_tools`` shape and the same
collision/dedup naming rules.
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
_IMPORT_MODE = "eager"
_CATALOG_PATH: Path | None = None
_LAZY_REGISTRY: Any | None = None


def _truthy(value: str | None) -> bool:
    if value is None:
        return False
    return value.strip().lower() in {"1", "true", "yes", "on"}


def _resolve_import_mode() -> str:
    """Pick the import mode from environment, defaulting to ``eager``."""
    explicit = os.environ.get("PROCESSKIT_MCP_MODE", "").strip().lower()
    # ``aggregate`` is a legacy harness marker and does not select a mode.
    if explicit in {"lazy_catalog", "lazy-catalog", "lazy"}:
        return "lazy_catalog"
    if _truthy(os.environ.get("PROCESSKIT_MCP_LAZY")):
        return "lazy_catalog"
    return "eager"


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


def _resolve_catalog_path() -> Path:
    """Return the catalog file consumed in lazy_catalog mode."""
    override = os.environ.get("PROCESSKIT_MCP_CATALOG_PATH")
    if override:
        return Path(override).expanduser().resolve()
    try:
        catalog_mod = importlib.import_module("processkit.gateway.catalog")
        default_catalog_path = getattr(catalog_mod, "default_catalog_path")
    except (ModuleNotFoundError, AttributeError):
        # Best-effort fallback when the helper is missing.
        return (
            _skills_root()
            / "processkit"
            / "processkit-gateway"
            / "mcp"
            / "tool-catalog.json"
        )
    return Path(default_catalog_path(_skills_root())).resolve()


def _register_lazy_tools() -> bool:
    """Wire the FastMCP surface from the cached gateway catalog.

    Returns ``True`` on success. Returns ``False`` if the lazy
    infrastructure is not importable or the catalog is missing — in
    that case the caller should fall back to the eager path.
    """
    global _LAZY_REGISTRY, _CANONICAL_GATEWAY_AVAILABLE, _REGISTRY_BACKEND
    global _SOURCE_SERVER_COUNT, _CATALOG_PATH

    try:
        registry_mod = importlib.import_module("processkit.gateway.registry")
        runtime_mod = importlib.import_module("processkit.gateway.runtime")
    except ModuleNotFoundError as exc:
        if exc.name in {
            "processkit.gateway",
            "processkit.gateway.registry",
            "processkit.gateway.runtime",
        }:
            return False
        raise

    GatewayRegistry = getattr(registry_mod, "GatewayRegistry")
    register_gateway_tools = getattr(registry_mod, "register_gateway_tools")
    RuntimeMetadata = getattr(runtime_mod, "RuntimeMetadata")

    catalog_path = _resolve_catalog_path()
    if not catalog_path.is_file():
        # No catalog committed yet — caller falls back to eager.
        return False

    registry = GatewayRegistry(
        self_skill=_SELF_SKILL,
        exclude_skills=set(_EXCLUDED_SKILLS),
        runtime=RuntimeMetadata(
            import_mode="lazy-catalog",
            lazy=True,
            lazy_daemon=False,
        ),
        catalog_path=catalog_path,
    )
    registry.load()

    # Replay aggregate-server collision/dedup rules over the catalog
    # entries so the public tool names match the eager mode.
    entries = [
        {
            "source_skill": item["source_skill"],
            "source_server_path": item["source_server_path"],
            "source_tool": item["source_tool"],
            "catalog_name": item["name"],
            "raw": item,
        }
        for item in registry.tools
    ]
    groups = _collision_groups(entries)
    duplicate_seen: dict[str, int] = {}
    registered_names: set[str] = set()
    rebuilt_tools: list[dict[str, Any]] = []

    for entry in entries:
        original_name = entry["source_tool"]
        duplicate_index = duplicate_seen.get(original_name, 0)
        duplicate_seen[original_name] = duplicate_index + 1

        name = _registered_name(entry, registered_names, duplicate_index)
        registered_names.add(name)

        # Preserve the catalog row but rename so the lazy wrapper
        # binds to the aggregate-server public name.
        catalog_row = dict(entry["raw"])
        catalog_row["name"] = name
        rebuilt_tools.append(catalog_row)

    # Replace the registry tools with the renamed copies so
    # register_gateway_tools and call_tool both use the aggregate names.
    registry.tools = rebuilt_tools
    registry._tool_index = {item["name"]: item for item in rebuilt_tools}

    register_gateway_tools(server, registry)

    _SOURCE_SERVER_COUNT = int(
        getattr(registry, "source_server_count", 0)
    ) or len({item["source_server_path"] for item in rebuilt_tools})
    _CATALOG_PATH = catalog_path
    _LAZY_REGISTRY = registry
    _CANONICAL_GATEWAY_AVAILABLE = True
    _REGISTRY_BACKEND = "gateway_lazy_catalog"

    duplicate_seen_meta: dict[str, int] = {}
    for item in rebuilt_tools:
        original_name = item["source_tool"]
        idx = duplicate_seen_meta.get(original_name, 0)
        duplicate_seen_meta[original_name] = idx + 1
        collision_sources = groups[original_name]
        collision_status = "unique"
        if len(collision_sources) > 1:
            collision_status = (
                "primary" if idx == 0 else "namespaced_duplicate"
            )
        _TOOLS.append({
            "name": item["name"],
            "source_skill": item["source_skill"],
            "source_server_path": item["source_server_path"],
            "source_tool": original_name,
            "annotations": item.get("annotations"),
            "collision_status": collision_status,
            "collision_sources": collision_sources,
            "deduplicated": item["name"] != original_name,
        })

    return True


def _register_eager_tools() -> None:
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


def _register_tools() -> None:
    global _IMPORT_MODE

    requested_mode = _resolve_import_mode()
    if requested_mode == "lazy_catalog":
        if _register_lazy_tools():
            _IMPORT_MODE = "lazy_catalog"
            return
        # Catalog or lazy infra unavailable — fall through to eager and
        # leave _IMPORT_MODE as "eager" so callers see the actual mode.
    _register_eager_tools()


_register_tools()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_aggregate_tools() -> dict:
    """List tools exposed by the aggregate processkit MCP server."""
    if _IMPORT_MODE == "lazy_catalog":
        import_mode = "lazy_catalog"
        lazy_daemon = True
    elif _CANONICAL_GATEWAY_AVAILABLE:
        import_mode = "gateway_registry"
        lazy_daemon = False
    else:
        import_mode = "eager"
        lazy_daemon = False

    payload = {
        "count": len(_TOOLS),
        "source_server_count": _SOURCE_SERVER_COUNT,
        "runtime": {
            "import_mode": import_mode,
            "lazy_daemon": lazy_daemon,
            "registry_backend": _REGISTRY_BACKEND,
            "canonical_gateway_available": _CANONICAL_GATEWAY_AVAILABLE,
        },
        "tools": _TOOLS,
    }
    if _CATALOG_PATH is not None:
        payload["runtime"]["catalog_path"] = _CATALOG_PATH.as_posix()
    return payload


if __name__ == "__main__":
    server.run()
