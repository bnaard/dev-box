#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
# ]
# ///
"""Provider-neutral repository management MCP server."""
from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Optional
from urllib.parse import urlparse

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

server = FastMCP("processkit-repo-management")

SUPPORTED_SCOPES = {
    "all",
    "local",
    "push",
    "issues",
    "prs",
    "change-requests",
}

PROVIDER_CLI = {
    "github": "gh",
    "gitlab": "glab",
    "gitea": "tea",
    "forgejo": "tea",
    "bitbucket-cloud": "bb",
    "azure-devops": "az",
    "sourcehut": "hut",
}

PROVIDER_LABELS = {
    "github": "GitHub",
    "gitlab": "GitLab",
    "gitea": "Gitea",
    "forgejo": "Forgejo / Codeberg",
    "bitbucket-cloud": "Bitbucket Cloud",
    "azure-devops": "Azure DevOps",
    "sourcehut": "SourceHut",
    "unknown": "Unknown",
}


def _project_root(project_root: Optional[str] = None) -> Path:
    if project_root:
        root = Path(project_root).expanduser().resolve()
        if not root.exists() or not root.is_dir():
            raise ValueError(f"project_root does not exist: {project_root}")
        return root

    here = Path.cwd().resolve()
    for candidate in [here, *here.parents]:
        if (candidate / ".git").exists():
            return candidate
    return here


def _run(
    args: list[str],
    *,
    cwd: Path,
    timeout: int = 30,
    input_text: Optional[str] = None,
) -> dict:
    try:
        completed = subprocess.run(
            args,
            cwd=cwd,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
            input=input_text,
        )
    except FileNotFoundError:
        return {
            "ok": False,
            "returncode": 127,
            "command": args,
            "stdout": "",
            "stderr": f"command not found: {args[0]}",
        }
    except subprocess.TimeoutExpired as exc:
        return {
            "ok": False,
            "returncode": 124,
            "command": args,
            "stdout": exc.stdout or "",
            "stderr": exc.stderr or "command timed out",
        }
    return {
        "ok": completed.returncode == 0,
        "returncode": completed.returncode,
        "command": args,
        "stdout": completed.stdout,
        "stderr": completed.stderr,
    }


def _git(root: Path, *args: str, timeout: int = 30) -> dict:
    return _run(["git", *args], cwd=root, timeout=timeout)


def _git_output(root: Path, *args: str) -> Optional[str]:
    result = _git(root, *args)
    if not result["ok"]:
        return None
    return result["stdout"].strip()


def _origin_url(root: Path, remote: str = "origin") -> Optional[str]:
    return _git_output(root, "remote", "get-url", remote)


def _parse_remote_url(url: str) -> dict:
    host = ""
    path = ""
    if "://" in url:
        parsed = urlparse(url)
        host = parsed.hostname or ""
        path = parsed.path.lstrip("/")
    else:
        scp_like = re.match(r"(?:(?P<user>[^@]+)@)?(?P<host>[^:]+):(?P<path>.+)", url)
        if scp_like:
            host = scp_like.group("host")
            path = scp_like.group("path")
        else:
            parsed = urlparse("ssh://" + url)
            host = parsed.hostname or ""
            path = parsed.path.lstrip("/")
    if path.endswith(".git"):
        path = path[:-4]
    parts = [part for part in path.split("/") if part]
    owner = "/".join(parts[:-1]) if len(parts) >= 2 else None
    repo = parts[-1] if parts else None
    return {
        "remote_url": url,
        "host": host.lower(),
        "path": path,
        "owner": owner,
        "repo": repo,
        "repo_slug": f"{owner}/{repo}" if owner and repo else None,
    }


def _provider_from_host(host: str) -> str:
    override = os.environ.get("REPO_PROVIDER", "").strip().lower()
    if override:
        aliases = {
            "bitbucket": "bitbucket-cloud",
            "azure": "azure-devops",
            "azdo": "azure-devops",
            "codeberg": "forgejo",
        }
        return aliases.get(override, override)
    if host == "github.com" or host.endswith(".github.com"):
        return "github"
    if host == "gitlab.com" or host.endswith(".gitlab.com"):
        return "gitlab"
    if host == "codeberg.org":
        return "forgejo"
    if host == "bitbucket.org":
        return "bitbucket-cloud"
    if host == "dev.azure.com" or host.endswith(".visualstudio.com"):
        return "azure-devops"
    if host == "git.sr.ht":
        return "sourcehut"
    return "unknown"


