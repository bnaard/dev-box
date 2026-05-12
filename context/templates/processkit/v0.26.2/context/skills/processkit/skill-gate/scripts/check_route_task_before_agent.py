"""check_route_task_before_agent.py — PreToolUse hook: require route_task before Agent.

WorkItem: BACK-20260510_0751-TallFern (T2.2)
Decision: DEC-20260510_0758-FierceFern (choice 4: hook over slash command)

Purpose
-------
Block bare-model Agent / Task dispatch until the agent has called
``route_task(task_description=...)`` in the same turn.  This enforces
the compliance contract requirement:

  "Call route_task(task_description) before any sub-agent / Task /
   Agent dispatch; read recommended_team_member_slug and
   recommended_model_class from the response."

Per-turn state mechanism
------------------------
The processkit gateway MCP server writes a route-task marker file at
``context/.state/skill-gate/route-task-<session_id>-<ts>.routed``
whenever ``route_task`` is called.  This hook scans for any such marker
whose ``ts`` field is within _TURN_WINDOW_SECONDS of now; if found, the
Agent dispatch is allowed.

If the marker infrastructure is not available (old server, different
harness), the hook degrades gracefully: it WARNs on stderr but does NOT
block (exit 0) to avoid breaking non-processkit projects.

The session_id used for scoping comes from the hook input's
``session_id`` field, falling back to ``PROCESSKIT_SESSION_ID`` env
var, then ``str(os.getpid())``.

stdin
-----
Reads a JSON object (Claude Code PreToolUse shape):
    tool_name   str  — name of the tool ("Agent" or "Task")
    tool_input  obj  — tool arguments
    session_id  str  — Claude Code session UUID (optional)
    cwd         str  — working directory at invocation time

Exit codes
----------
0   pass (route_task marker found, or graceful-degradation path)
2   blocked (stderr shown to user)
"""
from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

# How far back we look for a route_task marker.  A turn completes
# within a few seconds; 120 s is generous while still being "same turn".
_TURN_WINDOW_SECONDS = 120

_MARKER_SUBDIR = Path("context") / ".state" / "skill-gate"
_MARKER_GLOB = "route-task-*.routed"

_AGENT_TOOLS = frozenset({"Agent", "Task"})


def _project_root(cwd: str) -> Path:
    candidate = Path(cwd).resolve()
    while True:
        if (candidate / "context").is_dir():
            return candidate
        parent = candidate.parent
        if parent == candidate:
            return Path(cwd).resolve()
        candidate = parent


def _session_id(hook_input: dict) -> str:
    sid = hook_input.get("session_id", "")
    if sid:
        return str(sid)
    return os.environ.get("PROCESSKIT_SESSION_ID", str(os.getpid()))


def _marker_dir(cwd: str) -> Path:
    return _project_root(cwd) / _MARKER_SUBDIR


def _any_recent_route_task(cwd: str, session_id: str) -> bool:
    """Return True if a route_task marker exists within the turn window.

    Looks for markers matching ``route-task-<session_id>-*.routed`` that
    have a ``ts`` field within ``_TURN_WINDOW_SECONDS`` of now.

    Falls back to any route-task marker (any session) if no session-
    scoped marker exists — this covers harnesses that don't propagate
    the session UUID.
    """
    marker_dir = _marker_dir(cwd)
    if not marker_dir.is_dir():
        return False

    now = datetime.now(timezone.utc)
    cutoff = now - timedelta(seconds=_TURN_WINDOW_SECONDS)

    # Try session-scoped markers first
    session_prefix = f"route-task-{session_id}-"
    session_markers = list(marker_dir.glob(f"{session_prefix}*.routed"))
    all_markers = list(marker_dir.glob(_MARKER_GLOB))

    for candidate_set in (session_markers, all_markers):
        for marker in candidate_set:
            try:
                data = json.loads(marker.read_text(encoding="utf-8"))
            except (json.JSONDecodeError, OSError):
                continue
            ts_str = data.get("ts", "")
            try:
                ts = datetime.fromisoformat(ts_str)
                if ts.tzinfo is None:
                    ts = ts.replace(tzinfo=timezone.utc)
                if ts >= cutoff:
                    return True
            except (TypeError, ValueError):
                # Malformed ts — skip this marker
                continue

    return False


def _block_message() -> str:
    return (
        "BLOCKED: bare-model Agent dispatch without a prior route_task call.\n"
        "Call route_task(task_description=...) first to determine\n"
        "  recommended_team_member_slug + recommended_model_class,\n"
        "then dispatch with the recommended model.\n"
        "Example:\n"
        "  route = route_task(task_description='<what you need the sub-agent to do>')\n"
        "  # read route['recommended_team_member_slug'] and "
        "route['recommended_model_class']\n"
        "  Agent(prompt='...', model='<recommended model>')"
    )


def main() -> int:
    raw = sys.stdin.read()
    try:
        hook_input: dict = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError:
        return 0

    tool_name: str = hook_input.get("tool_name", "")
    if tool_name not in _AGENT_TOOLS:
        return 0

    cwd: str = hook_input.get("cwd", os.getcwd())
    sid = _session_id(hook_input)

    # Check for route_task marker
    marker_dir = _marker_dir(cwd)
    if not marker_dir.is_dir():
        # Marker infrastructure not set up — graceful degradation (warn only).
        print(
            "WARNING: route_task marker dir not found; skipping Agent dispatch check. "
            "Ensure route_task is called before Agent dispatch (compliance contract).",
            file=sys.stderr,
        )
        return 0

    if _any_recent_route_task(cwd, sid):
        return 0

    print(_block_message(), file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
