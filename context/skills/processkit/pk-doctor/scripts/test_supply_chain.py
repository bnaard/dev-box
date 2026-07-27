"""Focused tests for the pk-doctor supply-chain check."""

from __future__ import annotations

import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parent))

from checks.supply_chain import run  # noqa: E402


def test_nested_git_repositories_are_excluded(tmp_path: Path) -> None:
    nested_repo = tmp_path / "themes" / "vendored"
    nested_repo.mkdir(parents=True)
    (nested_repo / ".git").write_text("gitdir: elsewhere\n", encoding="utf-8")
    (nested_repo / "package.json").write_text(
        '{"name": "vendored-theme"}\n',
        encoding="utf-8",
    )

    findings = run({"repo_root": tmp_path, "since_files": None})
    missing_refs = {
        item.entity_ref
        for item in findings
        if item.id == "supply_chain.missing-lockfile"
    }

    assert "themes/vendored/package.json" not in missing_refs