def _provider_info(root: Path, remote: str = "origin") -> dict:
    url = _origin_url(root, remote)
    if not url:
        return {
            "provider": "unknown",
            "provider_label": PROVIDER_LABELS["unknown"],
            "remote": remote,
            "remote_url": None,
            "host": None,
            "repo_slug": None,
            "cli": None,
            "cli_available": False,
            "supports_remote_issues": False,
            "supports_remote_change_requests": False,
            "supports_remote_mutation": False,
            "notes": ["remote URL unavailable"],
        }

    parsed = _parse_remote_url(url)
    provider = _provider_from_host(parsed["host"])
    cli = PROVIDER_CLI.get(provider)
    cli_available = shutil.which(cli) is not None if cli else False
    github_ready = provider == "github" and cli_available
    notes = []
    if provider == "unknown":
        notes.append("provider not recognized from remote host")
    elif provider != "github":
        notes.append(
            "remote issue/change-request adapter reports capability only "
            "until provider CLI support is configured"
        )
    elif not cli_available:
        notes.append("gh CLI is unavailable")

    return {
        **parsed,
        "provider": provider,
        "provider_label": PROVIDER_LABELS.get(provider, provider),
        "remote": remote,
        "cli": cli,
        "cli_available": cli_available,
        "supports_remote_issues": github_ready,
        "supports_remote_change_requests": github_ready,
        "supports_remote_mutation": github_ready,
        "supports_local_git": shutil.which("git") is not None,
        "notes": notes,
    }


def _parse_porcelain(text: str) -> list[dict]:
    entries = []
    for line in text.splitlines():
        if not line:
            continue
        status = line[:2]
        path = line[3:] if len(line) > 3 else ""
        entries.append({"status": status, "path": path})
    return entries


def _ahead_behind(root: Path) -> dict:
    upstream = _git_output(root, "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}")
    if not upstream:
        return {
            "upstream": None,
            "ahead": None,
            "behind": None,
            "has_upstream": False,
        }
    counts = _git_output(root, "rev-list", "--left-right", "--count", f"HEAD...{upstream}")
    if not counts:
        return {
            "upstream": upstream,
            "ahead": None,
            "behind": None,
            "has_upstream": True,
        }
    ahead, behind = [int(part) for part in counts.split()[:2]]
    return {
        "upstream": upstream,
        "ahead": ahead,
        "behind": behind,
        "has_upstream": True,
    }


def _repo_state(project_root: Optional[str] = None, include_remote: bool = True) -> dict:
    root = _project_root(project_root)
    git_dir = _git_output(root, "rev-parse", "--git-dir")
    if git_dir is None:
        return {
            "project_root": root.as_posix(),
            "is_git_repo": False,
            "error": "not a git repository",
        }

    branch = _git_output(root, "branch", "--show-current") or None
    head = _git_output(root, "rev-parse", "--short", "HEAD") or None
    status_text = _git_output(root, "status", "--porcelain=v1") or ""
    status_entries = _parse_porcelain(status_text)
    remotes_text = _git_output(root, "remote", "-v") or ""
    remotes = []
    for line in remotes_text.splitlines():
        parts = line.split()
        if len(parts) >= 3:
            remotes.append({"name": parts[0], "url": parts[1], "kind": parts[2].strip("()")})

    remote_state = _ahead_behind(root) if include_remote else {
        "upstream": None,
        "ahead": None,
        "behind": None,
        "has_upstream": False,
    }
    provider = _provider_info(root)
    return {
        "project_root": root.as_posix(),
        "is_git_repo": True,
        "branch": branch,
        "head": head,
        "dirty": bool(status_entries),
        "status_count": len(status_entries),
        "status_entries": status_entries,
        "remotes": remotes,
        "remote_state": remote_state,
        "provider": provider,
    }


def _gh_repo_args(info: dict) -> list[str]:
    return ["--repo", info["repo_slug"]] if info.get("repo_slug") else []


