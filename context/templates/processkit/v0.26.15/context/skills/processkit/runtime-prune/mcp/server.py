#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
# ]
# ///
"""Provider-neutral runtime prune MCP server."""
from __future__ import annotations

import os
import shutil
import sys
from pathlib import Path
from typing import Iterable, Optional

from mcp.server.fastmcp import FastMCP
from mcp.types import ToolAnnotations


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

server = FastMCP("processkit-runtime-prune")

SUPPORTED_SCOPES = {
    "runtime-home",
    "build-cache",
    "agent-worktrees",
    "e2e-companion",
    "containers",
}
DEFAULT_SCOPES = [
    "runtime-home",
    "build-cache",
    "agent-worktrees",
    "containers",
    "e2e-companion",
]

RUNTIME_HOME_REL_PATHS = [
    ".aibox-home/cache",
    ".aibox-home/caches",
    ".aibox-home/diagnostics",
    ".aibox-home/tmp",
    ".aibox/cache",
    ".aibox/caches",
    ".aibox/diagnostics",
    ".aibox/runtime",
]
BUILD_CACHE_REL_PATHS = [
    "target/debug/incremental",
    "target/debug/build",
    "target/release/incremental",
    "target/release/build",
]
PROCESSKIT_APPLY_SCOPES = {"runtime-home", "build-cache"}
DELEGATED_APPLY_SCOPES = {
    "agent-worktrees",
    "e2e-companion",
    "containers",
}


def _project_root(project_root: Optional[str] = None) -> Path:
    if project_root:
        root = Path(project_root).expanduser().resolve()
        if not root.exists() or not root.is_dir():
            raise ValueError(f"project_root does not exist: {project_root}")
        return root

    here = Path.cwd().resolve()
    for candidate in [here, *here.parents]:
        if (
            (candidate / "aibox.toml").is_file()
            or (candidate / "AGENTS.md").is_file()
            or (candidate / ".git").exists()
        ):
            return candidate
    return here


def _normalize_scopes(scopes: Optional[list[str] | str]) -> list[str]:
    if scopes is None:
        values = DEFAULT_SCOPES
    elif isinstance(scopes, str):
        values = [
            part.strip()
            for part in scopes.split(",")
            if part.strip()
        ]
    else:
        values = [str(scope).strip() for scope in scopes if str(scope).strip()]

    normalized: list[str] = []
    for scope in values:
        if scope not in SUPPORTED_SCOPES:
            raise ValueError(
                f"unsupported scope {scope!r}; supported scopes: "
                f"{', '.join(sorted(SUPPORTED_SCOPES))}"
            )
        if scope not in normalized:
            normalized.append(scope)
    return normalized


