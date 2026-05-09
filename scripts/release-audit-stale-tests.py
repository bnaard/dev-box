#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""release-audit-stale-tests.py — flag tests with hardcoded layout assumptions.

Project-local extension to the upstream `pk-release-audit` skill. Sweeps test
files for hardcoded references to v0.25.x-specific tmux layout names, slot
labels, window/pane indices, and harness identifiers — all of which would
silently break under SnappyWolf 4c+4b parameterizable layouts (workitems
SnappyWolf, SilentFjord).

Goal: surface tests that pass under the current layout but assert on magic
strings that will not survive the SnappyWolf refactor.

Detect-only. Never modifies anything. Exit 0 always — this is an awareness
check, not a release blocker yet (will be promoted to ERROR once SnappyWolf
lands).

Usage:
    scripts/release-audit-stale-tests.py [--repo-root=PATH] [--baseline=PATH]
                                         [--check] [--update-baseline]

Modes:
    default           : print all hits, comparing against the baseline.
    --check           : exit 1 if hits diverge from the baseline (CI gate).
    --update-baseline : rewrite the baseline file with the current hit list.

Provenance: WorkItem BACK-20260509_1316-TallBear (v0.25.7 / track T2).
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

AUDIT_VERSION = "0.1.0"

# ---------------------------------------------------------------------------
# Hit categories
# ---------------------------------------------------------------------------
#
# Each category has a name, a description, a regex to match against each line
# of each test file, and an optional anti-regex to suppress false positives
# (e.g. comments that explicitly mention the magic string in prose).
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Category:
    name: str
    description: str
    pattern: re.Pattern[str]
    suppress: re.Pattern[str] | None = None


CATEGORIES: list[Category] = [
    Category(
        name="layout-name-literal",
        description=(
            "Hardcoded v0.25.x layout name as a string literal. SnappyWolf "
            "will rename / parameterize layouts; literals will silently miss."
        ),
        pattern=re.compile(
            r'"(dev|focus|browse|cowork|cowork-swap|ai)"'
        ),
        # Suppress lines that are obviously comment-only prose mentioning the
        # name (a // or # before the literal on the same line).
        suppress=re.compile(r'^\s*(//|#)'),
    ),
    Category(
        name="slot-label-literal",
        description=(
            "Hardcoded statusline slot label (OOM/LOG/PROC/AI/MCP/MIG/MEM). "
            "SilentFjord changes the OOM slot; tests asserting on the "
            "exact label will break."
        ),
        pattern=re.compile(
            r'"\s*(OOM|LOG|PROC|AI|MCP|MIG|MEM)\s+'
        ),
        suppress=re.compile(r'^\s*(//|#)'),
    ),
    Category(
        name="harness-name-literal",
        description=(
            "Hardcoded harness identifier (claude/codex/hermes/opencode) "
            "in test fixture data. SnappyWolf 4c/4b allows arbitrary harness "
            "sets; literals will not survive."
        ),
        pattern=re.compile(
            r'"(claude|codex|hermes|opencode)"'
        ),
        suppress=re.compile(r'^\s*(//|#)'),
    ),
    Category(
        name="tmux-index-literal",
        description=(
            "Hardcoded tmux window/pane index assertion. Slot order is "
            "changing under SilentFjord/SnappyWolf; index-based asserts "
            "will quietly desync."
        ),
        pattern=re.compile(
            r'(select-window|select-pane|kill-window|kill-pane)\s+-t\s+\S*[0-9]'
        ),
        suppress=re.compile(r'^\s*(//|#)'),
    ),
    Category(
        name="layout-tuple",
        description=(
            "Array/tuple/slice listing the full v0.25.x layout set. Will "
            "drift the moment SnappyWolf adds or renames a layout."
        ),
        pattern=re.compile(
            r'\[.*"dev".*"focus".*"cowork".*\]'
            r'|\[.*"cowork-swap".*"browse".*"ai".*\]'
        ),
        suppress=re.compile(r'^\s*(//|#)'),
    ),
]

# File-path globs that constitute "tests" in this project.
TEST_GLOBS: list[str] = [
    "cli/tests/**/*.rs",
    "cli/tests/**/*.sh",
    "cli/tests/**/*.py",
    "scripts/**/*test*.sh",
    "scripts/**/*test*.py",
    "tests/**/*.rs",
    "tests/**/*.sh",
    "tests/**/*.py",
    "**/*_test.rs",
    "**/*_test.py",
    "**/*_test.sh",
]

# Specific files that match TEST_GLOBS by name but are not actually tests
# (this file matches scripts/**/*test*.py because of "stale-tests" in the
# filename — exclude itself so it doesn't self-flag on regex source lines).
EXCLUDE_FILES: frozenset[str] = frozenset({
    "scripts/release-audit-stale-tests.py",
})

# Directories to never descend into.
EXCLUDE_DIRS: frozenset[str] = frozenset({
    ".git",
    "target",
    "node_modules",
    "dist",
    "context/templates",  # read-only upstream mirror
    ".claude/worktrees",
})


@dataclass(frozen=True)
class Hit:
    category: str
    path: str
    lineno: int
    line: str

    def render(self) -> str:
        return f"{self.category}\t{self.path}:{self.lineno}\t{self.line.rstrip()}"


def _is_excluded(path: Path, repo_root: Path) -> bool:
    try:
        rel = path.relative_to(repo_root)
    except ValueError:
        return True
    parts = rel.parts
    for i in range(len(parts)):
        prefix = "/".join(parts[: i + 1])
        if prefix in EXCLUDE_DIRS:
            return True
        if parts[i] in EXCLUDE_DIRS:
            return True
    return False


