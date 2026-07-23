"""test_entity_read_and_agent_hooks.py — Tests for the two new PreToolUse hooks.

Covers BACK-20260510_0751-TallFern (T2.1 check_entity_read and T2.2 check_route_task_before_agent).

Run from any directory:
    python3 context/skills/processkit/skill-gate/scripts/test_entity_read_and_agent_hooks.py

Exit 0 = all tests passed.
Exit 1 = at least one test failed.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).parent
_ENTITY_READ_SCRIPT = _SCRIPTS_DIR / "check_entity_read.py"
_AGENT_SCRIPT = _SCRIPTS_DIR / "check_route_task_before_agent.py"

PASS = "\033[32mPASS\033[0m"
FAIL = "\033[31mFAIL\033[0m"

failures: list[str] = []


def check(name: str, condition: bool, detail: str = "") -> None:
    if condition:
        print(f"  {PASS}  {name}")
    else:
        print(f"  {FAIL}  {name}" + (f": {detail}" if detail else ""))
        failures.append(name)


def run(script: Path, stdin_data: dict, extra_env: dict | None = None) -> subprocess.CompletedProcess:
    env = os.environ.copy()
    if extra_env:
        env.update(extra_env)
    return subprocess.run(
        [sys.executable, str(script)],
        input=json.dumps(stdin_data),
        capture_output=True,
        text=True,
        env=env,
    )


# ============================================================================
# T2.1 — check_entity_read.py
# ============================================================================

print("\n=== T2.1 check_entity_read.py ===\n")

# ---------------------------------------------------------------------------
# [1] BLOCK: canonical entity file paths
# ---------------------------------------------------------------------------
print("[1] Canonical entity paths are BLOCKED")

CANONICAL_ENTITY_PATHS = [
    "context/workitems/2026/05/BACK-20260510_0751-TallFern-foo.md",
    "context/decisions/DEC-20260510_0758-FierceFern-bar.md",
    "context/artifacts/ART-20260101_0000-TestArt.md",
    "context/scopes/SCOPE-test.md",
    "context/gates/GATE-test.md",
    "context/actors/ACTOR-legacy.md",
    "context/roles/ROLE-test.md",
    "context/bindings/BINDING-test.md",
]

for path in CANONICAL_ENTITY_PATHS:
    r = run(
        _ENTITY_READ_SCRIPT,
        {"tool_name": "Read", "tool_input": {"file_path": path}, "cwd": "/workspace"},
    )
    check(
        f"BLOCK {path.split('/')[1]}/...md",
        r.returncode == 2,
        f"got exit {r.returncode}, stderr={r.stderr[:80]}",
    )
    check(
        f"BLOCK {path.split('/')[1]}: stderr contains 'BLOCKED'",
        "BLOCKED" in r.stderr,
        r.stderr[:80],
    )

# Team-member entity file is also BLOCKED
r = run(
    _ENTITY_READ_SCRIPT,
    {"tool_name": "Read",
     "tool_input": {"file_path": "context/team-members/cora/team-member.md"},
     "cwd": "/workspace"},
)
check("BLOCK team-members/<slug>/team-member.md", r.returncode == 2,
      f"got exit {r.returncode}")
check("BLOCK team-member.md: stderr 'BLOCKED'", "BLOCKED" in r.stderr, r.stderr[:80])

# ---------------------------------------------------------------------------
# [2] PASS: non-entity paths (gray area)
# ---------------------------------------------------------------------------
print("\n[2] Non-entity / gray-area paths pass through")

SAFE_PATHS = [
    # skills source code
    "context/skills/processkit/index-management/mcp/server.py",
    # logs
    "context/logs/2026/05/LOG-test.md",
    # migrations/applied
    "context/migrations/applied/MIG-test.md",
    # schemas
    "context/schemas/team-member.yaml",
    # team-member sub-files
    "context/team-members/cora/persona.md",
    "context/team-members/cora/card.json",
    "context/team-members/cora/knowledge/some-doc.md",
    "context/team-members/finn/journal/2026-05-01.md",
    # outside context entirely
    "src/lib/processkit/__init__.py",
    "docs/harness-claude-code.md",
    "README.md",
]

for path in SAFE_PATHS:
    r = run(
        _ENTITY_READ_SCRIPT,
        {"tool_name": "Read", "tool_input": {"file_path": path}, "cwd": "/workspace"},
    )
    check(
        f"PASS {path}",
        r.returncode == 0,
        f"got exit {r.returncode}, stderr={r.stderr[:80]}",
    )

# ---------------------------------------------------------------------------
# [3] Non-Read tools are ignored
# ---------------------------------------------------------------------------
print("\n[3] Non-Read tools always pass")

for tool_name in ("Write", "Edit", "Agent", "Bash"):
    r = run(
        _ENTITY_READ_SCRIPT,
        {"tool_name": tool_name,
         "tool_input": {"file_path": "context/workitems/2026/05/BACK-test.md"},
         "cwd": "/workspace"},
    )
    check(f"PASS tool={tool_name}", r.returncode == 0,
          f"got exit {r.returncode}")

# ---------------------------------------------------------------------------
# [4] Empty stdin does not crash
# ---------------------------------------------------------------------------
print("\n[4] Robustness: empty/malformed stdin")

for bad_stdin in ("{}", '{"tool_name": "Read", "tool_input": {}, "cwd": "/workspace"}'):
    proc = subprocess.run(
        [sys.executable, str(_ENTITY_READ_SCRIPT)],
        input=bad_stdin,
        capture_output=True,
        text=True,
    )
    check("no crash on minimal stdin", proc.returncode in (0, 2),
          f"got {proc.returncode}")


# ============================================================================
# T2.2 — check_route_task_before_agent.py
# ============================================================================

print("\n=== T2.2 check_route_task_before_agent.py ===\n")

# ---------------------------------------------------------------------------
# [5] BLOCK when marker dir exists but no recent marker
# ---------------------------------------------------------------------------
print("[5] BLOCK when no recent route_task marker")

with tempfile.TemporaryDirectory() as tmpdir:
    root = Path(tmpdir)
    # Create the marker dir (so graceful-degradation path is not taken)
    marker_dir = root / "context" / ".state" / "skill-gate"
    marker_dir.mkdir(parents=True)
    # Also create context/ so _project_root finds it
    (root / "context").mkdir(exist_ok=True)

    r = run(
        _AGENT_SCRIPT,
        {"tool_name": "Agent", "tool_input": {}, "cwd": str(root)},
    )
    check("BLOCK Agent with no marker", r.returncode == 2,
          f"got exit {r.returncode}")
    check("stderr contains 'BLOCKED'", "BLOCKED" in r.stderr,
          r.stderr[:100])

# ---------------------------------------------------------------------------
# [6] PASS when recent route_task marker exists
# ---------------------------------------------------------------------------
print("\n[6] PASS when recent route_task marker is present")

import datetime

with tempfile.TemporaryDirectory() as tmpdir:
    root = Path(tmpdir)
    marker_dir = root / "context" / ".state" / "skill-gate"
    marker_dir.mkdir(parents=True)
    (root / "context").mkdir(exist_ok=True)

    # Write a fresh marker
    now = datetime.datetime.now(datetime.timezone.utc)
    marker = marker_dir / "route-task-testsession-999999999.routed"
    marker.write_text(json.dumps({"ts": now.isoformat(), "session_id": "testsession"}))

    r = run(
        _AGENT_SCRIPT,
        {"tool_name": "Agent", "tool_input": {}, "cwd": str(root)},
    )
    check("PASS Agent with fresh marker", r.returncode == 0,
          f"got exit {r.returncode}, stderr={r.stderr[:80]}")

# ---------------------------------------------------------------------------
# [7] BLOCK when marker is stale (> 120s)
# ---------------------------------------------------------------------------
print("\n[7] BLOCK when route_task marker is stale (> 120s)")

with tempfile.TemporaryDirectory() as tmpdir:
    root = Path(tmpdir)
    marker_dir = root / "context" / ".state" / "skill-gate"
    marker_dir.mkdir(parents=True)
    (root / "context").mkdir(exist_ok=True)

    # Write a stale marker (5 minutes ago)
    stale_ts = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(minutes=5)
    marker = marker_dir / "route-task-testsession-000000001.routed"
    marker.write_text(json.dumps({"ts": stale_ts.isoformat(), "session_id": "testsession"}))

    r = run(
        _AGENT_SCRIPT,
        {"tool_name": "Agent", "tool_input": {}, "cwd": str(root)},
    )
    check("BLOCK Agent with stale marker", r.returncode == 2,
          f"got exit {r.returncode}")

# ---------------------------------------------------------------------------
# [8] Non-Agent tools always pass
# ---------------------------------------------------------------------------
print("\n[8] Non-Agent/Task tools always pass")

with tempfile.TemporaryDirectory() as tmpdir:
    root = Path(tmpdir)
    marker_dir = root / "context" / ".state" / "skill-gate"
    marker_dir.mkdir(parents=True)
    (root / "context").mkdir(exist_ok=True)

    for tool_name in ("Read", "Write", "Bash", "create_workitem"):
        r = run(
            _AGENT_SCRIPT,
            {"tool_name": tool_name, "tool_input": {}, "cwd": str(root)},
        )
        check(f"PASS tool={tool_name}", r.returncode == 0,
              f"got exit {r.returncode}")

# ---------------------------------------------------------------------------
# [9] Graceful degradation when marker dir does not exist
# ---------------------------------------------------------------------------
print("\n[9] Graceful degradation — no marker dir → warn but do NOT block")

with tempfile.TemporaryDirectory() as tmpdir:
    root = Path(tmpdir)
    # context/ present but no .state/skill-gate
    (root / "context").mkdir()

    r = run(
        _AGENT_SCRIPT,
        {"tool_name": "Agent", "tool_input": {}, "cwd": str(root)},
    )
    check("PASS (graceful-degrade) when no marker dir", r.returncode == 0,
          f"got exit {r.returncode}, stderr={r.stderr[:80]}")

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print()
if failures:
    print(f"FAILED: {len(failures)} test(s): {failures}")
    sys.exit(1)
else:
    print("All tests passed.")
    sys.exit(0)