def _json_or_error(result: dict) -> object:
    if not result["ok"]:
        return {
            "error": "command_failed",
            "command": result["command"],
            "returncode": result["returncode"],
            "stderr": result["stderr"].strip(),
        }
    try:
        return json.loads(result["stdout"] or "[]")
    except json.JSONDecodeError as exc:
        return {
            "error": "invalid_json",
            "message": str(exc),
            "stdout": result["stdout"],
        }


def _list_github_issues(root: Path, state: str, limit: int) -> dict:
    info = _provider_info(root)
    if not info["supports_remote_issues"]:
        return _unsupported("list_repo_issues", info)
    args = [
        "gh", "issue", "list", "--state", state, "--limit", str(limit),
        "--json", "number,title,state,labels,updatedAt,url",
        *_gh_repo_args(info),
    ]
    data = _json_or_error(_run(args, cwd=root, timeout=30))
    if isinstance(data, dict) and data.get("error"):
        return data
    return {
        "provider": info,
        "state": state,
        "count": len(data),  # type: ignore[arg-type]
        "issues": data,
    }


def _list_github_change_requests(root: Path, state: str, limit: int) -> dict:
    info = _provider_info(root)
    if not info["supports_remote_change_requests"]:
        return _unsupported("list_repo_change_requests", info)
    args = [
        "gh", "pr", "list", "--state", state, "--limit", str(limit),
        "--json",
        "number,title,state,isDraft,reviewDecision,statusCheckRollup,updatedAt,url",
        *_gh_repo_args(info),
    ]
    data = _json_or_error(_run(args, cwd=root, timeout=30))
    if isinstance(data, dict) and data.get("error"):
        return data
    return {
        "provider": info,
        "state": state,
        "count": len(data),  # type: ignore[arg-type]
        "change_requests": data,
    }


def _unsupported(operation: str, info: dict) -> dict:
    return {
        "unsupported": True,
        "operation": operation,
        "provider": info,
        "reason": (
            f"{operation} is not implemented for "
            f"{info.get('provider_label', 'this provider')} in this adapter"
        ),
    }


def _normalize_scope(scope: Optional[str]) -> list[str]:
    value = (scope or "all").strip().lower()
    if value not in SUPPORTED_SCOPES:
        raise ValueError(
            f"unsupported scope {value!r}; supported scopes: "
            f"{', '.join(sorted(SUPPORTED_SCOPES))}"
        )
    if value == "all":
        return ["local", "push", "issues", "prs"]
    if value == "change-requests":
        return ["prs"]
    return [value]


def _checks_green(change_request: dict) -> bool:
    checks = change_request.get("statusCheckRollup") or []
    if not checks:
        return False
    for check in checks:
        conclusion = str(check.get("conclusion") or check.get("status") or "").upper()
        if conclusion not in {"SUCCESS", "COMPLETED", "NEUTRAL", "SKIPPED"}:
            return False
    return True


