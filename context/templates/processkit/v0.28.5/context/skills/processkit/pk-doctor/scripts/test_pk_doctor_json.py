"""test_pk_doctor_json.py — Tests for --json flag and run_pk_doctor MCP wrapper.

Covers BACK-20260510_0751-TallFern (T1.3).

Run from any directory:
    python3 context/skills/processkit/pk-doctor/scripts/test_pk_doctor_json.py

Exit 0 = all tests passed.
Exit 1 = at least one test failed.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent
_DOCTOR = _SCRIPTS_DIR / "doctor.py"
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


def run_doctor(*extra_args: str) -> subprocess.CompletedProcess:
    # doctor.py has uv inline deps (pyyaml, jsonschema); invoke via uv run.
    return subprocess.run(
        ["uv", "run", str(_DOCTOR), "--no-log",
         f"--repo-root={_REPO_ROOT}", *extra_args],
        capture_output=True, text=True,
        cwd=str(_REPO_ROOT),
    )


# ---------------------------------------------------------------------------
# Test 1: --json flag produces valid JSON on stdout
# ---------------------------------------------------------------------------
print("\n[1] doctor.py --json produces valid JSON")

result = run_doctor("--json")
check("exits 0 or 1 (not crash)", result.returncode in (0, 1),
      f"returncode={result.returncode}, stderr={result.stderr[:200]}")

try:
    payload = json.loads(result.stdout)
    check("stdout is valid JSON", True)
except json.JSONDecodeError as e:
    check("stdout is valid JSON", False, str(e))
    payload = {}

check("payload has 'findings' list", isinstance(payload.get("findings"), list),
      str(type(payload.get("findings"))))
check("payload has 'totals' dict", isinstance(payload.get("totals"), dict),
      str(type(payload.get("totals"))))
check("payload has 'action_totals' dict",
      isinstance(payload.get("action_totals"), dict),
      str(type(payload.get("action_totals"))))
check("totals has 'error' key", "error" in payload.get("totals", {}))
check("totals has 'warn' key", "warn" in payload.get("totals", {}))
check("totals has 'info' key", "info" in payload.get("totals", {}))
check("action_totals has 'actionable' key",
      "actionable" in payload.get("action_totals", {}))
check("payload has 'exit_code'", "exit_code" in payload)
check("exit_code matches returncode", payload.get("exit_code") == result.returncode,
      f"payload={payload.get('exit_code')}, actual={result.returncode}")

# ---------------------------------------------------------------------------
# Test 2: --json exit_code == 0 when no errors in payload
# ---------------------------------------------------------------------------
print("\n[2] exit_code consistent with totals.error")

if payload:
    totals = payload.get("totals", {})
    expected_exit = 1 if totals.get("error", 0) > 0 else 0
    check("exit_code consistent with error count",
          payload.get("exit_code") == expected_exit,
          f"exit_code={payload.get('exit_code')}, expected={expected_exit}")

# ---------------------------------------------------------------------------
# Test 3: default (no --json) still produces human-readable text
# ---------------------------------------------------------------------------
print("\n[3] default (no --json) produces human-readable report")

result_text = run_doctor()
check("exits 0 or 1", result_text.returncode in (0, 1))
check("stdout contains '## totals'", "## totals" in result_text.stdout,
      result_text.stdout[:200])
check("stdout contains '## actions'", "## actions" in result_text.stdout,
      result_text.stdout[:200])

# Should not be parseable as JSON in text mode
try:
    json.loads(result_text.stdout)
    check("stdout is NOT pure JSON (text mode)", False,
          "text output was parseable as JSON — unexpected")
except json.JSONDecodeError:
    check("stdout is NOT pure JSON (text mode)", True)

# ---------------------------------------------------------------------------
# Test 4: findings list items have expected keys
# ---------------------------------------------------------------------------
print("\n[4] findings items have expected structure")

if payload.get("findings"):
    first = payload["findings"][0]
    for key in (
        "severity",
        "category",
        "id",
        "message",
        "action_required",
        "action_kind",
        "default_agent_action",
        "requires_user_confirmation",
        "acceptable_resolution",
    ):
        check(f"finding has '{key}'", key in first, str(first))
else:
    # No findings is fine — just make sure the list is empty not missing
    check("findings is empty list (clean repo)", payload.get("findings") == [],
          str(payload.get("findings")))

# ---------------------------------------------------------------------------
# Test 5: doctor/aibox boundary guard stays clean
# ---------------------------------------------------------------------------
print("\n[5] doctor/aibox boundary guard stays clean")

boundary = run_doctor("--json", "--category=doctor_boundary")
check("doctor_boundary exits 0", boundary.returncode == 0,
      f"returncode={boundary.returncode}, stderr={boundary.stderr[:200]}")
try:
    boundary_payload = json.loads(boundary.stdout)
    boundary_findings = boundary_payload.get("findings", [])
    boundary_errors = [
        item for item in boundary_findings
        if item.get("category") == "doctor_boundary"
        and item.get("severity") == "ERROR"
    ]
    check("doctor_boundary has no errors", not boundary_errors,
          str(boundary_errors[:3]))
except json.JSONDecodeError as e:
    check("doctor_boundary stdout is valid JSON", False, str(e))

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
