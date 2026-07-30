"""check_entity_read.py — PreToolUse hook: block Read on canonical entity paths.

WorkItem: BACK-20260510_0751-TallFern (T2.1)
Decision: DEC-20260510_0758-FierceFern (choice 1: BLOCK strict, WARN lenient)

Purpose
-------
When the Read tool is invoked with a path matching a canonical entity
file in the processkit context tree, exit non-zero to block the call and
direct the agent to use ``get_entity`` / ``list_entities`` /
``search_entities`` instead.

Canonical entity dirs (BLOCK):
    context/{workitems,decisions,artifacts,team-members,scopes,gates,
             actors,roles,bindings}/**/*.md

Gray-area paths (pass through silently):
    - context/logs/
    - context/schemas/
    - context/migrations/applied/
    - context/team-members/<slug>/persona.md
    - context/team-members/<slug>/card.json
    - context/team-members/<slug>/{knowledge,journal,skills,relations,
                                   lessons,private,working}/...
    - context/skills/ (skill source code is explicitly readable)
    - Anything outside context/ entirely

stdin
-----
Reads a JSON object (Claude Code PreToolUse shape):
    tool_name   str  — name of the tool ("Read")
    tool_input  obj  — tool arguments (file_path)
    cwd         str  — working directory at invocation time

Exit codes
----------
0   pass (not blocked)
2   blocked (stderr shown to user; do NOT use exit 1)
"""
from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Canonical entity dirs — Read on these *.md files is BLOCKED.
# team-members is treated specially below (only the entity file is blocked,
# sub-files like persona.md, card.json, knowledge/* are allowed).
# ---------------------------------------------------------------------------

_CANONICAL_ENTITY_DIRS = frozenset({
    "workitems",
    "decisions",
    "artifacts",
    "scopes",
    "gates",
    "actors",
    "roles",
    "bindings",
})

# team-members entity file basename (only this file inside a slug dir is blocked).
_TM_ENTITY_FILE = "team-member.md"

# Sub-dirs inside a team-member slug dir that are NOT canonical entity files.
_TM_SAFE_SUBDIRS = frozenset({
    "knowledge", "journal", "skills", "relations",
    "lessons", "private", "working",
})


def _project_root(cwd: str) -> Path:
    """Walk up from cwd to find the directory containing context/."""
    candidate = Path(cwd).resolve()
    while True:
        if (candidate / "context").is_dir():
            return candidate
        parent = candidate.parent
        if parent == candidate:
            return Path(cwd).resolve()
        candidate = parent


def _is_blocked(file_path: str, cwd: str) -> tuple[bool, str]:
    """Return (blocked, reason_or_derived_id).

    Returns (True, message) when the path is a canonical entity file that
    should be read through the MCP gateway instead.  Returns (False, "")
    for all other paths.
    """
    p = Path(file_path)
    if not p.is_absolute():
        p = Path(cwd) / p
    try:
        p = p.resolve()
    except Exception:
        return False, ""

    root = _project_root(cwd)
    ctx = root / "context"

    # Must be inside context/
    try:
        rel = p.relative_to(ctx)
    except ValueError:
        return False, ""

    parts = rel.parts
    if not parts:
        return False, ""

    top = parts[0]  # e.g. "workitems", "decisions", "team-members", "skills"

    # skills/ — explicitly allowed (skill source code)
    if top == "skills":
        return False, ""

    # logs/, schemas/, migrations/ — pass through
    if top in ("logs", "schemas", "migrations", "templates", "notes", "discussions"):
        return False, ""

    # team-members/ — special handling
    if top == "team-members":
        # Must have at least <slug>/<file>
        if len(parts) < 2:
            return False, ""
        # parts[1] = slug, parts[2] = file or subdir
        if len(parts) == 2:
            # context/team-members/<file>.md — not a normal layout; pass through
            return False, ""
        slug = parts[1]
        rest = parts[2:]
        filename = rest[0]
        # Only the entity file is blocked
        if filename == _TM_ENTITY_FILE and len(rest) == 1:
            derived_id = f"TEAMMEMBER-{slug}"
            return True, derived_id
        # Sub-dirs and persona.md / card.json are safe
        return False, ""

    # Canonical entity dirs
    if top in _CANONICAL_ENTITY_DIRS:
        if not p.suffix == ".md":
            return False, ""
        # Derive ID from stem
        stem = p.stem
        return True, stem

    return False, ""


def _block_message(file_path: str, derived_id: str) -> str:
    short_id = derived_id.split("/")[-1] if "/" in derived_id else derived_id
    return (
        f"BLOCKED: {file_path} is a canonical entity file. "
        f"Use get_entity(id='{short_id}') or the kind-specific get_* tool. "
        "Reading entity files directly bypasses index re-validation and "
        "misses v1-penalty annotations. "
        "Alternatives: get_entity / list_entities / search_entities / "
        "get_entity_by_path."
    )


def main() -> int:
    raw = sys.stdin.read()
    try:
        hook_input: dict = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError:
        return 0

    tool_name: str = hook_input.get("tool_name", "")
    if tool_name != "Read":
        return 0

    tool_input: dict = hook_input.get("tool_input", {})
    cwd: str = hook_input.get("cwd", os.getcwd())

    file_path: str = tool_input.get("file_path", "") or tool_input.get("path", "")
    if not file_path:
        return 0

    blocked, derived_id = _is_blocked(file_path, cwd)
    if blocked:
        print(_block_message(file_path, derived_id), file=sys.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
