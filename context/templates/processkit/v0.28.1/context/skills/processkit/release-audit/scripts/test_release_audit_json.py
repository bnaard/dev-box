"""test_release_audit_json.py — Tests for --json flag and run_pk_release_audit wrapper.

Covers BACK-20260510_0751-TallFern (T1.4).

Run from any directory:
    python3 context/skills/processkit/release-audit/scripts/test_release_audit_json.py

Exit 0 = all tests passed.
Exit 1 = at least one test failed.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent
_AUDIT = _SCRIPTS_DIR / "release_audit.py"
_REPO_ROOT = next(
    p for p in Path(__file__).resolve().parents
    if (p / "src" / "context" / "schemas").is_dir()
)

PASS = "\033[32mPASS\033[0m"
FAIL = "\033[31mFAIL\033[0m"

failures: list[str] = []


def check(name: str, condition: bool, detail: str = "") -> None:
    if condition:
        print(f"  {PASS}  {name}")
    else:
        print(f"  {FAIL}  {name}" + (f": {detail}" if detail else ""))
        failures.append(name)


def run_audit(*extra_args: str) -> subprocess.CompletedProcess:
    # release_audit.py has uv inline deps (pyyaml); invoke via uv run.
    return subprocess.run(
        ["uv", "run", str(_AUDIT),
         f"--repo-root={_REPO_ROOT}", *extra_args],
        capture_output=True, text=True,
        cwd=str(_REPO_ROOT),
    )


# ---------------------------------------------------------------------------
# Test 1: --json produces valid JSON on stdout
# ---------------------------------------------------------------------------
print("\n[1] release_audit.py --json produces valid JSON")

result = run_audit("--json")
check("exits 0 or 1", result.returncode in (0, 1),
      f"returncode={result.returncode}, stderr={result.stderr[:200]}")

try:
    payload = json.loads(result.stdout)
    check("stdout is valid JSON", True)
except json.JSONDecodeError as e:
    check("stdout is valid JSON", False, str(e))
    payload = {}

check("payload has 'findings' list", isinstance(payload.get("findings"), list))
check("payload has 'totals' dict", isinstance(payload.get("totals"), dict))
check("totals has 'error'", "error" in payload.get("totals", {}))
check("totals has 'warn'", "warn" in payload.get("totals", {}))
check("totals has 'info'", "info" in payload.get("totals", {}))
check("payload has 'exit_code'", "exit_code" in payload)
check("payload has 'per_tree'", isinstance(payload.get("per_tree"), list))

# ---------------------------------------------------------------------------
# Test 2: exit_code consistent with error count
# ---------------------------------------------------------------------------
print("\n[2] exit_code consistent with totals.error")

if payload:
    totals = payload.get("totals", {})
    expected_exit = 1 if totals.get("error", 0) > 0 else 0
    check("exit_code matches error count",
          payload.get("exit_code") == expected_exit,
          f"exit_code={payload.get('exit_code')}, expected={expected_exit}")

# ---------------------------------------------------------------------------
# Test 3: default (no --json) produces human-readable text
# ---------------------------------------------------------------------------
print("\n[3] default (no --json) produces human-readable report")

result_text = run_audit()
check("exits 0 or 1", result_text.returncode in (0, 1))
check("stdout contains '## totals'", "## totals" in result_text.stdout,
      result_text.stdout[:200])

# ---------------------------------------------------------------------------
# Test 4: findings items have expected keys
# ---------------------------------------------------------------------------
print("\n[4] findings items have expected structure")

if payload.get("findings"):
    first = payload["findings"][0]
    for key in ("tree", "check", "severity", "id", "message"):
        check(f"finding has '{key}'", key in first, str(first))
else:
    check("findings is empty list (clean repo)", payload.get("findings") == [])

# ---------------------------------------------------------------------------
# Test 5: --tree=both runs both context and src-context
# ---------------------------------------------------------------------------
print("\n[5] --tree=both populates per_tree with two entries")

result_both = run_audit("--json", "--tree=both")
try:
    payload_both = json.loads(result_both.stdout)
    per_tree = payload_both.get("per_tree", [])
    labels = [e.get("tree") for e in per_tree]
    check("per_tree has context entry", "context" in labels, str(labels))
    check("per_tree has src-context entry", "src-context" in labels, str(labels))
except json.JSONDecodeError as e:
    check("--tree=both JSON parseable", False, str(e))

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
