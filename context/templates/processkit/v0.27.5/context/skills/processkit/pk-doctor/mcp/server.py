#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "sqlite-vec>=0.1.0",
# ]
# ///
"""processkit pk-doctor MCP server.

Exposes a single tool that runs the aggregator health-check script and
returns structured JSON output (BACK-20260510_0751-TallFern, T1.3).

Tools provided:
    run_pk_doctor(check?, fix?) -> {findings, totals, exit_code, ...}
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

server = FastMCP("processkit-pk-doctor")

_DOCTOR_SCRIPT = (
    Path(__file__).resolve().parent.parent / "scripts" / "doctor.py"
)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def run_pk_doctor(
    check: str | None = None,
    fix: str | None = None,
) -> dict:
    """Run the pk-doctor aggregator health check and return structured JSON.

    Invokes ``pk-doctor/scripts/doctor.py --json`` as a subprocess and
    parses the output. The raw human-readable report is available by
    running the script directly without the ``--json`` flag.
    This is the in-container processkit health surface for derived
    projects; host-side installer diagnostics are outside this tool's
    scope.

    Parameters
    ----------
    check:
        Comma-separated list of check categories to run (e.g.
        ``"entity_format,schema_compliance"``). Defaults to all checks.
    fix:
        Comma-separated list of categories to enable automatic fixes for.
        Passed as ``--fix=<value>`` to the script.

    Returns
    -------
    A dict with:
    - ``findings``: list of finding dicts (each has severity, category,
      id, message, suggested_fix, and actionability fields).
    - ``totals``: ``{error: int, warn: int, info: int}``.
    - ``action_totals``: counts for actionable findings, confirmation
      requirements, tracking needs, and action kinds.
    - ``exit_code``: 0 (clean) or 1 (errors found).
    - ``invocation``: the reconstructed invocation string.
    - ``duration_ms``: wall-clock time in milliseconds.
    - ``doctor_version``: version string of the doctor script.
    """
    root = paths.find_project_root()
    cmd = [sys.executable, str(_DOCTOR_SCRIPT), "--json", "--no-log"]
    cmd += ["--repo-root", str(root)]
    if check:
        cmd += [f"--category={check}"]
    if fix:
        cmd += [f"--fix={fix}"]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            cwd=str(root),
        )
    except Exception as exc:
        return {
            "error": f"failed to invoke doctor.py: {exc}",
            "findings": [],
            "totals": {"error": 1, "warn": 0, "info": 0},
            "action_totals": {"actionable": 1},
            "exit_code": 1,
        }

    stdout = result.stdout.strip()
    if not stdout:
        return {
            "error": "doctor.py produced no output",
            "stderr": result.stderr.strip(),
            "findings": [],
            "totals": {"error": 1, "warn": 0, "info": 0},
            "action_totals": {"actionable": 1},
            "exit_code": result.returncode,
        }

    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError:
        return {
            "error": "could not parse doctor.py JSON output",
            "raw_output": stdout[:500],
            "stderr": result.stderr.strip(),
            "findings": [],
            "totals": {"error": 1, "warn": 0, "info": 0},
            "action_totals": {"actionable": 1},
            "exit_code": result.returncode,
        }

    return payload


if __name__ == "__main__":
    server.run(transport="stdio")
