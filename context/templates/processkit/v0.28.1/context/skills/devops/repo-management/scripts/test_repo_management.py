"""Tests for the repo-management MCP server helpers."""
from __future__ import annotations

import importlib.util
import os
import subprocess
from pathlib import Path


SERVER_PATH = Path(__file__).resolve().parents[1] / "mcp" / "server.py"
PROJECT_ROOT = Path(__file__).resolve()
while PROJECT_ROOT.parent != PROJECT_ROOT:
    if (PROJECT_ROOT / "AGENTS.md").exists():
        break
    PROJECT_ROOT = PROJECT_ROOT.parent


def _load_server():
    lib = PROJECT_ROOT / "src" / "context" / "skills" / "_lib"
    os.environ["PROCESSKIT_LIB_PATH"] = str(lib)
    spec = importlib.util.spec_from_file_location("repo_management_server", SERVER_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)  # type: ignore[union-attr]
    return module


server = _load_server()


def _git(root: Path, *args: str) -> None:
    subprocess.run(["git", *args], cwd=root, check=True, capture_output=True, text=True)


def test_detects_github_remote(tmp_path):
    _git(tmp_path, "init")
    _git(tmp_path, "remote", "add", "origin", "git@github.com:owner/repo.git")

    info = server.detect_repo_provider(project_root=tmp_path.as_posix())

    assert info["provider"] == "github"
    assert info["host"] == "github.com"
    assert info["repo_slug"] == "owner/repo"
    assert info["cli"] == "gh"


def test_detects_provider_matrix(tmp_path):
    cases = [
        ("https://gitlab.com/group/project.git", "gitlab"),
        ("git@codeberg.org:owner/project.git", "forgejo"),
        ("https://bitbucket.org/team/project.git", "bitbucket-cloud"),
        ("https://dev.azure.com/org/project/_git/repo", "azure-devops"),
        ("git@git.sr.ht:~owner/project", "sourcehut"),
    ]
    for index, (url, provider) in enumerate(cases):
        repo = tmp_path / f"repo-{index}"
        repo.mkdir()
        _git(repo, "init")
        _git(repo, "remote", "add", "origin", url)

        info = server.detect_repo_provider(project_root=repo.as_posix())

        assert info["provider"] == provider


def test_inspect_repo_state_reports_dirty_files(tmp_path):
    _git(tmp_path, "init")
    _git(tmp_path, "config", "user.email", "test@example.com")
    _git(tmp_path, "config", "user.name", "Test User")
    (tmp_path / "README.md").write_text("hello\n", encoding="utf-8")

    state = server.inspect_repo_state(project_root=tmp_path.as_posix())

    assert state["is_git_repo"] is True
    assert state["dirty"] is True
    assert state["status_count"] == 1


def test_plan_reports_push_blocker_without_upstream(tmp_path):
    _git(tmp_path, "init")
    _git(tmp_path, "config", "user.email", "test@example.com")
    _git(tmp_path, "config", "user.name", "Test User")
    (tmp_path / "README.md").write_text("hello\n", encoding="utf-8")
    _git(tmp_path, "add", "README.md")
    _git(tmp_path, "commit", "-m", "initial")

    plan = server.plan_repo_reconcile(
        project_root=tmp_path.as_posix(),
        scope="push",
    )

    assert plan["blockers"]
    assert plan["blockers"][0]["kind"] == "push"
    assert "no upstream" in plan["blockers"][0]["reason"]


def test_unsupported_non_github_issue_listing(tmp_path):
    _git(tmp_path, "init")
    _git(tmp_path, "remote", "add", "origin", "https://gitlab.com/group/project.git")

    result = server.list_repo_issues(project_root=tmp_path.as_posix())

    assert result["unsupported"] is True
    assert result["provider"]["provider"] == "gitlab"
