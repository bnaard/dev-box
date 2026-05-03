"""Gateway registry and FastMCP registration helpers."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from .catalog import catalog_payload, read_catalog, write_catalog
from .loader import collect_source_tools
from .loader import import_server
from .naming import collision_groups, registered_name
from .permissions import annotation_metadata, permission_class
from .runtime import RuntimeMetadata


@dataclass
class GatewayRegistry:
    """Collected processkit MCP tool registry."""

    self_skill: str
    exclude_skills: set[str] = field(default_factory=set)
    runtime: RuntimeMetadata = field(default_factory=RuntimeMetadata)
    catalog_path: Path | None = None
    tools: list[dict[str, Any]] = field(default_factory=list)
    source_server_count: int = 0
    _loaded: bool = False
    _tool_index: dict[str, dict[str, Any]] = field(default_factory=dict)

    def load(self) -> "GatewayRegistry":
        if self._loaded:
            return self

        if self.runtime.import_mode == "lazy-catalog":
            self.load_catalog()
            return self

        entries = collect_source_tools(
            self_skill=self.self_skill,
            exclude_skills=self.exclude_skills,
        )
        self.source_server_count = len({
            entry["source_server_path"] for entry in entries
        })
        groups = collision_groups(entries)
        duplicate_seen: dict[str, int] = {}
        registered_names: set[str] = set()

        for entry in entries:
            original_name = entry["source_tool"]
            duplicate_index = duplicate_seen.get(original_name, 0)
            duplicate_seen[original_name] = duplicate_index + 1

            name = registered_name(entry, registered_names, duplicate_index)
            registered_names.add(name)

            tool = entry["tool"]
            annotations = getattr(tool, "annotations", None)
            collision_status = "unique"
            collision_sources = groups[original_name]
            if len(collision_sources) > 1:
                collision_status = (
                    "primary" if duplicate_index == 0
                    else "namespaced_duplicate"
                )

            self.tools.append({
                "name": name,
                "source_skill": entry["source_skill"],
                "source_server_path": entry["source_server_path"],
                "source_tool": original_name,
                "annotations": annotation_metadata(annotations),
                "permission_class": permission_class(annotations),
                "collision_status": collision_status,
                "collision_sources": collision_sources,
                "deduplicated": name != original_name,
                "tool": tool,
            })

        self._tool_index = {item["name"]: item for item in self.tools}
        self._loaded = True
        return self

    def load_catalog(self) -> "GatewayRegistry":
        """Load registered tool metadata from a catalog file."""
        if self._loaded:
            return self
        if self.catalog_path is None:
            raise RuntimeError("lazy-catalog mode requires catalog_path")
        payload = read_catalog(self.catalog_path)
        self.source_server_count = int(payload.get("source_server_count") or 0)
        self.tools = list(payload["tools"])
        self._tool_index = {item["name"]: item for item in self.tools}
        self._loaded = True
        return self

    def write_catalog(self, path: Path | None = None) -> Path:
        """Write a catalog generated from eager source tool discovery."""
        if self.runtime.import_mode == "lazy-catalog":
            raise RuntimeError("cannot generate catalog from lazy metadata")
        self.load()
        target = path or self.catalog_path
        if target is None:
            raise RuntimeError("catalog path required")
        write_catalog(target, catalog_payload(self.tools, self.source_server_count))
        return target

    def metadata(self, *, include_tools: bool = True) -> dict[str, Any]:
        self.load()
        payload: dict[str, Any] = {
            "count": len(self.tools),
            "source_server_count": self.source_server_count,
            "runtime": self.runtime.as_dict(),
        }
        if include_tools:
            payload["tools"] = [
                {k: v for k, v in item.items() if k != "tool"}
                for item in self.tools
            ]
        return payload

    async def call_tool(self, name: str, arguments: dict[str, Any]) -> Any:
        """Call a lazily loaded backing tool by gateway registered name."""
        self.load()
        item = self._tool_index.get(name)
        if item is None:
            raise KeyError(f"unknown gateway tool: {name}")
        tool = item.get("tool")
        if tool is None:
            source_path = item["source_server_path"]
            _skill, module = import_server(source_path)
            source_server = getattr(module, "server", None)
            manager = getattr(source_server, "_tool_manager", None)
            source_tools = getattr(manager, "_tools", {}) if manager else {}
            tool = source_tools.get(item["source_tool"])
            if tool is None:
                raise KeyError(
                    f"source tool {item['source_tool']!r} not found in "
                    f"{source_path}"
                )
            item["tool"] = tool
        return await tool.run(arguments)


def register_gateway_tools(server: Any, registry: GatewayRegistry) -> None:
    """Register all source tools on ``server`` using gateway names."""
    registry.load()
    if registry.runtime.import_mode == "lazy-catalog":
        from .lazy import make_lazy_tool

        for item in registry.tools:
            tool = make_lazy_tool(registry, item)
            server._tool_manager._tools[item["name"]] = tool
        return

    for item in registry.tools:
        tool = item["tool"]
        server.tool(
            name=item["name"],
            description=getattr(tool, "description", None),
            annotations=getattr(tool, "annotations", None),
        )(tool.fn)


def collect_gateway_tools(
    *,
    skills_root: Any | None = None,
    exclude_skills: set[str] | None = None,
) -> list[dict[str, Any]]:
    """Compatibility collector for aggregate-mcp and gateway adapters."""
    return collect_source_tools(
        root=skills_root,
        exclude_skills=exclude_skills,
    )
