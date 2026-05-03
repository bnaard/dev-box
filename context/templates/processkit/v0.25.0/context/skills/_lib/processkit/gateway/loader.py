"""Source MCP server discovery and import helpers."""

from __future__ import annotations

import hashlib
import importlib.util
import os
from functools import lru_cache
from pathlib import Path
from types import ModuleType
from typing import Any


def skills_root() -> Path:
    """Return the installed ``context/skills`` or ``src/context/skills`` root."""
    env = os.environ.get("PROCESSKIT_SKILLS_ROOT")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve()
    for parent in here.parents:
        if parent.name == "skills":
            return parent
    raise RuntimeError("processkit skills root not found")


def iter_server_paths(
    self_skill: str | None = None,
    root: Path | None = None,
    exclude_skills: set[str] | None = None,
) -> list[Path]:
    """List processkit skill MCP server scripts, excluding ``self_skill``."""
    root = root or skills_root()
    excluded = set(exclude_skills or set())
    if self_skill:
        excluded.add(self_skill)
    paths: list[Path] = []
    for candidate in root.glob("processkit/*/mcp/server.py"):
        skill = candidate.parents[1].name
        if skill in excluded:
            continue
        paths.append(candidate)
    return sorted(paths, key=lambda p: p.parents[1].name)


def relative_server_path(path: Path) -> str:
    try:
        return path.relative_to(skills_root()).as_posix()
    except ValueError:
        return path.as_posix()


@lru_cache(maxsize=64)
def import_server(path_text: str) -> tuple[str, ModuleType]:
    """Import an MCP ``server.py`` by path and return ``(skill, module)``."""
    raw_path = Path(path_text)
    if not raw_path.is_absolute() and not raw_path.exists():
        raw_path = skills_root() / raw_path
    path = raw_path.resolve()
    skill = path.parents[1].name
    digest = hashlib.sha1(path.as_posix().encode("utf-8")).hexdigest()[:12]
    module_name = f"processkit_gateway_{skill.replace('-', '_')}_{digest}"
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot import MCP server at {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return skill, module


def collect_source_tools(
    self_skill: str | None = None,
    root: Path | None = None,
    exclude_skills: set[str] | None = None,
) -> list[dict[str, Any]]:
    """Collect FastMCP tool objects from source processkit MCP servers."""
    entries: list[dict[str, Any]] = []
    for path in iter_server_paths(
        self_skill=self_skill,
        root=root,
        exclude_skills=exclude_skills,
    ):
        skill, module = import_server(path.as_posix())
        source_server = getattr(module, "server", None)
        manager = getattr(source_server, "_tool_manager", None)
        tools = getattr(manager, "_tools", {}) if manager else {}
        for original_name, tool in sorted(tools.items()):
            entries.append({
                "source_skill": skill,
                "source_server_path": relative_server_path(path),
                "source_tool": original_name,
                "tool": tool,
            })
    return entries