def _rel(path: Path, root: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def _dir_size(path: Path) -> tuple[int, int]:
    total = 0
    count = 0
    if not path.exists():
        return total, count

    stack = [path]
    while stack:
        current = stack.pop()
        try:
            stat = current.lstat()
        except OSError:
            continue
        total += stat.st_size
        count += 1
        if current.is_dir() and not current.is_symlink():
            try:
                children = list(current.iterdir())
            except OSError:
                continue
            stack.extend(children)
    return total, count


def _path_targets(root: Path, rel_paths: Iterable[str]) -> list[dict]:
    targets = []
    for rel_path in rel_paths:
        path = root / rel_path
        exists = path.exists()
        size, file_count = _dir_size(path) if exists else (0, 0)
        targets.append({
            "path": rel_path,
            "exists": exists,
            "size_bytes": size,
            "file_count": file_count,
            "symlink": path.is_symlink() if path.exists() else False,
        })
    return targets


def _git_worktrees(root: Path) -> list[dict]:
    git_dir = root / ".git"
    if not git_dir.exists():
        return []
    try:
        completed = subprocess.run(
            ["git", "-C", str(root), "worktree", "list", "--porcelain"],
            check=False,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired):
        return []
    if completed.returncode != 0:
        return []

    worktrees: list[dict] = []
    current: dict[str, object] = {}
    for line in completed.stdout.splitlines():
        if not line:
            if current:
                worktrees.append(current)
                current = {}
            continue
        if line.startswith("worktree "):
            path = Path(line.removeprefix("worktree ")).expanduser()
            current["path"] = path.as_posix()
            current["inside_project"] = _is_relative_to(path, root)
            current["managed_hint"] = _looks_managed_worktree(path)
        elif line == "bare":
            current["bare"] = True
        elif line == "detached":
            current["detached"] = True
        elif line.startswith("branch "):
            current["branch"] = line.removeprefix("branch ")
        elif line.startswith("HEAD "):
            current["head"] = line.removeprefix("HEAD ")
    if current:
        worktrees.append(current)
    return worktrees


def _is_relative_to(path: Path, root: Path) -> bool:
    try:
        path.resolve().relative_to(root.resolve())
    except (OSError, ValueError):
        return False
    return True


def _looks_managed_worktree(path: Path) -> bool:
    parts = {part.lower() for part in path.parts}
    text = path.as_posix().lower()
    return (
        ".aibox" in parts
        or ".aibox-worktrees" in parts
        or "aibox-worktrees" in parts
        or "/worktrees/" in text
        or "/agent-worktrees/" in text
    )


def _confirmation(scopes: list[str]) -> str:
    return "apply-prune:" + ",".join(scopes)


def _analyze(root: Path, scopes: list[str]) -> dict:
    scope_data: list[dict] = []
    total = 0

    if "runtime-home" in scopes:
        targets = _path_targets(root, RUNTIME_HOME_REL_PATHS)
        size = sum(item["size_bytes"] for item in targets)
        total += size
        scope_data.append({
            "scope": "runtime-home",
            "risk": "low",
            "expected_reclaim_bytes": size,
            "targets": targets,
            "notes": [
                "Managed runtime cache and diagnostics paths only.",
            ],
        })

    if "build-cache" in scopes:
        targets = _path_targets(root, BUILD_CACHE_REL_PATHS)
        size = sum(item["size_bytes"] for item in targets)
        total += size
        scope_data.append({
            "scope": "build-cache",
            "risk": "medium",
            "expected_reclaim_bytes": size,
            "targets": targets,
            "notes": [
                (
                    "Explicit build-cache subdirectories; not the whole "
                    "target tree."
                ),
            ],
        })

    if "agent-worktrees" in scopes:
        worktrees = _git_worktrees(root)
        managed = [item for item in worktrees if item.get("managed_hint")]
        scope_data.append({
            "scope": "agent-worktrees",
            "risk": "high",
            "expected_reclaim_bytes": 0,
            "worktree_count": len(worktrees),
            "managed_hint_count": len(managed),
            "targets": worktrees,
            "notes": [
                (
                    "Inventory only; deletion is delegated to the host "
                    "runtime manager because worktree ownership is external "
                    "to processkit."
                ),
            ],
        })

    if "containers" in scopes:
        scope_data.append({
            "scope": "containers",
            "risk": "high",
            "expected_reclaim_bytes": 0,
            "targets": [],
            "notes": [
                (
                    "Container cleanup is delegated to the host runtime "
                    "manager; this tool does not call local Docker, Podman, "
                    "or host CLI wrappers directly."
                ),
            ],
        })

    if "e2e-companion" in scopes:
        scope_data.append({
            "scope": "e2e-companion",
            "risk": "high",
            "expected_reclaim_bytes": 0,
            "targets": [],
            "notes": [
                (
                    "Companion cleanup must use the host runtime manager's "
                    "reachability checks, not local container inspection."
                ),
            ],
        })

    return {
        "project_root": root.as_posix(),
        "scopes": scopes,
        "supported_scopes": sorted(SUPPORTED_SCOPES),
        "total_expected_reclaim_bytes": total,
        "scope_data": scope_data,
    }


def _targets_for_scope(root: Path, scope: str) -> list[dict]:
    if scope == "runtime-home":
        return _path_targets(root, RUNTIME_HOME_REL_PATHS)
    if scope == "build-cache":
        return _path_targets(root, BUILD_CACHE_REL_PATHS)
    return []


def _remove_target(root: Path, rel_path: str) -> dict:
    path = root / rel_path
    evidence = {
        "path": rel_path,
        "existed": path.exists(),
        "applied": False,
        "reclaimed_bytes": 0,
        "file_count": 0,
    }
    if not path.exists():
        evidence["reason"] = "target absent"
        return evidence
    try:
        resolved = path.resolve()
        resolved.relative_to(root.resolve())
    except (OSError, ValueError):
        evidence["error"] = "target is outside project_root"
        return evidence
    if path.is_symlink():
        evidence["error"] = "refusing to remove symlink target"
        return evidence

    size, file_count = _dir_size(path)
    try:
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()
    except OSError as exc:
        evidence["error"] = str(exc)
        return evidence

    evidence["applied"] = True
    evidence["reclaimed_bytes"] = size
    evidence["file_count"] = file_count
    return evidence


def _apply_processkit_scope(root: Path, scope: str) -> dict:
    targets = _targets_for_scope(root, scope)
    target_results = [
        _remove_target(root, target["path"])
        for target in targets
        if target["exists"]
    ]
    errors = [item for item in target_results if item.get("error")]
    reclaimed = sum(item["reclaimed_bytes"] for item in target_results)
    applied_any = any(item["applied"] for item in target_results)
    return {
        "scope": scope,
        "owner": "processkit",
        "applied": applied_any and not errors,
        "unsupported": False,
        "reclaimed_bytes": reclaimed,
        "targets": target_results,
        "errors": errors,
        "reason": None if target_results else "no allowlisted targets present",
    }


def _apply_external_scope(root: Path, scope: str) -> dict:
    return {
        "scope": scope,
        "owner": "host runtime manager",
        "applied": False,
        "unsupported": True,
        "reason": (
            "scope is external to processkit; ask the owner to run the "
            "project's host-side runtime cleanup tool outside the container"
        ),
        "project_root": root.as_posix(),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def analyze_disk_usage(
    project_root: Optional[str] = None,
    scopes: Optional[list[str] | str] = None,
) -> dict:
    """Return structured disk usage and risk data for prune scopes."""
    try:
        root = _project_root(project_root)
        normalized = _normalize_scopes(scopes)
    except ValueError as exc:
        return {"error": str(exc)}
    return _analyze(root, normalized)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def plan_prune(
    project_root: Optional[str] = None,
    scopes: Optional[list[str] | str] = None,
) -> dict:
    """Return a dry-run prune plan without applying cleanup."""
    try:
        root = _project_root(project_root)
        normalized = _normalize_scopes(scopes)
    except ValueError as exc:
        return {"error": str(exc)}

    analysis = _analyze(root, normalized)
    actions = []
    for item in analysis["scope_data"]:
        processkit_owned = item["scope"] in PROCESSKIT_APPLY_SCOPES
        actions.append({
            "scope": item["scope"],
            "risk": item["risk"],
            "destructive": True,
            "apply_supported": processkit_owned,
            "apply_owner": (
                "processkit" if processkit_owned else "host runtime manager"
            ),
            "expected_reclaim_bytes": item["expected_reclaim_bytes"],
            "targets": item.get("targets", []),
            "dry_run_command": None,
            "apply_command": None,
            "notes": item.get("notes", []),
        })

    return {
        "project_root": root.as_posix(),
        "scopes": normalized,
        "dry_run": True,
        "required_confirmation": _confirmation(normalized),
        "total_expected_reclaim_bytes": (
            analysis["total_expected_reclaim_bytes"]
        ),
        "actions": actions,
        "safety": {
            "manual_deletion_performed": False,
            "apply_requires_confirmation": True,
            "destructive_logic_owner": (
                "processkit allowlist for supported scopes; host runtime "
                "manager for external scopes"
            ),
            "processkit_apply_scopes": sorted(PROCESSKIT_APPLY_SCOPES),
            "external_apply_scopes": sorted(DELEGATED_APPLY_SCOPES),
        },
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=False,
    openWorldHint=False,
))
def apply_prune(
    project_root: Optional[str] = None,
    scopes: Optional[list[str] | str] = None,
    confirmation: Optional[str] = None,
) -> dict:
    """Apply confirmed cleanup for processkit-owned scopes."""
    try:
        root = _project_root(project_root)
        normalized = _normalize_scopes(scopes)
    except ValueError as exc:
        return {"error": str(exc)}

    required = _confirmation(normalized)
    if confirmation != required:
        return {
            "error": "confirmation mismatch",
            "required_confirmation": required,
            "applied": False,
        }
    results = []
    for scope in normalized:
        if scope in PROCESSKIT_APPLY_SCOPES:
            results.append(_apply_processkit_scope(root, scope))
        else:
            results.append(_apply_external_scope(root, scope))

    return {
        "applied": all(item["applied"] for item in results),
        "partial": any(item["applied"] for item in results),
        "project_root": root.as_posix(),
        "scopes": normalized,
        "required_confirmation": required,
        "results": results,
    }


if __name__ == "__main__":
    server.run()