def _iter_test_files(repo_root: Path) -> Iterable[Path]:
    seen: set[Path] = set()
    for glob in TEST_GLOBS:
        for path in repo_root.glob(glob):
            if not path.is_file():
                continue
            if _is_excluded(path, repo_root):
                continue
            try:
                rel = path.relative_to(repo_root).as_posix()
            except ValueError:
                continue
            if rel in EXCLUDE_FILES:
                continue
            if path in seen:
                continue
            seen.add(path)
            yield path


def scan(repo_root: Path) -> list[Hit]:
    hits: list[Hit] = []
    for path in sorted(_iter_test_files(repo_root)):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(text.splitlines(), start=1):
            for cat in CATEGORIES:
                if cat.suppress is not None and cat.suppress.search(line):
                    continue
                if cat.pattern.search(line):
                    hits.append(Hit(
                        category=cat.name,
                        path=rel,
                        lineno=lineno,
                        line=line,
                    ))
                    # one category per line is enough; first-match wins so
                    # the report stays compact.
                    break
    return hits


# ---------------------------------------------------------------------------
# Baseline I/O
# ---------------------------------------------------------------------------


def _baseline_lines(hits: list[Hit]) -> list[str]:
    """Render hits in a stable, diffable form (path:lineno\\tcategory)."""
    lines = sorted(
        f"{h.path}:{h.lineno}\t{h.category}"
        for h in hits
    )
    return lines


def _read_baseline(baseline_path: Path) -> list[str] | None:
    if not baseline_path.is_file():
        return None
    return [
        ln.rstrip("\n")
        for ln in baseline_path.read_text(encoding="utf-8").splitlines()
        if ln.strip() and not ln.startswith("#")
    ]


def _write_baseline(baseline_path: Path, hits: list[Hit]) -> None:
    body = [
        "# release-audit-stale-tests baseline",
        "# Generated by scripts/release-audit-stale-tests.py --update-baseline",
        "# Each line: <path>:<lineno>\\t<category>",
        "# Provenance: WorkItem BACK-20260509_1316-TallBear (v0.25.7 / track T2).",
        "",
    ]
    body.extend(_baseline_lines(hits))
    body.append("")
    baseline_path.parent.mkdir(parents=True, exist_ok=True)
    baseline_path.write_text("\n".join(body), encoding="utf-8")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def _resolve_repo_root(explicit: str | None) -> Path:
    if explicit:
        return Path(explicit).resolve()
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True, text=True, check=True,
        )
        return Path(out.stdout.strip()).resolve()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return Path.cwd()


def _format_report(hits: list[Hit], repo_root: Path, baseline: list[str] | None) -> str:
    lines: list[str] = []
    lines.append(f"# release-audit-stale-tests v{AUDIT_VERSION}")
    lines.append(f"repo_root: {repo_root}")
    lines.append(f"hits: {len(hits)}")
    lines.append("")

    by_cat: dict[str, list[Hit]] = {}
    for h in hits:
        by_cat.setdefault(h.category, []).append(h)

    for cat in CATEGORIES:
        cat_hits = by_cat.get(cat.name, [])
        lines.append(f"## {cat.name} — {len(cat_hits)} hit(s)")
        lines.append(f"  > {cat.description}")
        for h in cat_hits:
            snippet = h.line.strip()
            if len(snippet) > 120:
                snippet = snippet[:117] + "..."
            lines.append(f"  [!] {h.path}:{h.lineno}  {snippet}")
        lines.append("")

    if baseline is not None:
        current_lines = set(_baseline_lines(hits))
        baseline_lines = set(baseline)
        added = sorted(current_lines - baseline_lines)
        removed = sorted(baseline_lines - current_lines)
        lines.append(
            f"## baseline diff — {len(added)} added / {len(removed)} removed"
        )
        for entry in added:
            lines.append(f"  [+] {entry}")
        for entry in removed:
            lines.append(f"  [-] {entry}")
        lines.append("")
    else:
        lines.append("## baseline diff — (no baseline file present)")
        lines.append("")

    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(
        prog="release-audit-stale-tests",
        description=(
            "Flag tests with hardcoded v0.25.x layout/slot/harness assumptions."
        ),
    )
    p.add_argument("--repo-root", default=None)
    p.add_argument(
        "--baseline",
        default=None,
        help=(
            "Path to baseline file (default: "
            "context/artifacts/release-audit-stale-tests.baseline)."
        ),
    )
    p.add_argument(
        "--check",
        action="store_true",
        help="Exit 1 if hits diverge from the baseline.",
    )
    p.add_argument(
        "--update-baseline",
        action="store_true",
        help="Rewrite the baseline with the current hit list.",
    )
    args = p.parse_args(argv)

    repo_root = _resolve_repo_root(args.repo_root)
    baseline_path = Path(args.baseline) if args.baseline else (
        repo_root / "scripts" / "release-audit-stale-tests.baseline"
    )

    hits = scan(repo_root)

    if args.update_baseline:
        _write_baseline(baseline_path, hits)
        print(
            f"baseline updated: {baseline_path.relative_to(repo_root)} "
            f"({len(hits)} entries)"
        )
        return 0

    baseline = _read_baseline(baseline_path)
    print(_format_report(hits, repo_root, baseline))

    if args.check:
        if baseline is None:
            print(
                "ERROR: --check requested but no baseline at "
                f"{baseline_path}",
                file=sys.stderr,
            )
            return 1
        current_lines = set(_baseline_lines(hits))
        baseline_lines = set(baseline)
        if current_lines != baseline_lines:
            print(
                "ERROR: stale-test sweep diverges from baseline; "
                "review hits and rerun with --update-baseline if "
                "intentional.",
                file=sys.stderr,
            )
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
