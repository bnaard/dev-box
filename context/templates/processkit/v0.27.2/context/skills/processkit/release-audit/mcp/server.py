#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
# ]
# ///
"""processkit release-audit MCP server.

Exposes a single tool that runs the pre-release validation script and
returns structured JSON output (BACK-20260510_0751-TallFern, T1.4).

Tools provided:
    run_pk_release_audit(tree?) -> {findings, totals, exit_code, ...}
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


def _find_lib() -> Path:
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for c in (here / "src" / "lib", here / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import paths  # noqa: E402

server = FastMCP("processkit-release-audit")

_AUDIT_SCRIPT = (
    Path(__file__).resolve().parent.parent / "scripts" / "release_audit.py"
)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def run_pk_release_audit(
    tree: str | None = None,
) -> dict:
    """Run the pre-release validation sweep and return structured JSON.

    Invokes ``release-audit/scripts/release_audit.py --json`` as a
    subprocess and parses the output. The raw human-readable report is
    available by running the script directly without the ``--json`` flag.

    Parameters
    ----------
    tree:
        Which tree to audit: ``"context"`` (live dogfood, default),
        ``"src-context"`` (shipped release deliverable), or ``"both"``.

    Returns
    -------
    A dict with:
    - ``findings``: list of finding dicts (each has tree, check,
      severity, id, message, entity_ref).
    - ``totals``: ``{error: int, warn: int, info: int}``.
    - ``exit_code``: 0 (clean) or 1 (errors found).
    - ``per_tree``: per-tree tally breakdown.
    - ``audit_version``: version string of the audit script.
    """
    root = paths.find_project_root()
    cmd = [sys.executable, str(_AUDIT_SCRIPT), "--json"]
    cmd += ["--repo-root", str(root)]
    if tree:
        cmd += [f"--tree={tree}"]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            cwd=str(root),
        )
    except Exception as exc:
        return {
            "error": f"failed to invoke release_audit.py: {exc}",
            "findings": [],
            "totals": {"error": 1, "warn": 0, "info": 0},
            "exit_code": 1,
        }

    stdout = result.stdout.strip()
    if not stdout:
        return {
            "error": "release_audit.py produced no output",
            "stderr": result.stderr.strip(),
            "findings": [],
            "totals": {"error": 1, "warn": 0, "info": 0},
            "exit_code": result.returncode,
        }

    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError:
        return {
            "error": "could not parse release_audit.py JSON output",
            "raw_output": stdout[:500],
            "stderr": result.stderr.strip(),
            "findings": [],
            "totals": {"error": 1, "warn": 0, "info": 0},
            "exit_code": result.returncode,
        }

    return payload


if __name__ == "__main__":
    server.run(transport="stdio")