def _plan(project_root: Optional[str], scope: Optional[str], limit: int) -> dict:
    scopes = _normalize_scope(scope)
    state = _repo_state(project_root)
    actions: list[dict] = []
    blockers: list[dict] = []
    observations: list[dict] = []

    if not state.get("is_git_repo"):
        blockers.append({"kind": "local", "reason": state.get("error")})
        return {
            "scope": scopes,
            "state": state,
            "actions": actions,
            "blockers": blockers,
            "observations": observations,
        }

    remote_state = state["remote_state"]
    if "local" in scopes:
        if state["dirty"]:
            actions.append({
                "kind": "commit_local_changes",
                "safe_to_apply": False,
                "reason": "worktree has uncommitted changes; commit message and path scope required",
                "status_count": state["status_count"],
            })
        else:
            observations.append({"kind": "local", "message": "worktree clean"})

    if "push" in scopes:
        ahead = remote_state.get("ahead")
        behind = remote_state.get("behind")
        if not remote_state.get("has_upstream"):
            blockers.append({"kind": "push", "reason": "current branch has no upstream"})
        elif behind and behind > 0:
            blockers.append({
                "kind": "push",
                "reason": "branch is behind upstream; merge or rebase first",
                "behind": behind,
            })
        elif ahead and ahead > 0:
            actions.append({
                "kind": "push_current_branch",
                "safe_to_apply": True,
                "ahead": ahead,
                "upstream": remote_state.get("upstream"),
            })
        else:
            observations.append({"kind": "push", "message": "nothing to push"})

    if "issues" in scopes:
        issues = list_repo_issues(project_root=project_root, state="open", limit=limit)
        if issues.get("unsupported") or issues.get("error"):
            blockers.append({"kind": "issues", "reason": issues})
        else:
            count = issues.get("count", 0)
            observations.append({"kind": "issues", "open_count": count})
            for issue in issues.get("issues", []):
                actions.append({
                    "kind": "review_issue",
                    "safe_to_apply": False,
                    "id": issue.get("number"),
                    "title": issue.get("title"),
                    "reason": "needs evidence before comment or close",
                })

    if "prs" in scopes:
        crs = list_repo_change_requests(project_root=project_root, state="open", limit=limit)
        if crs.get("unsupported") or crs.get("error"):
            blockers.append({"kind": "change_requests", "reason": crs})
        else:
            observations.append({
                "kind": "change_requests",
                "open_count": crs.get("count", 0),
            })
            for cr in crs.get("change_requests", []):
                draft = bool(cr.get("isDraft"))
                approved = cr.get("reviewDecision") in (None, "APPROVED")
                checks_green = _checks_green(cr)
                actions.append({
                    "kind": "merge_change_request",
                    "safe_to_apply": (not draft) and approved and checks_green,
                    "id": cr.get("number"),
                    "title": cr.get("title"),
                    "confirmation": f"merge-change-request:{cr.get('number')}",
                    "blockers": [
                        reason for reason, blocked in [
                            ("draft", draft),
                            ("not approved", not approved),
                            ("checks not green", not checks_green),
                        ] if blocked
                    ],
                })

    return {
        "scope": scopes,
        "state": state,
        "actions": actions,
        "blockers": blockers,
        "observations": observations,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def detect_repo_provider(
    project_root: Optional[str] = None,
    remote: str = "origin",
) -> dict:
    """Detect the repository provider from a git remote."""
    return _provider_info(_project_root(project_root), remote)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def inspect_repo_state(
    project_root: Optional[str] = None,
    include_remote: bool = True,
) -> dict:
    """Inspect local git state, remotes, branch, and provider support."""
    return _repo_state(project_root, include_remote)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=True,
))
def list_repo_issues(
    project_root: Optional[str] = None,
    state: str = "open",
    limit: int = 50,
) -> dict:
    """List repository issues for the detected provider."""
    root = _project_root(project_root)
    info = _provider_info(root)
    if info["provider"] == "github":
        return _list_github_issues(root, state, limit)
    return _unsupported("list_repo_issues", info)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=True,
))
def list_repo_change_requests(
    project_root: Optional[str] = None,
    state: str = "open",
    limit: int = 50,
) -> dict:
    """List repository pull requests, merge requests, or change requests."""
    root = _project_root(project_root)
    info = _provider_info(root)
    if info["provider"] == "github":
        return _list_github_change_requests(root, state, limit)
    return _unsupported("list_repo_change_requests", info)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=True,
))
def plan_repo_reconcile(
    project_root: Optional[str] = None,
    scope: str = "all",
    dry_run: bool = True,
    limit: int = 50,
) -> dict:
    """Build a repository reconciliation plan without mutating state."""
    plan = _plan(project_root, scope, limit)
    plan["dry_run"] = dry_run
    return plan


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=True,
))
def resolve_repo_issue(
    id: int,
    resolution: str,
    comment: Optional[str] = None,
    close: bool = False,
    confirmation: Optional[str] = None,
    project_root: Optional[str] = None,
) -> dict:
    """Comment on and optionally close a supported issue."""
    root = _project_root(project_root)
    info = _provider_info(root)
    if info["provider"] != "github" or not info["supports_remote_mutation"]:
        return _unsupported("resolve_repo_issue", info)
    if close and confirmation != f"close-issue:{id}":
        return {
            "error": "confirmation_required",
            "required_confirmation": f"close-issue:{id}",
        }
    body = comment or resolution
    results = []
    if body:
        results.append(_run([
            "gh", "issue", "comment", str(id), "--body", body,
            *_gh_repo_args(info),
        ], cwd=root, timeout=30))
    if close:
        results.append(_run([
            "gh", "issue", "close", str(id), "--comment", resolution,
            *_gh_repo_args(info),
        ], cwd=root, timeout=30))
    return {
        "ok": all(result["ok"] for result in results),
        "provider": info,
        "issue": id,
        "closed": close,
        "results": results,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=True,
))
def merge_change_request(
    id: int,
    method: str = "squash",
    auto_queue: bool = True,
    confirmation: Optional[str] = None,
    project_root: Optional[str] = None,
) -> dict:
    """Merge or queue a supported change request without bypassing checks."""
    root = _project_root(project_root)
    info = _provider_info(root)
    if info["provider"] != "github" or not info["supports_remote_mutation"]:
        return _unsupported("merge_change_request", info)
    if confirmation != f"merge-change-request:{id}":
        return {
            "error": "confirmation_required",
            "required_confirmation": f"merge-change-request:{id}",
        }
    if method not in {"merge", "squash", "rebase"}:
        return {"error": "unsupported_merge_method", "method": method}
    args = ["gh", "pr", "merge", str(id), f"--{method}", "--delete-branch"]
    if auto_queue:
        args.append("--auto")
    args.extend(_gh_repo_args(info))
    result = _run(args, cwd=root, timeout=60)
    return {
        "ok": result["ok"],
        "provider": info,
        "change_request": id,
        "result": result,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def commit_local_changes(
    message: str,
    paths: Optional[list[str]] = None,
    checks: Optional[list[list[str]]] = None,
    project_root: Optional[str] = None,
) -> dict:
    """Run optional checks, stage paths, and create a git commit."""
    root = _project_root(project_root)
    if not message.strip():
        return {"error": "commit message required"}
    check_results = []
    for check in checks or []:
        if not check:
            continue
        result = _run([str(part) for part in check], cwd=root, timeout=300)
        check_results.append(result)
        if not result["ok"]:
            return {
                "ok": False,
                "error": "check_failed",
                "failed_check": result,
                "checks": check_results,
            }
    stage_args = ["git", "add", "--"]
    stage_args.extend(paths or ["."])
    stage = _run(stage_args, cwd=root, timeout=30)
    if not stage["ok"]:
        return {"ok": False, "error": "stage_failed", "stage": stage}
    commit = _run(["git", "commit", "-m", message], cwd=root, timeout=120)
    return {
        "ok": commit["ok"],
        "stage": stage,
        "commit": commit,
        "checks": check_results,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=True,
))
def push_current_branch(
    remote: str = "origin",
    branch: Optional[str] = None,
    project_root: Optional[str] = None,
) -> dict:
    """Push the current branch to its configured remote without force."""
    root = _project_root(project_root)
    selected_branch = branch or _git_output(root, "branch", "--show-current")
    if not selected_branch:
        return {"error": "branch_required"}
    result = _run(["git", "push", remote, selected_branch], cwd=root, timeout=120)
    return {
        "ok": result["ok"],
        "remote": remote,
        "branch": selected_branch,
        "result": result,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=True,
))
def run_repo_reconcile(
    project_root: Optional[str] = None,
    dry_run: bool = True,
    max_items: int = 50,
    scope: str = "all",
    commit_message: Optional[str] = None,
    push: bool = False,
) -> dict:
    """Apply guarded repository reconciliation actions."""
    plan = _plan(project_root, scope, max_items)
    if dry_run:
        return {"dry_run": True, "plan": plan, "applied": []}
    root = _project_root(project_root)
    applied = []
    if commit_message and any(
        action["kind"] == "commit_local_changes"
        for action in plan["actions"]
    ):
        applied.append(commit_local_changes(
            message=commit_message,
            project_root=root.as_posix(),
        ))
        plan = _plan(project_root, scope, max_items)
    if push:
        remote_state = plan["state"].get("remote_state", {})
        if remote_state.get("ahead", 0) and not remote_state.get("behind", 0):
            applied.append(push_current_branch(project_root=root.as_posix()))
    return {
        "dry_run": False,
        "plan": plan,
        "applied": applied,
        "note": (
            "Issue close and change-request merge actions require explicit "
            "narrow tool calls with confirmation tokens."
        ),
    }


if __name__ == "__main__":
    server.run(transport="stdio")
