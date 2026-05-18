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
import subprocess
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
    "repo-artifacts",
    "tool-caches",
    "agent-worktrees",
    "e2e-companion",
    "containers",
    "action-artifacts",
    "release-assets",
    "package-registry",
}
DEFAULT_SCOPES = [
    "runtime-home",
    "build-cache",
    "repo-artifacts",
    "tool-caches",
    "agent-worktrees",
    "containers",
    "e2e-companion",
    "action-artifacts",
    "release-assets",
    "package-registry",
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
REPO_ARTIFACT_REL_PATHS = [
    "dist",
    "build",
    "site",
    "htmlcov",
    "coverage",
    ".coverage",
    ".pytest_cache",
    ".ruff_cache",
    ".mypy_cache",
    ".nox",
    ".tox",
]
TOOL_CACHE_REL_PATHS = [
    ".cache/pip",
    ".cache/pip-tools",
    ".cache/uv",
    ".npm/_cacache",
    ".pnpm-store",
    "node_modules/.cache",
]
PROCESSKIT_APPLY_SCOPES = {
    "runtime-home",
    "build-cache",
    "repo-artifacts",
    "tool-caches",
}
DELEGATED_APPLY_SCOPES = {
    "agent-worktrees",
    "e2e-companion",
    "containers",
    "action-artifacts",
    "release-assets",
    "package-registry",
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


def _git_remote_url(root: Path) -> str | None:
    try:
        completed = subprocess.run(
            ["git", "-C", str(root), "remote", "get-url", "origin"],
            check=False,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode != 0:
        return None
    url = completed.stdout.strip()
    return url or None


def _repo_provider(root: Path) -> dict:
    url = _git_remote_url(root)
    if not url:
        return {
            "provider": "unknown",
            "remote_url": None,
            "host": None,
            "notes": ["No origin remote detected; use generic provider docs."],
        }

    lowered = url.lower()
    provider = "generic"
    host = None
    for marker in ("github.com", "gitlab.com", "codeberg.org"):
        if marker in lowered:
            host = marker
            break
    if "github.com" in lowered:
        provider = "github"
    elif "gitlab.com" in lowered:
        provider = "gitlab"
    elif "codeberg.org" in lowered:
        provider = "codeberg"
    elif "forgejo" in lowered:
        provider = "forgejo"
    elif "gitea" in lowered:
        provider = "gitea"
    return {
        "provider": provider,
        "remote_url": url,
        "host": host,
        "notes": [],
    }


def _confirmation(scopes: list[str]) -> str:
    return "apply-prune:" + ",".join(scopes)


def _analyze(root: Path, scopes: list[str]) -> dict:
    scope_data: list[dict] = []
    total = 0
    provider = _repo_provider(root)

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

    if "repo-artifacts" in scopes:
        targets = _path_targets(root, REPO_ARTIFACT_REL_PATHS)
        size = sum(item["size_bytes"] for item in targets)
        total += size
        scope_data.append({
            "scope": "repo-artifacts",
            "risk": "medium",
            "expected_reclaim_bytes": size,
            "targets": targets,
            "notes": [
                (
                    "Local generated release/build/test artifacts only; "
                    "verify nothing here is the only copy before applying."
                ),
            ],
        })

    if "tool-caches" in scopes:
        targets = _path_targets(root, TOOL_CACHE_REL_PATHS)
        size = sum(item["size_bytes"] for item in targets)
        total += size
        scope_data.append({
            "scope": "tool-caches",
            "risk": "medium",
            "expected_reclaim_bytes": size,
            "targets": targets,
            "notes": [
                (
                    "Project-local package/tool caches only; global caches "
                    "are host-owned and intentionally not removed."
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
                (
                    "Best practice is staged cleanup: inspect with "
                    "`docker system df`, prune builder cache separately, "
                    "avoid `--volumes` unless data ownership is explicit."
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

    if "action-artifacts" in scopes:
        scope_data.append({
            "scope": "action-artifacts",
            "risk": "high",
            "expected_reclaim_bytes": 0,
            "targets": [],
            "provider": provider,
            "notes": [
                (
                    "Remote CI/action artifacts and logs are provider-owned; "
                    "set short retention for non-release artifacts and "
                    "delete older artifacts/runs through the provider API."
                ),
            ],
        })

    if "release-assets" in scopes:
        scope_data.append({
            "scope": "release-assets",
            "risk": "high",
            "expected_reclaim_bytes": 0,
            "targets": [],
            "provider": provider,
            "notes": [
                (
                    "Remote release assets are permanent until deleted; "
                    "keep source tags/releases, but remove binary assets "
                    "outside the supported-version retention window."
                ),
            ],
        })

    if "package-registry" in scopes:
        scope_data.append({
            "scope": "package-registry",
            "risk": "high",
            "expected_reclaim_bytes": 0,
            "targets": [],
            "provider": provider,
            "notes": [
                (
                    "Remote package/container cleanup needs registry admin "
                    "permissions; keep semver releases, active deployment "
                    "tags, and referenced digests; delete stale untagged or "
                    "superseded versions."
                ),
            ],
        })

    return {
        "project_root": root.as_posix(),
        "scopes": scopes,
        "supported_scopes": sorted(SUPPORTED_SCOPES),
        "repo_provider": provider,
        "total_expected_reclaim_bytes": total,
        "scope_data": scope_data,
    }


def _targets_for_scope(root: Path, scope: str) -> list[dict]:
    if scope == "runtime-home":
        return _path_targets(root, RUNTIME_HOME_REL_PATHS)
    if scope == "build-cache":
        return _path_targets(root, BUILD_CACHE_REL_PATHS)
    if scope == "repo-artifacts":
        return _path_targets(root, REPO_ARTIFACT_REL_PATHS)
    if scope == "tool-caches":
        return _path_targets(root, TOOL_CACHE_REL_PATHS)
    return []


def _remote_runbook(scope: str, provider: str = "unknown") -> dict | None:
    if scope == "containers":
        return {
            "required_env": [],
            "inventory_command": "docker system df && docker buildx du",
            "dry_run_command": (
                "docker builder prune --filter until=${RETENTION_HOURS:-168}h "
                "--dry-run || true"
            ),
            "apply_command": (
                "docker builder prune --filter until=${RETENTION_HOURS:-168}h "
                "--force && docker image prune "
                "--filter until=${RETENTION_HOURS:-168}h --force"
            ),
            "space_estimate": (
                "Use the RECLAIMABLE column from `docker system df` and "
                "`docker buildx du`."
            ),
        }

    if provider == "github":
        return _github_runbook(scope)
    if provider == "gitlab":
        return _gitlab_runbook(scope)
    if provider in {"codeberg", "forgejo", "gitea"}:
        return _forgejo_gitea_runbook(scope, provider)
    if scope in {"action-artifacts", "release-assets", "package-registry"}:
        return {
            "required_env": ["PROVIDER_API_URL", "PROVIDER_TOKEN"],
            "inventory_command": None,
            "dry_run_command": None,
            "apply_command": None,
            "space_estimate": (
                "Provider is unknown; inventory and deletion endpoints must "
                "be filled from the forge or registry API documentation."
            ),
        }
    return None


def _github_runbook(scope: str) -> dict | None:
    if scope == "action-artifacts":
        inventory = (
            "gh api \"repos/${OWNER}/${REPO}/actions/artifacts?per_page=100\" "
            "--paginate > /tmp/pk-action-artifacts.json\n"
            "jq -r '.artifacts[] | [.id,.name,.size_in_bytes,.created_at,"
            ".expires_at] | @tsv' /tmp/pk-action-artifacts.json"
        )
        dry_run = (
            "RETENTION_DAYS=${RETENTION_DAYS:-60}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            "gh api \"repos/${OWNER}/${REPO}/actions/artifacts?per_page=100\" "
            "--paginate > /tmp/pk-action-artifacts.json\n"
            "jq -r --arg cutoff \"$cutoff\" '\n"
            "  .artifacts[] | select(.created_at < $cutoff) |\n"
            "  [.id,.name,.size_in_bytes,.created_at,.expires_at] | @tsv\n"
            "' /tmp/pk-action-artifacts.json\n"
            "jq --arg cutoff \"$cutoff\" '\n"
            "  [ .artifacts[] | select(.created_at < $cutoff) | "
            ".size_in_bytes ] | add // 0\n"
            "' /tmp/pk-action-artifacts.json"
        )
        apply = (
            "RETENTION_DAYS=${RETENTION_DAYS:-60}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            "gh api \"repos/${OWNER}/${REPO}/actions/artifacts?per_page=100\" "
            "--paginate > /tmp/pk-action-artifacts.json\n"
            "jq -r --arg cutoff \"$cutoff\" '\n"
            "  .artifacts[] | select(.created_at < $cutoff) | .id\n"
            "' /tmp/pk-action-artifacts.json |\n"
            "while read -r id; do\n"
            "  gh api --method DELETE "
            "\"repos/${OWNER}/${REPO}/actions/artifacts/${id}\"\n"
            "done"
        )
        return _runbook(["OWNER", "REPO"], inventory, dry_run, apply)

    if scope == "release-assets":
        inventory = (
            "gh release list --limit 200 --json tagName,createdAt,isLatest "
            "> /tmp/pk-releases.json\n"
            "jq -r '.[] | [.tagName,.createdAt,.isLatest] | @tsv' "
            "/tmp/pk-releases.json"
        )
        dry_run = (
            "RETENTION_DAYS=${RETENTION_DAYS:-180}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            "gh release list --limit 200 --json tagName,createdAt,isLatest |\n"
            "jq -r --arg cutoff \"$cutoff\" '\n"
            "  .[] | select(.createdAt < $cutoff and (.isLatest|not)) | "
            ".tagName\n"
            "' | while read -r tag; do\n"
            "  gh release view \"$tag\" --json tagName,assets |\n"
            "  jq -r '.assets[] | [$tag,.name,.size] | @tsv' "
            "--arg tag \"$tag\"\n"
            "done"
        )
        apply = (
            "RETENTION_DAYS=${RETENTION_DAYS:-180}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            "gh release list --limit 200 --json tagName,createdAt,isLatest |\n"
            "jq -r --arg cutoff \"$cutoff\" '\n"
            "  .[] | select(.createdAt < $cutoff and (.isLatest|not)) | "
            ".tagName\n"
            "' | while read -r tag; do\n"
            "  gh release view \"$tag\" --json assets |\n"
            "  jq -r '.assets[].name' |\n"
            "  while read -r asset; do\n"
            "    gh release delete-asset \"$tag\" \"$asset\" --yes\n"
            "  done\n"
            "done"
        )
        return _runbook([], inventory, dry_run, apply)

    if scope == "package-registry":
        inventory = (
            "OWNER_TYPE=${OWNER_TYPE:-orgs}\n"
            "PACKAGE_TYPE=${PACKAGE_TYPE:-container}\n"
            "gh api \"$OWNER_TYPE/${OWNER}/packages/${PACKAGE_TYPE}/"
            "${PACKAGE_NAME}/versions?per_page=100\" --paginate "
            "> /tmp/pk-package-versions.json\n"
            "jq -r '.[] | [.id,.name,((.metadata.container.tags // [])|join("
            "\",\")),.created_at] | @tsv' /tmp/pk-package-versions.json"
        )
        dry_run = (
            "RETENTION_DAYS=${RETENTION_DAYS:-90}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            "OWNER_TYPE=${OWNER_TYPE:-orgs}\n"
            "PACKAGE_TYPE=${PACKAGE_TYPE:-container}\n"
            "gh api \"$OWNER_TYPE/${OWNER}/packages/${PACKAGE_TYPE}/"
            "${PACKAGE_NAME}/versions?per_page=100\" --paginate "
            "> /tmp/pk-package-versions.json\n"
            "jq -r --arg cutoff \"$cutoff\" '\n"
            "  .[] | select(.created_at < $cutoff) |\n"
            "  select((.metadata.container.tags // []) | length == 0) |\n"
            "  [.id,.name,.created_at] | @tsv\n"
            "' /tmp/pk-package-versions.json"
        )
        apply = (
            "RETENTION_DAYS=${RETENTION_DAYS:-90}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            "OWNER_TYPE=${OWNER_TYPE:-orgs}\n"
            "PACKAGE_TYPE=${PACKAGE_TYPE:-container}\n"
            "gh api \"$OWNER_TYPE/${OWNER}/packages/${PACKAGE_TYPE}/"
            "${PACKAGE_NAME}/versions?per_page=100\" --paginate "
            "> /tmp/pk-package-versions.json\n"
            "jq -r --arg cutoff \"$cutoff\" '\n"
            "  .[] | select(.created_at < $cutoff) |\n"
            "  select((.metadata.container.tags // []) | length == 0) | .id\n"
            "' /tmp/pk-package-versions.json |\n"
            "while read -r id; do\n"
            "  gh api --method DELETE "
            "\"$OWNER_TYPE/${OWNER}/packages/${PACKAGE_TYPE}/"
            "${PACKAGE_NAME}/versions/${id}\"\n"
            "done"
        )
        return _runbook(
            ["OWNER", "PACKAGE_NAME"],
            inventory,
            dry_run,
            apply,
            space_estimate=(
                "GitHub package version listings do not always expose "
                "reclaimable bytes; dry-run prints the exact versions."
            ),
        )
    return None


def _gitlab_runbook(scope: str) -> dict | None:
    api = "${GITLAB_API_URL:-https://gitlab.com/api/v4}"
    if scope == "action-artifacts":
        inventory = (
            f"curl --header \"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/jobs?per_page=100\" "
            "> /tmp/pk-gitlab-jobs.json\n"
            "jq -r '.[] | select(.artifacts_file) | "
            "[.id,.name,.artifacts_file.size,.created_at] | @tsv' "
            "/tmp/pk-gitlab-jobs.json"
        )
        dry_run = (
            "RETENTION_DAYS=${RETENTION_DAYS:-60}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            f"curl --header \"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/jobs?per_page=100\" "
            "> /tmp/pk-gitlab-jobs.json\n"
            "jq -r --arg cutoff \"$cutoff\" '\n"
            "  .[] | select(.created_at < $cutoff) | "
            "select(.artifacts_file) |\n"
            "  [.id,.name,.artifacts_file.size,.created_at] | @tsv\n"
            "' /tmp/pk-gitlab-jobs.json\n"
            "jq --arg cutoff \"$cutoff\" '[.[] | select(.created_at < "
            "$cutoff) | .artifacts_file.size? // 0] | add // 0' "
            "/tmp/pk-gitlab-jobs.json"
        )
        apply = (
            "RETENTION_DAYS=${RETENTION_DAYS:-60}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            f"curl --header \"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/jobs?per_page=100\" "
            "> /tmp/pk-gitlab-jobs.json\n"
            "jq -r --arg cutoff \"$cutoff\" '.[] | select(.created_at < "
            "$cutoff) | select(.artifacts_file) | .id' "
            "/tmp/pk-gitlab-jobs.json |\n"
            "while read -r id; do\n"
            f"  curl --request DELETE --header "
            f"\"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/jobs/${{id}}/artifacts\"\n"
            "done"
        )
        return _runbook(["GITLAB_TOKEN", "PROJECT_ID"], inventory, dry_run, apply)

    if scope == "package-registry":
        inventory = (
            f"curl --header \"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/packages?per_page=100\" "
            "> /tmp/pk-gitlab-packages.json\n"
            "jq -r '.[] | [.id,.name,.version,.package_type,.created_at] | "
            "@tsv' /tmp/pk-gitlab-packages.json"
        )
        dry_run = (
            "RETENTION_DAYS=${RETENTION_DAYS:-90}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            f"curl --header \"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/packages?per_page=100\" "
            "> /tmp/pk-gitlab-packages.json\n"
            "jq -r --arg cutoff \"$cutoff\" '.[] | "
            "select(.created_at < $cutoff) | "
            "[.id,.name,.version,.package_type,.created_at] | @tsv' "
            "/tmp/pk-gitlab-packages.json"
        )
        apply = (
            "RETENTION_DAYS=${RETENTION_DAYS:-90}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            f"curl --header \"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/packages?per_page=100\" "
            "> /tmp/pk-gitlab-packages.json\n"
            "jq -r --arg cutoff \"$cutoff\" '.[] | "
            "select(.created_at < $cutoff) | .id' "
            "/tmp/pk-gitlab-packages.json |\n"
            "while read -r id; do\n"
            f"  curl --request DELETE --header "
            f"\"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/packages/${{id}}\"\n"
            "done"
        )
        return _runbook(
            ["GITLAB_TOKEN", "PROJECT_ID"],
            inventory,
            dry_run,
            apply,
            space_estimate=(
                "GitLab package listings may omit byte totals; dry-run "
                "prints exact package IDs and versions."
            ),
        )

    if scope == "release-assets":
        inventory = (
            f"curl --header \"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/releases?per_page=100\" "
            "> /tmp/pk-gitlab-releases.json\n"
            "jq -r '.[] | [.tag_name,.created_at,(.assets.links|length)] | "
            "@tsv' /tmp/pk-gitlab-releases.json"
        )
        dry_run = (
            "RETENTION_DAYS=${RETENTION_DAYS:-180}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            f"curl --header \"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/releases?per_page=100\" "
            "> /tmp/pk-gitlab-releases.json\n"
            "jq -r --arg cutoff \"$cutoff\" '.[] | "
            "select(.created_at < $cutoff) | "
            ".tag_name as $tag | .assets.links[]? | "
            "[$tag,.id,.name,.url] | @tsv' /tmp/pk-gitlab-releases.json"
        )
        apply = (
            "# GitLab release asset links can point to external or package "
            "assets; review dry-run output before deleting links.\n"
            "RETENTION_DAYS=${RETENTION_DAYS:-180}\n"
            "cutoff=$(date -u -d \"$RETENTION_DAYS days ago\" "
            "+%Y-%m-%dT%H:%M:%SZ)\n"
            f"curl --header \"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/releases?per_page=100\" "
            "> /tmp/pk-gitlab-releases.json\n"
            "jq -r --arg cutoff \"$cutoff\" '.[] | "
            "select(.created_at < $cutoff) | "
            ".tag_name as $tag | .assets.links[]? | [$tag,.id] | @tsv' "
            "/tmp/pk-gitlab-releases.json |\n"
            "while IFS=$'\\t' read -r tag link_id; do\n"
            f"  curl --request DELETE --header "
            f"\"PRIVATE-TOKEN: $GITLAB_TOKEN\" "
            f"\"{api}/projects/${{PROJECT_ID}}/releases/${{tag}}/assets/"
            "links/${link_id}\"\n"
            "done"
        )
        return _runbook(["GITLAB_TOKEN", "PROJECT_ID"], inventory, dry_run, apply)
    return None


def _forgejo_gitea_runbook(scope: str, provider: str) -> dict | None:
    api = "${FORGE_API_URL:-https://codeberg.org/api/v1}"
    required = ["FORGE_API_URL", "FORGE_TOKEN", "OWNER", "REPO"]
    auth = "Authorization: token $FORGE_TOKEN"
    if scope == "action-artifacts":
        inventory = (
            f"curl --header \"{auth}\" "
            f"\"{api}/repos/${{OWNER}}/${{REPO}}/actions/artifacts\" "
            "> /tmp/pk-forge-artifacts.json\n"
            "jq -r '.artifacts[]? | [.id,.name,.size_in_bytes,.created_at] | "
            "@tsv' /tmp/pk-forge-artifacts.json"
        )
        dry_run = inventory + "\n" + (
            "jq '[.artifacts[]?.size_in_bytes // 0] | add // 0' "
            "/tmp/pk-forge-artifacts.json"
        )
        apply = (
            "# Review the dry-run output before applying.\n"
            + inventory
            + "\n"
            "jq -r '.artifacts[]?.id' /tmp/pk-forge-artifacts.json |\n"
            "while read -r id; do\n"
            f"  curl --request DELETE --header \"{auth}\" "
            f"\"{api}/repos/${{OWNER}}/${{REPO}}/actions/artifacts/${{id}}\"\n"
            "done"
        )
        return _runbook(required, inventory, dry_run, apply)

    if scope == "release-assets":
        inventory = (
            f"curl --header \"{auth}\" "
            f"\"{api}/repos/${{OWNER}}/${{REPO}}/releases\" "
            "> /tmp/pk-forge-releases.json\n"
            "jq -r '.[] | [.tag_name,.created_at,(.assets|length)] | @tsv' "
            "/tmp/pk-forge-releases.json"
        )
        dry_run = (
            inventory
            + "\n"
            "jq -r '.[] | .tag_name as $tag | .assets[]? | "
            "[$tag,.id,.name,.size] | @tsv' /tmp/pk-forge-releases.json"
        )
        apply = (
            "# Review the dry-run output before applying.\n"
            + inventory
            + "\n"
            "jq -r '.[] | .tag_name as $tag | .assets[]? | [$tag,.id] | "
            "@tsv' /tmp/pk-forge-releases.json |\n"
            "while IFS=$'\\t' read -r tag asset_id; do\n"
            f"  curl --request DELETE --header \"{auth}\" "
            f"\"{api}/repos/${{OWNER}}/${{REPO}}/releases/tags/"
            "${tag}/assets/${asset_id}\"\n"
            "done"
        )
        return _runbook(required, inventory, dry_run, apply)

    if scope == "package-registry":
        inventory = (
            f"curl --header \"{auth}\" "
            f"\"{api}/packages/${{OWNER}}\" > /tmp/pk-forge-packages.json\n"
            "jq -r '.[] | [.type,.name,.version,.created_at] | @tsv' "
            "/tmp/pk-forge-packages.json"
        )
        dry_run = inventory
        apply = (
            "# Set PACKAGE_TYPE, PACKAGE_NAME, and PACKAGE_VERSION from the "
            "dry-run output before applying one package version.\n"
            f"curl --request DELETE --header \"{auth}\" "
            f"\"{api}/packages/${{OWNER}}/${{PACKAGE_TYPE}}/"
            "${PACKAGE_NAME}/${PACKAGE_VERSION}\""
        )
        return _runbook(
            required + ["PACKAGE_TYPE", "PACKAGE_NAME", "PACKAGE_VERSION"],
            inventory,
            dry_run,
            apply,
            space_estimate=(
                f"{provider} package listings may not expose byte totals; "
                "dry-run prints package coordinates."
            ),
        )
    return None


def _runbook(
    required_env: list[str],
    inventory_command: str | None,
    dry_run_command: str | None,
    apply_command: str | None,
    *,
    space_estimate: str = (
        "Dry-run prints selected rows and a provider-reported byte total "
        "when the API exposes sizes."
    ),
) -> dict:
    return {
        "required_env": required_env,
        "inventory_command": inventory_command,
        "dry_run_command": dry_run_command,
        "apply_command": apply_command,
        "space_estimate": space_estimate,
    }


def _remote_dry_run_command(scope: str, provider: str = "unknown") -> str | None:
    runbook = _remote_runbook(scope, provider)
    return runbook.get("dry_run_command") if runbook else None


def _remote_apply_command(scope: str, provider: str = "unknown") -> str | None:
    runbook = _remote_runbook(scope, provider)
    return runbook.get("apply_command") if runbook else None


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
        provider = item.get("provider", {}).get(
            "provider",
            analysis.get("repo_provider", {}).get("provider", "unknown"),
        )
        runbook = _remote_runbook(item["scope"], provider)
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
            "provider": item.get("provider"),
            "required_env": (
                runbook.get("required_env") if runbook else []
            ),
            "inventory_command": (
                runbook.get("inventory_command") if runbook else None
            ),
            "dry_run_command": (
                runbook.get("dry_run_command") if runbook else None
            ),
            "apply_command": runbook.get("apply_command") if runbook else None,
            "space_estimate": (
                runbook.get("space_estimate") if runbook else None
            ),
            "notes": item.get("notes", []),
        })

    return {
        "project_root": root.as_posix(),
        "scopes": normalized,
        "repo_provider": analysis.get("repo_provider"),
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
