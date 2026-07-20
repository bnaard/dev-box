#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""test_doctor.py — smoke tests for pk-doctor (Phase 1).

Seeds a tiny context/ + src/context/ tree in a tempdir and asserts:
- CheckResult shape + severity tallies
- aggregator exit code (0 if no ERROR, 1 otherwise)
- stub LogEntry payload shape (emit called exactly once per run)

Run from any directory:

    python3 context/skills/processkit/pk-doctor/scripts/test_doctor.py

or via the uv shebang if PyYAML/jsonschema aren't globally installed.

Exit 0 = all tests passed.
Exit 1 = at least one test failed.
"""

from __future__ import annotations

import json
import importlib.util
import os
import sqlite3
import subprocess
import sys
import tempfile
import textwrap
from datetime import datetime, timezone, timedelta
from pathlib import Path
import yaml


# ---------------------------------------------------------------------------
# Locate files relative to this script, not cwd.
# ---------------------------------------------------------------------------
_SCRIPTS_DIR = Path(__file__).resolve().parent
_DOCTOR = _SCRIPTS_DIR / "doctor.py"
_REPO_ROOT = next(
    p for p in Path(__file__).resolve().parents
    if (p / "src" / "context" / "schemas").is_dir()
)
_SCHEMAS_SRC = _REPO_ROOT / "src" / "context" / "schemas"

PASS = "\033[32mPASS\033[0m"
FAIL = "\033[31mFAIL\033[0m"

failures: list[str] = []


def check(name: str, condition: bool, detail: str = "") -> None:
    if condition:
        print(f"  {PASS}  {name}")
    else:
        print(f"  {FAIL}  {name}" + (f": {detail}" if detail else ""))
        failures.append(name)


# ---------------------------------------------------------------------------
# Fixture tree builder
# ---------------------------------------------------------------------------

def _seed_tree(root: Path) -> None:
    """Populate a minimal repo layout under `root`."""
    # Copy live schemas so schema_filename has something to validate against.
    (root / "src" / "context" / "schemas").mkdir(parents=True)
    for name in ("workitem.yaml", "logentry.yaml", "migration.yaml"):
        src = _SCHEMAS_SRC / name
        if src.is_file():
            (root / "src" / "context" / "schemas" / name).write_text(
                src.read_text(encoding="utf-8"), encoding="utf-8"
            )

    # Stub the drift script — simplest: tree is trivially in-sync.
    scripts = root / "scripts"
    scripts.mkdir()
    drift = scripts / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)

    # --- one valid workitem -----------------------------------------------
    ctx = root / "context"
    (ctx / "workitems").mkdir(parents=True)
    (ctx / "workitems" / "BACK-20260420_1200-GoodApple-valid-item.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: WorkItem
            metadata:
              id: BACK-20260420_1200-GoodApple-valid-item
              created: 2026-04-20T12:00:00Z
            spec:
              title: Valid test item
              state: proposed
              type: task
            ---
            body
            """),
        encoding="utf-8",
    )

    # --- one workitem with filename / id mismatch (WARN) ------------------
    (ctx / "workitems" / "BACK-bad-filename.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: WorkItem
            metadata:
              id: BACK-20260420_1300-BadApple-item
              created: 2026-04-20T13:00:00Z
            spec:
              title: Mismatched filename
              state: proposed
            ---
            """),
        encoding="utf-8",
    )

    # --- one mis-sharded log (WARN from sharding) -------------------------
    (ctx / "logs").mkdir()
    (ctx / "logs" / "LOG-toplevel-misplaced.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: LogEntry
            metadata:
              id: LOG-toplevel-misplaced
              created: 2026-04-20T14:00:00Z
            spec:
              event_type: test.event
              actor: ACTOR-test
              timestamp: 2026-04-20T14:00:00Z
            ---
            """),
        encoding="utf-8",
    )

    # --- one stale pending migration (WARN from migrations) ---------------
    (ctx / "migrations" / "pending").mkdir(parents=True)
    stale_created = (datetime.now(timezone.utc) - timedelta(days=30)).isoformat().replace("+00:00", "Z")
    (ctx / "migrations" / "pending" / "MIG-20260101T000000.md").write_text(
        textwrap.dedent(f"""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Migration
            metadata:
              id: MIG-20260101T000000
              created: {stale_created}
            spec:
              source: processkit
              from_version: v0.0.1
              to_version: v0.0.2
              state: pending
              summary: stale test migration
            ---
            """),
        encoding="utf-8",
    )
    # Ensure the other two required state-bucket dirs exist.
    (ctx / "migrations" / "in-progress").mkdir()
    (ctx / "migrations" / "applied").mkdir()


# ---------------------------------------------------------------------------
# Run helper
# ---------------------------------------------------------------------------

def _run_doctor(repo_root: Path, *extra: str, stub_path: Path) -> subprocess.CompletedProcess:
    env = os.environ.copy()
    env["PK_DOCTOR_LOGENTRY_STUB"] = str(stub_path)
    # Use the uv shebang for dep isolation.
    cmd = [str(_DOCTOR), "--repo-root", str(repo_root), *extra]
    return subprocess.run(cmd, capture_output=True, text=True, env=env)


# ---------------------------------------------------------------------------
# Test 1: baseline run on seeded fixture
# ---------------------------------------------------------------------------
print("\n[1] doctor.py — baseline detect-only run on seeded fixture")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, stub_path=stub)

    check("ran without crash (exit 0 or 1)", result.returncode in (0, 1),
          f"got {result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    check("stdout contains report header", "pk-doctor v" in result.stdout)
    check("schema_filename section present", "## schema_filename" in result.stdout)
    check("sharding section present", "## sharding" in result.stdout)
    check("migrations section present", "## migrations" in result.stdout)
    check("drift section present", "## drift" in result.stdout)
    check("MCP health section present", "## mcp_config_drift" in result.stdout)
    check("MCP gateway section present", "## mcp_gateway" in result.stdout)
    check("totals line present", "## totals" in result.stdout)
    check("stub log payload written", stub.is_file())
    if stub.is_file():
        payload = json.loads(stub.read_text())
        check("payload event_type correct", payload.get("event_type") == "doctor.report")
        check("payload has categories", isinstance(payload.get("details", {}).get("categories"), dict))
        check("payload has duration_ms", "duration_ms" in payload.get("details", {}))
        check("payload has doctor_version", payload.get("details", {}).get("doctor_version") == "1.0.0")
        # Should see our seeded findings: bad filename WARN, mis-sharded log WARN,
        # stale migration WARN.
        findings_blob = json.dumps(payload["details"]["top_findings"])
        check("top_findings contains filename-id-mismatch",
              "schema.filename-id-mismatch" in findings_blob,
              findings_blob[:400])
        check("top_findings contains sharding.log-wrong-bucket",
              "sharding.log-wrong-bucket" in findings_blob,
              findings_blob[:400])
        check("top_findings contains migration.stale-pending",
              "migration.stale-pending" in findings_blob,
              findings_blob[:400])

# ---------------------------------------------------------------------------
# Test 2: ERROR causes exit 1; INFO-only run exits 0
# ---------------------------------------------------------------------------
print("\n[2] doctor.py — exit code reflects ERROR tally")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    # Inject a broken-schema workitem that will produce an ERROR.
    (root / "context" / "workitems" / "BACK-20260420_1400-CleanBad-invalid.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: WorkItem
            metadata:
              id: BACK-20260420_1400-CleanBad-invalid
              created: 2026-04-20T14:00:00Z
            spec:
              state: proposed
            ---
            """),  # missing required `title`
        encoding="utf-8",
    )
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, stub_path=stub)
    check("exit 1 when ERROR present", result.returncode == 1,
          f"got {result.returncode}\nstdout:\n{result.stdout[-600:]}")
    payload = json.loads(stub.read_text())
    check("ERROR tally non-zero", any(
        c.get("ERROR", 0) > 0 for c in payload["details"]["categories"].values()
    ))

# ---------------------------------------------------------------------------
# Test 3: --category= restricts run
# ---------------------------------------------------------------------------
print("\n[3] doctor.py — --category=drift runs only the drift check")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--category=drift", stub_path=stub)
    check("exit 0 (clean drift)", result.returncode == 0,
          f"got {result.returncode}\nstderr:\n{result.stderr}")
    check("schema_filename section absent", "## schema_filename" not in result.stdout)
    check("drift section present", "## drift" in result.stdout)
    payload = json.loads(stub.read_text())
    check("only drift category in payload",
          list(payload["details"]["categories"].keys()) == ["drift"])

# ---------------------------------------------------------------------------
# Test 4: --fix + --fix-all mutex
# ---------------------------------------------------------------------------
print("\n[4] doctor.py — --fix and --fix-all are mutually exclusive")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--fix=migrations", "--fix-all", stub_path=stub)
    # argparse mutex groups exit 2 with error on stderr.
    check("mutex enforced (exit 2)", result.returncode == 2,
          f"got {result.returncode}\nstderr:{result.stderr[:200]}")

# ---------------------------------------------------------------------------
# Test 5: single logentry emitted per run (stub file overwritten, not appended)
# ---------------------------------------------------------------------------
print("\n[5] doctor.py — exactly one logentry emission per run")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, stub_path=stub)
    check("stub exists", stub.is_file())
    content = stub.read_text()
    check("stub parses as single JSON object (not JSONL)",
          content.strip().startswith("{") and content.strip().endswith("}"))
    # JSON object count — a second run should overwrite, not duplicate.
    result2 = _run_doctor(root, stub_path=stub)
    payload2 = json.loads(stub.read_text())
    check("second run overwrote (still valid JSON object)",
          isinstance(payload2, dict) and payload2.get("event_type") == "doctor.report")

# ---------------------------------------------------------------------------
# Test 6: _check_actor_id_pattern unit tests
# ---------------------------------------------------------------------------
print("\n[6] _check_actor_id_pattern — actor two-class ID pattern unit tests")

# Import the function under test directly.
sys.path.insert(0, str(_SCRIPTS_DIR))
from checks.schema_filename import _check_actor_id_pattern  # noqa: E402

_ALLOWED = [
    "assistant", "developer", "jr-architect", "jr-developer",
    "jr-researcher", "product-manager", "sr-architect", "sr-researcher",
]

# valid identity-class ID → no error
check(
    "identity class OK",
    _check_actor_id_pattern(
        "ACTOR-20260421_0144-AmberDawn-legacy-historical-backfill", _ALLOWED
    ) is None,
)

# valid role-class ID in allowlist → no error
check(
    "role class in allowlist OK",
    _check_actor_id_pattern("ACTOR-product-manager", _ALLOWED) is None,
)

# role-class ID NOT in allowlist → ERROR
result_not_in_list = _check_actor_id_pattern("ACTOR-hacker", _ALLOWED)
check(
    "role class not in allowlist → error",
    result_not_in_list is not None and "not in spec.role_actor_ids" in result_not_in_list,
    repr(result_not_in_list),
)

# malformed (underscores and caps) → ERROR
result_malformed = _check_actor_id_pattern("ACTOR-Sr_Architect", _ALLOWED)
check(
    "malformed ID (caps+underscores) → error",
    result_malformed is not None,
    repr(result_malformed),
)

# empty-slug → ERROR
result_empty = _check_actor_id_pattern("ACTOR-", _ALLOWED)
check(
    "empty slug (ACTOR-) → error",
    result_empty is not None,
    repr(result_empty),
)

# identity class with broken datetime → ERROR
result_bad_dt = _check_actor_id_pattern("ACTOR-12345-FooBar-x", _ALLOWED)
check(
    "broken datetime in identity class → error",
    result_bad_dt is not None,
    repr(result_bad_dt),
)

# ---------------------------------------------------------------------------
# Test 6b: preauth_applied — Codex allowed_tools gap is detected
# ---------------------------------------------------------------------------
print("\n[6b] preauth_applied — Codex allowed_tools gap")

from checks.preauth_applied import run as _preauth_run  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    spec_dir = (
        root / "context" / "skills" / "processkit" / "skill-gate" / "assets"
    )
    spec_dir.mkdir(parents=True)
    (spec_dir / "preauth.json").write_text(
        json.dumps({
            "version": 1,
            "permissions": {"allow": ["mcp__processkit-a__*"]},
            "enabledMcpjsonServers": ["processkit-a"],
            "codex": {
                "mcp": {
                    "allowed_tools": [
                        "mcp__processkit-a__*",
                        "mcp__processkit-b__*",
                    ],
                },
            },
        }),
        encoding="utf-8",
    )
    (root / ".codex").mkdir()
    (root / ".codex" / "config.toml").write_text(
        textwrap.dedent("""\
            [mcp]
            allowed_tools = ["mcp__processkit-a__*"]
            """),
        encoding="utf-8",
    )
    results = _preauth_run({"repo_root": root})
    codex_warns = [
        r for r in results if r.id == "preauth_applied.codex-tools-missing"
    ]
    check("Codex missing allowed_tools emits WARN", len(codex_warns) == 1)
    check(
        "Codex WARN names missing processkit tool",
        codex_warns and "mcp__processkit-b__*" in codex_warns[0].message,
        codex_warns[0].message if codex_warns else "",
    )

# ---------------------------------------------------------------------------
# Test 6b2: preauth_applied — actual mcp-config files are source of truth
# ---------------------------------------------------------------------------
print("\n[6b2] preauth_applied — stale manifest cannot hide mcp-config drift")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    spec_dir = (
        root / "context" / "skills" / "processkit" / "skill-gate" / "assets"
    )
    spec_dir.mkdir(parents=True)
    (spec_dir / "preauth.json").write_text(
        json.dumps({
            "version": 1,
            "permissions": {"allow": ["mcp__processkit-a__*"]},
            "enabledMcpjsonServers": ["processkit-a"],
            "codex": {
                "mcp": {"allowed_tools": ["mcp__processkit-a__*"]},
            },
        }),
        encoding="utf-8",
    )
    cfg_a = (
        root / "context" / "skills" / "processkit" /
        "a" / "mcp" / "mcp-config.json"
    )
    cfg_b = (
        root / "src" / "context" / "skills" / "processkit" /
        "runtime-prune" / "mcp" / "mcp-config.json"
    )
    cfg_a.parent.mkdir(parents=True)
    cfg_b.parent.mkdir(parents=True)
    cfg_a.write_text(
        '{"mcpServers":{"processkit-a":{}}}\n',
        encoding="utf-8",
    )
    cfg_b.write_text(
        '{"mcpServers":{"processkit-runtime-prune":{}}}\n',
        encoding="utf-8",
    )
    (root / "context" / ".processkit-mcp-manifest.json").write_text(
        json.dumps({
            "version": 1,
            "generated_at": "2026-05-14T00:00:00Z",
            "processkit_version": "v0.test",
            "per_skill": [{
                "path": "context/skills/processkit/a/mcp/mcp-config.json",
                "sha256": "stale-test-value",
            }],
            "per_gateway": [],
            "per_server_header": [],
            "aggregate_sha256": "stale-test-value",
        }),
        encoding="utf-8",
    )

    results = _preauth_run({"repo_root": root})
    drift_warns = [
        r for r in results
        if r.id == "preauth_applied.spec-drift"
        and "mcp-config-derived" in r.message
    ]
    check("mcp-config drift emits WARN", len(drift_warns) == 1)
    check(
        "mcp-config WARN names source-only runtime-prune",
        drift_warns
        and "processkit-runtime-prune" in drift_warns[0].message,
        drift_warns[0].message if drift_warns else "",
    )

# ---------------------------------------------------------------------------
# Test 6c: MCP gateway mode is an intentional derived-project alternative
# ---------------------------------------------------------------------------
print("\n[6c] mcp gateway — manifest + harness gateway mode")

from checks.mcp_config_drift import (  # noqa: E402
    _aggregate as _mcp_manifest_aggregate,
    _sha256_of_file as _mcp_config_sha256,
    run as _mcp_config_drift_run,
)
from checks.mcp_gateway import run as _mcp_gateway_run  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    granular_cfg = (
        root / "context" / "skills" / "processkit" /
        "artifact-management" / "mcp" / "mcp-config.json"
    )
    gateway_cfg = (
        root / "context" / "skills" / "processkit" /
        "processkit-gateway" / "mcp" / "mcp-config.json"
    )
    granular_cfg.parent.mkdir(parents=True)
    gateway_cfg.parent.mkdir(parents=True)
    granular_cfg.write_text(
        json.dumps({
            "mcpServers": {
                "processkit-artifact-management": {
                    "command": "uv",
                    "args": [
                        "run",
                        "context/skills/processkit/artifact-management/"
                        "mcp/server.py",
                    ],
                },
            },
        }),
        encoding="utf-8",
    )
    gateway_cfg.write_text(
        json.dumps({
            "mcpServers": {
                "processkit-gateway": {
                    "command": "uv",
                    "args": [
                        "run",
                        "context/skills/processkit/processkit-gateway/"
                        "mcp/server.py",
                    ],
                    "env": {"PROCESSKIT_MCP_MODE": "gateway"},
                },
            },
        }),
        encoding="utf-8",
    )
    per_skill = [{
        "path": (
            "context/skills/processkit/artifact-management/"
            "mcp/mcp-config.json"
        ),
        "sha256": _mcp_config_sha256(granular_cfg),
    }]
    per_gateway = [{
        "path": (
            "context/skills/processkit/processkit-gateway/"
            "mcp/mcp-config.json"
        ),
        "sha256": _mcp_config_sha256(gateway_cfg),
    }]
    (root / "context").mkdir(exist_ok=True)
    (root / "context" / ".processkit-mcp-manifest.json").write_text(
        json.dumps({
            "version": 1,
            "generated_at": "2026-05-02T00:00:00Z",
            "processkit_version": "v0.test",
            "per_skill": per_skill,
            "per_gateway": per_gateway,
            "per_server_header": [],
            "aggregate_sha256": _mcp_manifest_aggregate(per_skill),
        }),
        encoding="utf-8",
    )
    (root / "aibox.lock").write_text("[processkit]\n", encoding="utf-8")
    (root / ".mcp.json").write_text(
        json.dumps({"mcpServers": {"processkit-gateway": {}}}),
        encoding="utf-8",
    )

    drift_results = _mcp_config_drift_run({"repo_root": root})
    check(
        "gateway-only .mcp.json does not trip harness-stale",
        any(r.id == "mcp_config_drift.gateway-mode" for r in drift_results),
        [r.to_dict() for r in drift_results],
    )

    (root / ".mcp.json").write_text(
        json.dumps({
            "mcpServers": {
                "processkit-gateway": {},
                "processkit-artifact-management": {},
            },
        }),
        encoding="utf-8",
    )
    gateway_results = _mcp_gateway_run({"repo_root": root})
    check(
        "mixed gateway + granular registration emits WARN",
        any(r.id == "mcp_gateway.mixed-registration"
            and r.severity == "WARN" for r in gateway_results),
        [r.to_dict() for r in gateway_results],
    )

# ---------------------------------------------------------------------------
# Test 6d: manifest generator keeps gateway outside granular aggregate
# ---------------------------------------------------------------------------
print("\n[6d] manifest generator — gateway metadata is separate")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    granular_cfg = (
        root / "context" / "skills" / "processkit" /
        "artifact-management" / "mcp" / "mcp-config.json"
    )
    gateway_cfg = (
        root / "context" / "skills" / "processkit" /
        "processkit-gateway" / "mcp" / "mcp-config.json"
    )
    source_only_cfg = (
        root / "src" / "context" / "skills" / "processkit" /
        "runtime-prune" / "mcp" / "mcp-config.json"
    )
    granular_cfg.parent.mkdir(parents=True)
    gateway_cfg.parent.mkdir(parents=True)
    source_only_cfg.parent.mkdir(parents=True)
    granular_cfg.write_text(
        '{"mcpServers":{"processkit-artifact-management":{}}}\n',
        encoding="utf-8",
    )
    gateway_cfg.write_text(
        '{"mcpServers":{"processkit-gateway":{}}}\n',
        encoding="utf-8",
    )
    source_only_cfg.write_text(
        '{"mcpServers":{"processkit-runtime-prune":{}}}\n',
        encoding="utf-8",
    )

    generator_path = _REPO_ROOT / "scripts" / "generate-mcp-manifest.py"
    spec = importlib.util.spec_from_file_location(
        "generate_mcp_manifest_test", generator_path
    )
    assert spec and spec.loader
    generator = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(generator)

    entries = generator._collect_entries(root)
    gateway_entries = generator._collect_gateway_entries(root)
    check(
        "gateway config excluded from per_skill entries",
        "context/skills/processkit/processkit-gateway/mcp/mcp-config.json"
        not in [e["path"] for e in entries],
        entries,
    )
    check(
        "source-only shipped MCP config included with context path",
        [e["path"] for e in entries] == [
            "context/skills/processkit/artifact-management/mcp/mcp-config.json",
            "context/skills/processkit/runtime-prune/mcp/mcp-config.json",
        ],
        entries,
    )
    check(
        "gateway config collected separately",
        [e["path"] for e in gateway_entries] == [
            "context/skills/processkit/processkit-gateway/mcp/mcp-config.json"
        ],
        gateway_entries,
    )

# ---------------------------------------------------------------------------
# Test 6e: release MCP preauth validator derives truth from shipped configs
# ---------------------------------------------------------------------------
print("\n[6e] release MCP preauth validator — shipped configs are truth")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    runtime_cfg = (
        root / "context" / "skills" / "processkit" /
        "runtime-prune" / "mcp" / "mcp-config.json"
    )
    preauth = (
        root / "context" / "skills" / "processkit" /
        "skill-gate" / "assets" / "preauth.json"
    )
    manifest = root / "context" / ".processkit-mcp-manifest.json"
    runtime_cfg.parent.mkdir(parents=True)
    preauth.parent.mkdir(parents=True)
    manifest.parent.mkdir(parents=True, exist_ok=True)
    runtime_cfg.write_text(
        '{"mcpServers":{"processkit-runtime-prune":{}}}\n',
        encoding="utf-8",
    )
    preauth.write_text(
        json.dumps({
            "version": 1,
            "permissions": {"allow": []},
            "enabledMcpjsonServers": [],
            "codex": {"mcp": {"allowed_tools": []}},
        }),
        encoding="utf-8",
    )
    manifest.write_text(
        json.dumps({
            "version": 1,
            "generated_at": "2026-05-14T00:00:00Z",
            "processkit_version": "v0.test",
            "per_skill": [],
            "per_gateway": [],
            "per_server_header": [],
            "aggregate_sha256": "stale",
        }),
        encoding="utf-8",
    )

    validator_path = _REPO_ROOT / "scripts" / "validate-release-mcp-preauth.py"
    spec = importlib.util.spec_from_file_location(
        "validate_release_mcp_preauth_test", validator_path
    )
    assert spec and spec.loader
    validator = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(validator)

    validator_failures = validator.validate(root)
    check(
        "validator catches stale manifest and preauth",
        any("manifest per_skill missing" in f for f in validator_failures)
        and any("enabledMcpjsonServers missing" in f for f in validator_failures)
        and any("permissions.allow missing" in f for f in validator_failures)
        and any("codex.mcp.allowed_tools missing" in f for f in validator_failures),
        validator_failures,
    )

    runtime_rel = "context/skills/processkit/runtime-prune/mcp/mcp-config.json"
    preauth.write_text(
        json.dumps({
            "version": 1,
            "permissions": {"allow": ["mcp__processkit-runtime-prune__*"]},
            "enabledMcpjsonServers": ["processkit-runtime-prune"],
            "codex": {
                "mcp": {
                    "allowed_tools": ["mcp__processkit-runtime-prune__*"],
                },
            },
        }),
        encoding="utf-8",
    )
    manifest.write_text(
        json.dumps({
            "version": 1,
            "generated_at": "2026-05-14T00:00:00Z",
            "processkit_version": "v0.test",
            "per_skill": [{
                "path": runtime_rel,
                "sha256": validator._sha256_of_json(runtime_cfg),
            }],
            "per_gateway": [],
            "per_server_header": [],
            "aggregate_sha256": "not-checked-here",
        }),
        encoding="utf-8",
    )
    check("validator accepts aligned metadata", validator.validate(root) == [])

# ---------------------------------------------------------------------------
# Test 6f: v1_entity_drift — superseded v1 entities are surfaced
# ---------------------------------------------------------------------------
print("\n[6f] v1_entity_drift — superseded v1 entities are surfaced")

from checks.v1_entity_drift import run as _v1_entity_drift_run  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    actor = root / "context" / "actors" / "ACTOR-legacy.md"
    log = root / "context" / "logs" / "2026" / "05" / "LOG-legacy.md"
    actor.parent.mkdir(parents=True)
    log.parent.mkdir(parents=True)
    actor.write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: Actor
            metadata:
              id: ACTOR-legacy
            spec: {}
            ---
            """),
        encoding="utf-8",
    )
    log.write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: LogEntry
            metadata:
              id: LOG-legacy
            spec: {}
            ---
            """),
        encoding="utf-8",
    )

    drift = _v1_entity_drift_run({"repo_root": root})
    drift_by_id = {item.id: item for item in drift}
    check(
        "6f: v1 Actor in active dir emits superseded WARN",
        drift_by_id.get("v1.entity-superseded")
        and drift_by_id["v1.entity-superseded"].severity == "WARN"
        and "TeamMember" in drift_by_id["v1.entity-superseded"].message,
        [item.to_dict() for item in drift],
    )
    check(
        "6f: v1 LogEntry in append-only bucket is INFO",
        drift_by_id.get("v1.immutable-bucket")
        and drift_by_id["v1.immutable-bucket"].severity == "INFO",
        [item.to_dict() for item in drift],
    )

# ---------------------------------------------------------------------------
# Test 7: pk-doctor integration — invalid actor ID triggers ERROR
# ---------------------------------------------------------------------------
print("\n[7] pk-doctor integration — invalid actor ID emits ERROR")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)

    # Copy live actor schema into fixture tree.
    actor_schema_src = _SCHEMAS_SRC / "actor.yaml"
    if actor_schema_src.is_file():
        (root / "src" / "context" / "schemas" / "actor.yaml").write_text(
            actor_schema_src.read_text(encoding="utf-8"), encoding="utf-8"
        )

    # Plant an actor with an ID not in the allowlist.
    (root / "context" / "actors").mkdir(parents=True)
    (root / "context" / "actors" / "ACTOR-hacker.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Actor
            metadata:
              id: ACTOR-hacker
              created: 2026-04-21T00:00:00Z
            spec:
              type: ai-agent
              name: Hacker Bot
              active: true
            ---
            """),
        encoding="utf-8",
    )

    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, stub_path=stub)
    check("exit 1 for invalid actor ID", result.returncode == 1,
          f"got {result.returncode}\nstdout:\n{result.stdout[-600:]}")
    check(
        "invalid_actor_id_pattern in output",
        "invalid_actor_id_pattern" in result.stdout,
        result.stdout[-600:],
    )

# ---------------------------------------------------------------------------
# Test 8: derived-project schema_filename fallback (HappyReef)
# Ensures pk-doctor walks entity files even when the dogfood
# `src/context/schemas/` tree is absent — the bug that hid all
# entity-hygiene findings in aibox-installed repos.
# ---------------------------------------------------------------------------
print("\n[8] schema_filename — derived-project fallback to context/schemas/")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    # No src/ tree — schemas live at context/schemas/ (derived layout).
    (root / "context" / "schemas").mkdir(parents=True)
    src_logentry = _SCHEMAS_SRC / "logentry.yaml"
    if src_logentry.is_file():
        (root / "context" / "schemas" / "logentry.yaml").write_text(
            src_logentry.read_text(encoding="utf-8"), encoding="utf-8"
        )
    # Stub drift script so doctor doesn't blow up there.
    (root / "scripts").mkdir()
    drift = root / "scripts" / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)
    # Seed a malformed LogEntry — should now be caught.
    (root / "context" / "logs").mkdir()
    (root / "context" / "logs" / "LOG-20260425_1100-Fixture-event.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: LogEntry
            metadata:
              id: LOG-20260425_1100-Fixture-event
              created: '2026-04-25T11:00:00+00:00'
            spec:
              event_type: test.fixture
              timestamp: '2026-04-25T11:00:00+00:00'
              summary: missing actor — should be caught
            ---
            """),
        encoding="utf-8",
    )
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--category=schema_filename", stub_path=stub)
    check(
        "exit 1 (ERROR found via derived layout)",
        result.returncode == 1,
        f"got {result.returncode}; stdout: {result.stdout[-400:]}",
    )
    check(
        "schema.invalid actor-required ERROR fired",
        "'actor' is a required property" in result.stdout,
        result.stdout[-400:],
    )
    check(
        "walked > 0 entity files (no longer silent zero)",
        "walked 1 entity file" in result.stdout,
        result.stdout[-400:],
    )

# ---------------------------------------------------------------------------
# Test 8b: schema_filename --fix emits append-only logentry correction
# ---------------------------------------------------------------------------
print("\n[8b] schema_filename --fix emits append-only logentry.corrected")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    # No src/ tree for this test: use derived schema layout.
    (root / "context" / "schemas").mkdir(parents=True)
    src_logentry = _SCHEMAS_SRC / "logentry.yaml"
    if src_logentry.is_file():
        (root / "context" / "schemas" / "logentry.yaml").write_text(
            src_logentry.read_text(encoding="utf-8"), encoding="utf-8"
        )
    # Stub drift script so doctor can run in this fixture.
    (root / "scripts").mkdir()
    drift = root / "scripts" / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)

    # Seed one malformed LogEntry (id/date mismatches + missing actor).
    (root / "context" / "logs").mkdir(parents=True)
    malformed = root / "context" / "logs" / "LOG-20260425_1100-Fixture-event.md"
    malformed.write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: LogEntry
            metadata:
              id: LOG-20260424_1100-Fixture-event
              created: '2026-04-24T11:00:00+00:00'
            spec:
              event_type: test.fixture
              timestamp: '2026-04-25T11:00:00+00:00'
              summary: mismatched filename and no actor
            ---
            """),
        encoding="utf-8",
    )
    malformed_before = malformed.read_text(encoding="utf-8")
    stub = root / ".doctor-logentry.json"

    detect = _run_doctor(root, "--category=schema_filename", stub_path=stub)
    check(
        "detect-only catches schema_filename issues",
        detect.returncode == 1 and "schema.filename-id-mismatch" in detect.stdout
        and "schema.filename-date-mismatch" in detect.stdout
        and "'actor' is a required property" in detect.stdout,
        detect.stdout[-600:],
    )

    fixed = _run_doctor(
        root,
        "--category=schema_filename",
        "--fix=schema_filename",
        "--yes",
        stub_path=stub,
    )
    check(
        "schema_filename fix pass still exits ERROR (pre-fix checks are pre-status)",
        fixed.returncode == 1,
        f"got {fixed.returncode}; stdout: {fixed.stdout[-400:]}",
    )
    payload_fix = json.loads(stub.read_text())
    check(
        "fix run emitted at least one correction LogEntry",
        any(
            isinstance(item, dict)
            and item.get("correction_log_id")
            for item in payload_fix.get("details", {}).get("fixes_applied", [])
        ),
        json.dumps(payload_fix.get("details", {}).get("fixes_applied", []), indent=2),
    )
    check(
        "original logfrontmatter untouched",
        malformed.read_text(encoding="utf-8") == malformed_before,
    )

    corrected = [
        p for p in (root / "context" / "logs").rglob("LOG-*-Fix-*.md")
    ]
    check("exactly one append-only correction file written", len(corrected) == 1, str(corrected))
    if corrected:
        corrected_fm_text = corrected[0].read_text(encoding="utf-8").split("---", 2)
        if len(corrected_fm_text) >= 3:
            corrected_fm = yaml.safe_load(corrected_fm_text[1])
        else:
            corrected_fm = {}
        check(
            "correction has logentry.corrected event_type",
            isinstance(corrected_fm, dict)
            and corrected_fm.get("spec", {}).get("event_type")
            == "logentry.corrected",
        )
        check(
            "correction targets the malformed log",
            isinstance(corrected_fm.get("spec", {}).get("details"), dict)
            and corrected_fm["spec"]["details"].get("corrects", {}).get("id")
            == "LOG-20260424_1100-Fixture-event",
        )

    fixed_detect = _run_doctor(root, "--category=schema_filename", stub_path=stub)
    payload_fixed_detect = json.loads(stub.read_text())
    fixed_stats = payload_fixed_detect.get("details", {}).get("categories", {}).get(
        "schema_filename", {}
    )
    check(
        "re-run drops schema_filename errors after correction",
        fixed_stats.get("ERROR", 0) == 0 and fixed_stats.get("WARN", 0) == 0,
    )
    check(
        "re-run keeps schema_filename as INFO-only",
        fixed_stats.get("INFO", 0) >= 1,
        str(fixed_stats),
    )

# ---------------------------------------------------------------------------
# Test 9: migrations layout fallback (DeepMoss)
# Ensures pk-doctor counts true Migration entities sitting at the top
# level of context/migrations/ (derived-project layout) and skips
# aibox-CLI upgrade docs.
# ---------------------------------------------------------------------------
print("\n[9] migrations — derived-project layout (top-level + applied/) fallback")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    (root / "context" / "migrations" / "applied").mkdir(parents=True)
    (root / "scripts").mkdir()
    drift = root / "scripts" / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)
    # Real Migration entity at the TOP level (derived layout).
    (root / "context" / "migrations" / "MIG-20260425T1100-test-fixture.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Migration
            metadata:
              id: MIG-20260425T1100-test-fixture
              created: '2026-04-25T11:00:00+00:00'
            spec:
              source: processkit
              state: pending
            ---
            test fixture migration
            """),
        encoding="utf-8",
    )
    # aibox-CLI upgrade doc — must be filtered out.
    (root / "context" / "migrations" / "20260425_1100_0.18.5-to-0.18.6.md").write_text(
        "# CLI upgrade doc — not a Migration entity\n",
        encoding="utf-8",
    )
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--category=migrations", stub_path=stub)
    check(
        "1 pending Migration counted (CLI doc filtered)",
        "1 pending migration" in result.stdout,
        result.stdout[-400:],
    )

# ---------------------------------------------------------------------------
# Test 10: skill_dag — clean roster (no missing refs, no cycles, no violations)
# ---------------------------------------------------------------------------
print("\n[10] skill_dag — clean roster produces 0 errors")

from checks.skill_dag import run as _skill_dag_run  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skills = root / "context" / "skills"
    # category: processkit; skills: alpha (layer 0) and beta (layer 1 uses alpha)
    (skills / "processkit" / "alpha").mkdir(parents=True)
    (skills / "processkit" / "alpha" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: alpha
            metadata:
              processkit:
                layer: 0
            ---
            """),
        encoding="utf-8",
    )
    (skills / "processkit" / "beta").mkdir(parents=True)
    (skills / "processkit" / "beta" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: beta
            metadata:
              processkit:
                layer: 1
                uses:
                  - skill: alpha
                    purpose: depends on alpha
            ---
            """),
        encoding="utf-8",
    )
    ctx = {"repo_root": root}
    results = _skill_dag_run(ctx)
    errors = [r for r in results if r.severity == "ERROR"]
    infos = [r for r in results if r.id == "skill.dag.summary"]
    check("10a: clean roster — 0 ERRORs", len(errors) == 0,
          f"errors: {[r.message for r in errors]}")
    check("10b: summary INFO emitted", len(infos) == 1, str(infos))
    check("10c: walked 2 skills in summary",
          infos and "walked 2 skill(s)" in infos[0].message,
          infos[0].message if infos else "")

# ---------------------------------------------------------------------------
# Test 11: skill_dag — missing reference triggers ERROR
# ---------------------------------------------------------------------------
print("\n[11] skill_dag — missing reference")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skills = root / "context" / "skills"
    (skills / "processkit" / "foo").mkdir(parents=True)
    (skills / "processkit" / "foo" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: foo
            metadata:
              processkit:
                layer: 2
                uses:
                  - skill: nonexistent-bar
                    purpose: references a skill that does not exist
            ---
            """),
        encoding="utf-8",
    )
    ctx = {"repo_root": root}
    results = _skill_dag_run(ctx)
    missing_errors = [r for r in results if r.id == "skill.dag.missing-ref"]
    check("11a: missing-ref ERROR fired",
          len(missing_errors) == 1,
          f"got: {[r.message for r in missing_errors]}")
    check("11b: error message names both skills",
          missing_errors and "foo" in missing_errors[0].message and "nonexistent-bar" in missing_errors[0].message,
          missing_errors[0].message if missing_errors else "")

# ---------------------------------------------------------------------------
# Test 12: skill_dag — 3-node cycle triggers ERROR
# ---------------------------------------------------------------------------
print("\n[12] skill_dag — 3-node cycle detection")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skills = root / "context" / "skills"
    for sname, dep in [("aaa", "ccc"), ("bbb", "aaa"), ("ccc", "bbb")]:
        (skills / "processkit" / sname).mkdir(parents=True)
        (skills / "processkit" / sname / "SKILL.md").write_text(
            textwrap.dedent(f"""\
                ---
                name: {sname}
                metadata:
                  processkit:
                    layer: 1
                    uses:
                      - skill: {dep}
                        purpose: cyclic dep
                ---
                """),
            encoding="utf-8",
        )
    ctx = {"repo_root": root}
    results = _skill_dag_run(ctx)
    cycle_errors = [r for r in results if r.id == "skill.dag.cycle"]
    check("12a: cycle ERROR fired",
          len(cycle_errors) >= 1,
          f"got: {[r.message for r in cycle_errors]}")
    check("12b: cycle message contains 'cycle detected'",
          any("cycle detected" in r.message for r in cycle_errors),
          str([r.message for r in cycle_errors]))

# ---------------------------------------------------------------------------
# Test 13: skill_dag — layer violation triggers ERROR
# ---------------------------------------------------------------------------
print("\n[13] skill_dag — layer violation")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skills = root / "context" / "skills"
    # low (layer 0) uses high (layer 3) — violation
    (skills / "processkit" / "low").mkdir(parents=True)
    (skills / "processkit" / "low" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: low
            metadata:
              processkit:
                layer: 0
                uses:
                  - skill: high
                    purpose: upward layer reference
            ---
            """),
        encoding="utf-8",
    )
    (skills / "processkit" / "high").mkdir(parents=True)
    (skills / "processkit" / "high" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: high
            metadata:
              processkit:
                layer: 3
            ---
            """),
        encoding="utf-8",
    )
    ctx = {"repo_root": root}
    results = _skill_dag_run(ctx)
    layer_errors = [r for r in results if r.id == "skill.dag.layer-violation"]
    check("13a: layer-violation ERROR fired",
          len(layer_errors) == 1,
          f"got: {[r.message for r in layer_errors]}")
    check("13b: error message mentions both layers",
          layer_errors and "layer 0" in layer_errors[0].message and "layer 3" in layer_errors[0].message,
          layer_errors[0].message if layer_errors else "")
    check("13c: error message names both skills",
          layer_errors and "low" in layer_errors[0].message and "high" in layer_errors[0].message,
          layer_errors[0].message if layer_errors else "")

# ---------------------------------------------------------------------------
# Test 14: skill_dag -- --category=skill_dag integration
# ---------------------------------------------------------------------------
print("\n[14] skill_dag — integration via --category=skill_dag")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    # Stub drift + minimal seed
    (root / "scripts").mkdir()
    drift = root / "scripts" / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)
    # One clean skill under context/skills/
    skills = root / "context" / "skills"
    (skills / "processkit" / "clean-skill").mkdir(parents=True)
    (skills / "processkit" / "clean-skill" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: clean-skill
            metadata:
              processkit:
                layer: 0
            ---
            """),
        encoding="utf-8",
    )
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--category=skill_dag", stub_path=stub)
    check("14a: --category=skill_dag runs without crash",
          result.returncode in (0, 1),
          f"got {result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    check("14b: skill_dag section present in output",
          "## skill_dag" in result.stdout,
          result.stdout[-400:])
check("14c: summary line present",
      "walked" in result.stdout and "skill(s)" in result.stdout,
      result.stdout[-400:])

# ---------------------------------------------------------------------------
# Test 15: context_consumption checkpoint/report CLI
# ---------------------------------------------------------------------------
print("\n[15] context_consumption — checkpoint report labels local estimates")

from checks.commands_consistency import run as _run_commands_consistency  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skill_dir = root / "context" / "skills" / "processkit" / "demo-skill"
    (skill_dir / "commands").mkdir(parents=True)
    (skill_dir / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            metadata:
              processkit:
                commands:
                  - name: demo-run
            ---
            """),
        encoding="utf-8",
    )
    (skill_dir / "commands" / "demo-run.md").write_text(
        "Use the demo skill.\n",
        encoding="utf-8",
    )

    findings = _run_commands_consistency({"repo_root": root})
    check(
        "commands_consistency rejects unprefixed processkit commands",
        any(f.id == "commands_consistency.unprefixed-processkit-command"
            for f in findings),
        "\n".join(f"{f.id}: {f.message}" for f in findings),
    )

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skill_dir = root / "context" / "skills" / "processkit" / "demo-skill"
    (skill_dir / "commands").mkdir(parents=True)
    (root / ".claude" / "commands").mkdir(parents=True)
    (root / ".agents" / "skills" / "pk-demo").mkdir(parents=True)
    (skill_dir / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            metadata:
              processkit:
                commands:
                  - name: pk-demo
                    args: "thing"
            ---
            """),
        encoding="utf-8",
    )
    command_text = textwrap.dedent("""\
        ---
        argument-hint: "thing"
        allowed-tools: []
        ---

        Use the demo skill for $ARGUMENTS.
        """)
    (skill_dir / "commands" / "pk-demo.md").write_text(
        command_text,
        encoding="utf-8",
    )
    (root / ".claude" / "commands" / "pk-demo.md").write_text(
        command_text,
        encoding="utf-8",
    )
    (root / ".agents" / "skills" / "pk-demo" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: pk-demo
            description: "Use the demo skill for $ARGUMENTS."
            ---

            Use the demo skill for $ARGUMENTS.
            """),
        encoding="utf-8",
    )

    findings = _run_commands_consistency({"repo_root": root})
    check(
        "commands_consistency accepts matching command projections",
        all(f.severity == "INFO" for f in findings),
        "\n".join(f"{f.id}: {f.message}" for f in findings),
    )

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skill_dir = root / "context" / "skills" / "processkit" / "demo-skill"
    (skill_dir / "commands").mkdir(parents=True)
    (root / ".claude" / "commands").mkdir(parents=True)
    (root / ".agents" / "skills" / "pk-extra").mkdir(parents=True)
    (skill_dir / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            metadata:
              processkit:
                commands:
                  - name: pk-demo
                    args: "thing"
            ---
            """),
        encoding="utf-8",
    )
    (skill_dir / "commands" / "pk-demo.md").write_text(
        textwrap.dedent("""\
            ---
            argument-hint: "other"
            allowed-tools: []
            ---

            Use the demo skill.
            """),
        encoding="utf-8",
    )
    (root / ".claude" / "commands" / "pk-extra.md").write_text(
        "orphan\n",
        encoding="utf-8",
    )
    (root / ".agents" / "skills" / "pk-extra" / "SKILL.md").write_text(
        "orphan\n",
        encoding="utf-8",
    )

    findings = _run_commands_consistency({"repo_root": root})
    ids = {f.id for f in findings}
    check(
        "commands_consistency detects projection and argument drift",
        {
            "commands_consistency.argument-hint-mismatch",
            "commands_consistency.missing-claude-projection",
            "commands_consistency.claude-only-command",
            "commands_consistency.missing-agent-skill-projection",
            "commands_consistency.agent-only-command",
        }.issubset(ids),
        "\n".join(f"{f.id}: {f.message}" for f in findings),
    )

from checks.context_consumption import (  # noqa: E402
    compare_checkpoints as _compare_context_checkpoints,
    write_checkpoint as _write_context_checkpoint,
)

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    (root / ".agents" / "skills" / "pk-resume").mkdir(parents=True)
    (root / ".agents" / "skills" / "pk-resume" / "SKILL.md").write_text(
        "resume command\n",
        encoding="utf-8",
    )
    (root / "context" / "skills" / "processkit" / "alpha").mkdir(parents=True)
    (root / "context" / "skills" / "processkit" / "alpha" / "SKILL.md").write_text(
        "alpha skill\n",
        encoding="utf-8",
    )

    _write_context_checkpoint(root, "before")
    (root / ".agents" / "skills" / "pk-extra").mkdir(parents=True)
    (root / ".agents" / "skills" / "pk-extra" / "SKILL.md").write_text(
        "extra command adapter text\n" * 3,
        encoding="utf-8",
    )
    _write_context_checkpoint(root, "after")

    report = _compare_context_checkpoints(root, "before", "after")
    check(
        "report has positive token delta",
        report["totals"]["delta"]["estimated_tokens"] > 0,
        json.dumps(report["totals"], indent=2),
    )
    check(
        "report labels estimates as non-billing",
        "not provider-billed token usage" in report["billing_notice"],
        report["billing_notice"],
    )
    check(
        "top file delta attributes added command adapter",
        any(
            item["path"] == ".agents/skills/pk-extra/SKILL.md"
            and item["status"] == "added"
            for item in report["top_file_deltas"]
        ),
        json.dumps(report["top_file_deltas"], indent=2)[:500],
    )

    cli = _SCRIPTS_DIR / "checks" / "context_consumption.py"
    result = subprocess.run(
        [sys.executable, str(cli), "--repo-root", str(root), "report", "before", "after"],
        capture_output=True,
        text=True,
    )
    check("report CLI exits 0", result.returncode == 0, result.stderr)
    check("report CLI prints billing notice",
          "not provider-billed token usage" in result.stdout,
          result.stdout)

# ---------------------------------------------------------------------------
# Test 16: v2 plan guardrails from SmoothRiver Sprint A/C/D catalogue
# ---------------------------------------------------------------------------
print("\n[16] v2 plan guardrails — vocabulary, sharding, contracts")

from checks.schema_vocabulary import run as _schema_vocab_run  # noqa: E402
from checks.sharding import run as _sharding_run  # noqa: E402
from checks.v2_contracts import run as _v2_contracts_run  # noqa: E402
from checks.common import CheckResult as _CheckResult  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    (root / "context" / "schemas").mkdir(parents=True)
    (root / "context" / "artifacts").mkdir(parents=True)
    (root / "context" / "bindings").mkdir(parents=True)
    (root / "context" / "migrations" / "applied").mkdir(parents=True)
    (root / "context" / "workitems").mkdir(parents=True)
    (root / "context" / "logs").mkdir(parents=True)
    (root / "context" / "skills" / "processkit" /
     "index-management" / "config").mkdir(parents=True)

    (root / "context" / "schemas" / "artifact.yaml").write_text(
        textwrap.dedent("""\
            spec:
              known_kinds: [document, cost-policy]
            """),
        encoding="utf-8",
    )
    (root / "context" / "schemas" / "binding.yaml").write_text(
        textwrap.dedent("""\
            spec:
              known_types: [role-assignment, triage-classification]
            """),
        encoding="utf-8",
    )
    (root / "context" / "schemas" / "workitem.yaml").write_text(
        "spec:\n  known_types: [task]\n",
        encoding="utf-8",
    )
    (root / "context" / "schemas" / "logentry.yaml").write_text(
        "spec:\n  known_event_types: [test.event]\n",
        encoding="utf-8",
    )
    (root / "context" / "schemas" / "migration.yaml").write_text(
        "spec:\n  known_kinds: [source-upgrade, schema-extension, data-fix]\n",
        encoding="utf-8",
    )
    (root / "context" / "artifacts" / "ART-bad-kind.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Artifact
            metadata:
              id: ART-bad-kind
              created: 2026-04-30T00:00:00Z
            spec:
              name: Bad kind
              kind: unknown-policy-shape
            ---
            """),
        encoding="utf-8",
    )
    (root / "context" / "bindings" / "BIND-bad-type.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Binding
            metadata:
              id: BIND-bad-type
              created: 2026-04-30T00:00:00Z
            spec:
              type: unknown-binding-shape
              subject: ART-a
              target: ART-b
            ---
            """),
        encoding="utf-8",
    )
    (root / "context" / "migrations" / "applied" /
     "MIG-bad-kind.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Migration
            metadata:
              id: MIG-bad-kind
              created: 2026-04-30T00:00:00Z
            spec:
              source: processkit
              kind: mystery-transition
              from_version: v0.1.0
              to_version: v0.2.0
              state: applied
              source_api_version: processkit.projectious.work/v1
              source_processkit_version: v0.1.0
              target_api_version: processkit.projectious.work/v2
              target_processkit_version: v0.2.0
              apply_mode: one-shot
            ---
            """),
        encoding="utf-8",
    )
    (root / "context" / "workitems" / "BACK-legacy-feature.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: WorkItem
            metadata:
              id: BACK-legacy-feature
              created: 2026-04-30T00:00:00Z
            spec:
              title: Legacy feature work item
              state: done
              type: feature
            ---
            """),
        encoding="utf-8",
    )
    (root / "context" / "logs" / "LOG-legacy-release.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: LogEntry
            metadata:
              id: LOG-legacy-release
              created: 2026-04-30T00:00:00Z
            spec:
              event_type: release.shipped
              actor: system
              timestamp: 2026-04-30T00:00:00Z
            ---
            """),
        encoding="utf-8",
    )
    vocab_results = _schema_vocab_run({"repo_root": root})
    check(
        "16a: unknown Artifact kind emits plan check ID",
        any(r.id == "schema.unknown-kind-without-schema-entry"
            for r in vocab_results),
    )
    check(
        "16b: unknown Binding type emits plan check ID",
        any(r.id == "schema.unknown-type-without-schema-entry"
            for r in vocab_results),
    )
    check(
        "16c: unknown Migration kind emits plan check ID",
        any(r.id == "schema.unknown-migration-kind-without-schema-entry"
            for r in vocab_results),
    )
    vocab_messages = "\n".join(r.message for r in vocab_results)
    check(
        "16c1: legacy schema vocabulary is rejected",
        "feature" in vocab_messages
        and "release.shipped" in vocab_messages,
        vocab_messages,
    )
    legacy_vocab = [
        r.to_dict() for r in vocab_results
        if "feature" in r.message or "release.shipped" in r.message
    ]
    check(
        "16c1b: legacy vocabulary requires migration",
        legacy_vocab
        and all(r["action_kind"] == "migration_needed" for r in legacy_vocab)
        and all(r["acceptable_resolution"] == "migrated" for r in legacy_vocab),
        json.dumps(legacy_vocab, indent=2),
    )
    policy_resolution = _CheckResult(
        severity="WARN",
        category="fixture",
        id="fixture.policy",
        message="policy decision needed",
    ).to_dict()
    check(
        "16c1c: policy findings do not accept terminal exceptions",
        policy_resolution["acceptable_resolution"] == "linked_tracking_item",
        json.dumps(policy_resolution, indent=2),
    )

    (root / "src" / "context" / "schemas").mkdir(parents=True)
    (root / "src" / "context" / "schemas" / "model.yaml").write_text(
        "kind: Schema\nmetadata:\n  id: SCHEMA-model\nspec: {}\n",
        encoding="utf-8",
    )
    (root / "src" / "context" / "schemas" / "INDEX.md").write_text(
        "- [model.yaml](model.yaml)\n"
        "- [statemachine.yaml](statemachine.yaml)\n",
        encoding="utf-8",
    )
    demotion_results = _schema_vocab_run({"repo_root": root})
    check(
        "16c2: demoted first-class Model schema is blocked from src/",
        any(r.id == "schema.demoted-kind-still-shipped"
            and "model.yaml" in r.message
            for r in demotion_results),
    )
    check(
        "16c3: demoted first-class schema index entry is blocked from src/",
        any(r.id == "schema.demoted-kind-still-indexed"
            and "StateMachine" in r.message
            for r in demotion_results),
    )

    settings = (
        root / "context" / "skills" / "processkit" /
        "index-management" / "config" / "settings.toml"
    )
    settings.write_text(
        textwrap.dedent("""\
            [sharding.workitem]
            pattern = "date-shard"
            template = "{year}/{month}/"
            activate_above_count = 1
            """),
        encoding="utf-8",
    )
    for suffix in ("one", "two"):
        (root / "context" / "workitems" / f"BACK-{suffix}.md").write_text(
            textwrap.dedent(f"""\
                ---
                apiVersion: processkit.projectious.work/v2
                kind: WorkItem
                metadata:
                  id: BACK-{suffix}
                  created: 2026-04-30T00:00:00Z
                spec:
                  title: {suffix}
                  state: backlog
                  type: task
                ---
                """),
            encoding="utf-8",
        )
    sharding_results = _sharding_run({"repo_root": root})
    check(
        "16d: workitem threshold check fires",
        any(r.id == "sharding.workitem-shard-threshold"
            for r in sharding_results),
    )

    (root / "context" / "artifacts" / "ART-new-policy.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Artifact
            metadata:
              id: ART-new-policy
              created: 2026-04-30T00:00:00Z
            spec:
              name: New policy
              kind: cost-policy
              supersedes: [ART-missing-policy]
            ---
            """),
        encoding="utf-8",
    )
    (root / "context" / "bindings" / "BIND-triage.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Binding
            metadata:
              id: BIND-triage
              created: 2026-04-30T00:00:00Z
            spec:
              type: triage-classification
              subject: NOTE-a
              target: BACK-one
              conditions: {}
            ---
            """),
        encoding="utf-8",
    )
    contract_results = _v2_contracts_run({"repo_root": root})
    check(
        "16e: policy supersedes chain-break check fires",
        any(r.id == "v2.policy-supersedes-chain-break"
            for r in contract_results),
    )
    check(
        "16f: inbox injection-mode check fires",
        any(r.id == "v2.inbox-injection-mode-untyped"
            for r in contract_results),
    )

# ---------------------------------------------------------------------------
# Test 17: entity storage policy doctor findings
# ---------------------------------------------------------------------------
print("\n[17] entity storage hygiene — legacy layout and policy signals")

from checks.entity_storage_hygiene import (  # noqa: E402
    run as _entity_storage_run,
)

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    ctx = root / "context"
    for rel in (
        "archive/actors-v1",
        "models",
        "migrations",
        "workitems/2026/05",
        "roles",
        "team-members/thrifty-otter/private",
    ):
        (ctx / rel).mkdir(parents=True, exist_ok=True)
    (ctx / ".DS_Store").write_text("host", encoding="utf-8")
    (ctx / "models" / "MODEL-legacy.md").write_text(
        "legacy\n",
        encoding="utf-8",
    )
    (ctx / "migrations" /
     "20260410_1523_0.17.6-to-0.17.9.md").write_text(
        "# briefing\n",
        encoding="utf-8",
    )
    (ctx / "workitems" /
     "BACK-20260411_0000-ProudTiger-legacy.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: WorkItem
            metadata:
              id: BACK-20260411_0000-ProudTiger-legacy
              created: 2026-04-11T00:00:00Z
            spec:
              title: Legacy root work
              state: backlog
              type: task
            ---
            """),
        encoding="utf-8",
    )
    (ctx / "workitems" / "2026" / "05" /
     "BACK-20260511_1528-StoutStream-modern.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: WorkItem
            metadata:
              id: BACK-20260511_1528-StoutStream-modern
              created: 2026-05-11T15:28:00Z
            spec:
              title: Modern sharded work
              state: backlog
              type: task
            ---
            """),
        encoding="utf-8",
    )
    (ctx / "workitems" /
     "BACK-20260512_1200-BrightLake-new-root.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: WorkItem
            metadata:
              id: BACK-20260512_1200-BrightLake-new-root
              created: 2026-05-12T12:00:00Z
            spec:
              title: New root work after sharding
              state: backlog
              type: task
            ---
            """),
        encoding="utf-8",
    )
    (ctx / "roles" /
     "ROLE-20260414_1100-GentleFalcon-project-manager.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Role
            metadata:
              id: ROLE-20260414_1100-GentleFalcon-project-manager
              created: 2026-04-14T11:00:00Z
            spec:
              name: Project Manager
            ---
            """),
        encoding="utf-8",
    )
    (ctx / "roles" / "ROLE-software-engineer-senior.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Role
            metadata:
              id: ROLE-software-engineer-senior
              created: 2026-04-14T11:00:00Z
            spec:
              name: Software Engineer
            ---
            """),
        encoding="utf-8",
    )
    (ctx / "team-members" / "thrifty-otter" /
     "team-member.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: TeamMember
            metadata:
              id: TEAMMEMBER-thrifty-otter
              created: 2026-04-22T00:00:00Z
            spec:
              type: human
              name: Owner
              slug: thrifty-otter
              active: true
            ---
            """),
        encoding="utf-8",
    )
    storage_results = _entity_storage_run({"repo_root": root})
    storage_ids = {r.id for r in storage_results}
    storage_by_id = {r.id: r for r in storage_results}
    for expected in {
        "storage.host-artifact",
        "storage.demoted-model-root",
        "storage.legacy-archive-policy",
        "storage.root-migration-briefings",
        "storage.mixed-layout",
        "storage.placeholder-timestamp",
        "storage.filename-policy-mixed",
        "storage.team-member-private-policy",
        "storage.human-team-member-slug-policy",
        "storage.policy-summary",
    }:
        check(f"17: emits {expected}", expected in storage_ids)
    briefing = storage_by_id["storage.root-migration-briefings"].to_dict()
    check("17: migration briefings remain actionable",
          briefing["action_required"] is True)
    check("17: migration briefings request archive action",
          briefing["action_kind"] == "archive_needed",
          json.dumps(briefing, indent=2))
    for sid in {
        "storage.mixed-layout",
        "storage.placeholder-timestamp",
        "storage.filename-policy-mixed",
    }:
        payload = storage_by_id[sid].to_dict()
        check(
            f"17: {sid} requires migration",
            payload["action_kind"] == "migration_needed"
            and payload["acceptable_resolution"] == "migrated",
            json.dumps(payload, indent=2),
        )


# ---------------------------------------------------------------------------
# Test 17a: legacy runtime Migration IDs are tolerated historical IDs
# ---------------------------------------------------------------------------
print("\n[17a] entity storage hygiene — legacy runtime migration IDs")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    applied = root / "context" / "migrations" / "applied"
    applied.mkdir(parents=True)
    (applied / "MIG-20260517_1500-QuietLake.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Migration
            metadata:
              id: MIG-20260517_1500-QuietLake
              created: 2026-05-17T15:00:00Z
            spec:
              state: applied
            ---
            """),
        encoding="utf-8",
    )
    (applied / "MIG-RUNTIME-20260517T150001.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Migration
            metadata:
              id: MIG-RUNTIME-20260517T150001
              created: 2026-05-17T15:00:01Z
            spec:
              state: applied
            ---
            """),
        encoding="utf-8",
    )
    storage_results = _entity_storage_run({"repo_root": root})
    storage_ids = {r.id for r in storage_results}
    check(
        "17a: legacy runtime migration emits INFO",
        "storage.legacy-runtime-migration-filenames" in storage_ids,
        [r.to_dict() for r in storage_results],
    )
    mixed = [
        r.to_dict() for r in storage_results
        if r.id == "storage.filename-policy-mixed"
    ]
    check(
        "17a: legacy runtime migration does not trigger mixed policy WARN",
        not mixed,
        json.dumps(mixed, indent=2),
    )


# ---------------------------------------------------------------------------
# Test 17b: binding filename warning stays strict and actionable
# ---------------------------------------------------------------------------
print("\n[17b] context hygiene — mixed Binding filenames advise data fix")

from checks.context_hygiene import (  # noqa: E402
    run as _context_hygiene_run,
)

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    bindings = root / "context" / "bindings"
    bindings.mkdir(parents=True)
    (bindings / "BIND-20260517_1500-QuietLake-role-slot-fill.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Binding
            metadata:
              id: BIND-20260517_1500-QuietLake-role-slot-fill
              created: 2026-05-17T15:00:00Z
            spec:
              type: role-slot-fill
              subject: TEAMMEMBER-cora
              target: SLOT-20260517_1500-QuietLake
            ---
            """),
        encoding="utf-8",
    )
    (bindings / "BIND-product-manager-senior-hf3f8e9.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v2
            kind: Binding
            metadata:
              id: BIND-product-manager-senior-hf3f8e9
              created: 2026-05-17T15:00:00Z
            spec:
              type: role-slot-fill
              subject: TEAMMEMBER-finn
              target: SLOT-product-manager-senior
            ---
            """),
        encoding="utf-8",
    )
    results = _context_hygiene_run({"repo_root": root})
    mixed = [
        r.to_dict() for r in results
        if r.id == "binding.filename-style-mixed"
    ]
    check("17b: mixed Binding filenames still WARN", bool(mixed), results)
    check(
        "17b: warning calls out role-slot-fill and LogEntry rewrites",
        bool(mixed)
        and "role-slot-fill" in mixed[0]["message"]
        and "LogEntry references" in mixed[0]["suggested_fix"],
        json.dumps(mixed, indent=2),
    )


# ---------------------------------------------------------------------------
# Test 17c: entity_storage_hygiene --fix archives CLI migration briefings
# ---------------------------------------------------------------------------
print("\n[17c] entity_storage_hygiene --fix moves uncompleted CLI briefings")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    (root / "context" / "migrations").mkdir(parents=True)
    (root / "scripts").mkdir()
    drift = root / "scripts" / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)

    active_briefing = (
        root / "context" / "migrations" / "20260410_1523_0.17.6-to-0.17.9.md"
    )
    completed_briefing = (
        root / "context" / "migrations" / "20260409_0900_0.17.5-to-0.17.6.md"
    )
    active_briefing.write_text(
        "# Active historical migration note\n",
        encoding="utf-8",
    )
    completed_briefing.write_text(
        textwrap.dedent("""\
            # Migration: v0.17.5 → v0.17.6

            **Status:** completed
            """),
        encoding="utf-8",
    )

    storage_results = _entity_storage_run({"repo_root": root})
    storage_ids = {r.id for r in storage_results}
    check("17c: detects root migration briefing", "storage.root-migration-briefings" in storage_ids)

    stub = root / ".doctor-logentry.json"
    fixed = _run_doctor(
        root,
        "--category=entity_storage_hygiene",
        "--fix=entity_storage_hygiene",
        "--yes",
        stub_path=stub,
    )
    check(
        "17c: fix pass exits cleanly",
        fixed.returncode == 0,
        f"got {fixed.returncode}; stdout: {fixed.stdout[-400:]}",
    )

    fix_payload = json.loads(stub.read_text())
    fix_records = fix_payload.get("details", {}).get("fixes_applied", [])
    archived = [
        item for item in fix_records
        if isinstance(item, dict)
        and item.get("status") == "archived"
    ]
    check(
        "17c: fix run emits archived status",
        len(archived) == 1,
        json.dumps(fix_records, indent=2),
    )
    check(
        "17c: briefing moved out of migration root",
        not active_briefing.exists(),
        str(active_briefing),
    )
    check(
        "17c: archive contains briefing payload",
        (root / "context" / "archive" / "cli-migration-briefings" /
         active_briefing.name).exists(),
    )
    check(
        "17c: completed briefing stays in place",
        completed_briefing.exists(),
        str(completed_briefing),
    )

    storage_after = _entity_storage_run({"repo_root": root})
    storage_after_ids = {r.id for r in storage_after}
    check(
        "17c: no uncompleted briefing finding remains after fix",
        "storage.root-migration-briefings" not in storage_after_ids
        or len(storage_after) == 0,
        json.dumps([r.to_dict() for r in storage_after], indent=2),
    )

# ---------------------------------------------------------------------------
# Test 18: agents_md_hygiene — managed blocks, references, stale guidance
# ---------------------------------------------------------------------------
print("\n[18] agents_md_hygiene — AGENTS.md reconciliation briefing")

from checks.agents_md_hygiene import run as _agents_md_hygiene_run  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    (root / "AGENTS.md").write_text(
        textwrap.dedent("""\
            # AGENTS.md

            <!-- pk-compliance-contract v2 BEGIN -->
            Call `record_decision` after accepted recommendations.
            <!-- pk-compliance-contract v2 END -->

            ## Team

            | Role | Model tier |
            |------|------------|
            | engineer | Sonnet |

            Legacy Template Actors live in context/actors/.
            Old process customizations live under spec.x_aibox.

            <!-- pk-commands BEGIN -->
            <!--
            build: "make build"
            test: "make test"
            -->
            <!-- pk-commands END -->
            """),
        encoding="utf-8",
    )
    (root / "CLAUDE.md").write_text(
        "Duplicated policy surface that never mentions the canonical file.\n",
        encoding="utf-8",
    )
    results = _agents_md_hygiene_run({"repo_root": root})
    ids = {r.id for r in results}
    for expected in {
        "agents_md_hygiene.managed-block-missing",
        "agents_md_hygiene.references-missing",
        "agents_md_hygiene.semantic-cleanup",
        "agents_md_hygiene.commands-schema",
        "agents_md_hygiene.provider-pointer-not-thin",
    }:
        check(f"18: emits {expected}", expected in ids, [r.to_dict() for r in results])
    actionable = [r.to_dict() for r in results if r.id.endswith("references-missing")]
    check(
        "18: reconciliation finding carries project-agent briefing",
        actionable
        and actionable[0]["action_required"] is True
        and "briefing" in actionable[0].get("extra", {}),
        json.dumps(actionable, indent=2),
    )

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    agents_text = textwrap.dedent("""\
        # AGENTS.md

        <!-- pk-managed:pk-compliance-contract-v2 BEGIN -->
        <!-- pk-compliance-contract v2 BEGIN -->
        Run `pk-resume` on session start.
        Call `find_skill` and `route_task` before processkit work.
        Use index-management and MCP tools for entity IO.
        Do not edit context/templates/.
        Call `record_decision` or `skip_decision_record`.
        Use migration-management for context/migrations/pending.
        Preserve recommended_team_member_slug and recommended_model_class.
        TeamMember defaults bind to model-profile and model-spec artifacts.
        Re-merge .mcp.json from .processkit-mcp-manifest.json and
        mcp-config.json changes.
        Keep CLAUDE.md, CODEX.md, .cursor/rules as pointers to AGENTS.md.
        <!-- pk-compliance-contract v2 END -->
        <!-- pk-managed:pk-compliance-contract-v2 END -->

        <!-- pk-managed:pk-commands BEGIN -->
        <!-- pk-commands BEGIN -->
        <!--
        build: "make build"
        test: "make test"
        lint: ""
        fmt: ""
        typecheck: ""
        -->
        <!-- pk-commands END -->
        <!-- pk-managed:pk-commands END -->
        """)
    (root / "AGENTS.md").write_text(agents_text, encoding="utf-8")
    (root / "src").mkdir()
    (root / "src" / "AGENTS.md").write_text(agents_text, encoding="utf-8")
    (root / "CLAUDE.md").write_text("See AGENTS.md.\n", encoding="utf-8")
    clean_results = _agents_md_hygiene_run({"repo_root": root})
    check(
        "18: clean AGENTS.md emits clean INFO",
        [r.id for r in clean_results] == ["agents_md_hygiene.clean"],
        [r.to_dict() for r in clean_results],
    )

print("\n[19] runtime_health — container-local probe helpers")

if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))
from checks import runtime_health

check(
    "19: detects enabled Codex harness from aibox.toml",
    runtime_health._codex_enabled_from_text(
        '[ai]\nharnesses = [\n  { harness = "codex", enable = true },\n]\n'
    )
    if hasattr(runtime_health, "_codex_enabled_from_text")
    else runtime_health._codex_enabled,
)
check(
    "19: parses sleep-infinity PID1 command",
    runtime_health._is_sleep_infinity("sleep infinity") is True,
    "sleep-infinity command should be flagged",
)
check(
    "19: ignores non-sleep PID1 command",
    runtime_health._is_sleep_infinity("docker-init -- /bin/bash") is False,
    "docker-init should not be flagged",
)
check(
    "19: parses cgroup memory event counters",
    runtime_health._parse_key_value_ints("low 0\noom_kill 2\n").get("oom_kill") == 2,
    "oom_kill counter should parse",
)

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    target = root / ".aibox-home" / ".cache"
    result = runtime_health._write_probe(target)
    check(
        "19: runtime-home write probe creates and removes probe file",
        result.severity == "INFO"
        and result.id == "runtime-home.write-ok"
        and not list(target.glob(".pk-doctor-write-*")),
        json.dumps(result.to_dict(), indent=2),
    )

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    restored = {
        "_is_container": runtime_health._is_container,
        "_configured_lnav": runtime_health._configured_lnav,
        "_sqlite_vec_probe": runtime_health._sqlite_vec_probe,
        "_codex_enabled": runtime_health._codex_enabled,
        "_bwrap_smoke": runtime_health._bwrap_smoke,
        "_pid1_finding": runtime_health._pid1_finding,
        "_cgroup_findings": runtime_health._cgroup_findings,
        "_process_counts": runtime_health._process_counts,
        "_powerkit_configured": runtime_health._powerkit_configured,
        "_aibox_runtime_markers": runtime_health._aibox_runtime_markers,
        "_runtime_home_findings": runtime_health._runtime_home_findings,
        "shutil_which": runtime_health.shutil.which,
    }
    try:
        runtime_health._is_container = lambda: True
        runtime_health._configured_lnav = lambda _root: True
        runtime_health.shutil.which = lambda name: (
            "/usr/bin/lnav" if name == "lnav" else f"/usr/bin/{name}"
        )
        runtime_health._sqlite_vec_probe = lambda: (
            True,
            "sqlite-vec import/load probe passed",
        )
        runtime_health._codex_enabled = lambda _root: True
        runtime_health._bwrap_smoke = lambda: (True, "bwrap smoke probe passed")
        runtime_health._pid1_finding = lambda: runtime_health.CheckResult(
            severity="INFO",
            category=runtime_health.CATEGORY,
            id="runtime.pid1-ok",
            message="PID 1 runtime process is healthy",
        )
        runtime_health._cgroup_findings = lambda: [
            runtime_health.CheckResult(
                severity="INFO",
                category=runtime_health.CATEGORY,
                id="runtime.cgroup-memory-current",
                message="cgroup memory.current is 1 byte",
            )
        ]
        runtime_health._process_counts = lambda: [
            runtime_health.CheckResult(
                severity="INFO",
                category=runtime_health.CATEGORY,
                id="runtime.process-count",
                message="runtime process count is 1",
            )
        ]
        runtime_health._powerkit_configured = lambda _root: False
        runtime_health._aibox_runtime_markers = lambda _root: True
        runtime_health._runtime_home_findings = lambda _root: [
            runtime_health.CheckResult(
                severity="INFO",
                category=runtime_health.CATEGORY,
                id="runtime-home.write-ok",
                message="runtime home writable",
            )
        ]

        run_results = runtime_health.run({"repo_root": root})
    finally:
        runtime_health._is_container = restored["_is_container"]
        runtime_health._configured_lnav = restored["_configured_lnav"]
        runtime_health._sqlite_vec_probe = restored["_sqlite_vec_probe"]
        runtime_health._codex_enabled = restored["_codex_enabled"]
        runtime_health._bwrap_smoke = restored["_bwrap_smoke"]
        runtime_health._pid1_finding = restored["_pid1_finding"]
        runtime_health._cgroup_findings = restored["_cgroup_findings"]
        runtime_health._process_counts = restored["_process_counts"]
        runtime_health._powerkit_configured = restored["_powerkit_configured"]
        runtime_health._aibox_runtime_markers = restored["_aibox_runtime_markers"]
        runtime_health._runtime_home_findings = restored["_runtime_home_findings"]
        runtime_health.shutil.which = restored["shutil_which"]

    runtime_ids = {item.id for item in run_results}
    for expected in {
        "runtime.container-detected",
        "runtime.lnav-available",
        "runtime.sqlite-vec-ok",
        "runtime.codex-bwrap-ok",
        "runtime.pid1-ok",
        "runtime.cgroup-memory-current",
        "runtime.process-count",
        "runtime-home.write-ok",
    }:
        check(
            f"19: runtime_health.run emits {expected}",
            expected in runtime_ids,
            [item.to_dict() for item in run_results],
        )

# ---------------------------------------------------------------------------
# Test 20: id_vocabulary — historical collisions are inventory only
# ---------------------------------------------------------------------------
print("\n[20] id_vocabulary — historical lexical collisions are non-actionable")

from checks.id_vocabulary import run as _id_vocabulary_run  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    lib = _REPO_ROOT / "src" / "context" / "skills" / "_lib"
    if str(lib) not in sys.path:
        sys.path.insert(0, str(lib))
    db = root / "context" / ".cache" / "processkit" / "index.sqlite"
    db.parent.mkdir(parents=True)
    conn = sqlite3.connect(db)
    try:
        conn.execute("CREATE TABLE entities (id TEXT)")
        conn.executemany(
            "INSERT INTO entities (id) VALUES (?)",
            [
                ("BACK-20260409_1449-CleanRapidRiver-one",),
                ("DEC-20260409_1450-CleanRapidRiver-two",),
            ],
        )
        conn.commit()
    finally:
        conn.close()

    findings = _id_vocabulary_run({"repo_root": root})
    vocab_ids = {item.id for item in findings}
    check(
        "20: configured kind capacity replaces default-pair warning",
        "id-vocabulary.configured-kind-capacity-ok" in vocab_ids
        and "id-vocabulary.default-pair-capacity-low" not in vocab_ids,
        [item.to_dict() for item in findings],
    )
    ambiguity = [
        item for item in findings
        if item.id == "id-vocabulary.lexical-token-historical-ambiguity"
    ]
    check("20: historical ambiguity emits one finding", len(ambiguity) == 1)
    payload = ambiguity[0].to_dict() if ambiguity else {}
    check(
        "20: historical ambiguity is INFO and non-actionable",
        payload.get("severity") == "INFO"
        and payload.get("action_required") is False
        and payload.get("action_kind") is None,
        json.dumps(payload, indent=2),
    )

# ---------------------------------------------------------------------------
# Test 21: supply_chain — offline lockfile/license/security advisories
# ---------------------------------------------------------------------------
print("\n[21] supply_chain — offline lockfile/license/security advisories")

from checks.supply_chain import run as _supply_chain_run  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    (root / "package.json").write_text(
        textwrap.dedent("""\
            {
                "name": "test-app",
                "private": true
            }
            """),
        encoding="utf-8",
    )
    findings = _supply_chain_run({"repo_root": root, "since_files": None})
    ids = {item.id for item in findings}
    check("21a: missing lockfile emits ERROR", "supply_chain.missing-lockfile" in ids)
    check("21b: missing policy emits WARN", "supply_chain.no-policy" in ids)
    check("21c: inventory summary is emitted", "supply_chain.inventory" in ids)

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    policy = root / ".processkit"
    policy.mkdir()
    (policy / "supply-chain-policy.yaml").write_text(
        textwrap.dedent("""\
            licenses:
              allow:
                - MIT
              review:
                - CDDL-1.0
              deny:
                - AGPL-3.0-only
            """),
        encoding="utf-8",
    )
    core_root = (
        root / "src" / "context" / "skills" /
        "processkit" / "supply-chain-audit" / "scripts"
    )
    core_root.mkdir(parents=True)
    (core_root / "supply_chain_audit.py").write_text(
        textwrap.dedent("""\
            from __future__ import annotations

            def run_audit(repo_root, *_, **__):
                return {
                    "components": [
                        {"name": "blocked-lib", "license": "AGPL-3.0-only"},
                        {"name": "review-lib", "license": "CDDL-1.0"},
                        {"name": "mystery-lib", "license": "UNKNOWN-LICENSE"},
                    ],
                    "security": [
                        {"id": "CVE-2026-9999", "severity": "critical", "dependency": "blocked-lib"},
                        {"id": "CVE-2026-1111", "severity": "medium", "dependency": "review-lib"},
                    ],
                    "skipped_scanners": [
                        {"name": "trivy", "reason": "not-installed"},
                    ],
                    "outdated": [{"name": "blocked-lib", "severity": "high"}],
                    "supplier_quality": [{"supplier": "acme", "risk": "medium"}],
                    "phases": {
                        "phase2": {
                            "scanner_findings": [
                                {"id": "TRVY-0001", "severity": "low", "dependency": "blocked-lib"},
                            ],
                        },
                    },
                }
            """),
        encoding="utf-8",
    )
    (root / "sbom.json").write_text(
        "{}",
        encoding="utf-8",
    )

    core_findings = _supply_chain_run({"repo_root": root, "since_files": None})
    ids = {item.id for item in core_findings}
    check("21d: denied license emits ERROR", "supply_chain.denied-license" in ids)
    check("21e: review license emits WARN", "supply_chain.review-license" in ids)
    check("21f: unknown license emits WARN", "supply_chain.unknown-license" in ids)
    check("21g: skipped scanner is WARN", "supply_chain.scanner-skipped" in ids)
    check("21h: high severity vulnerability emits ERROR", "supply_chain.high-or-critical-vulnerability" in ids)
    check("21i: outdated signal remains advisory", "supply_chain.advisory-outdated" in ids)
    check("21j: supplier quality remains advisory", "supply_chain.advisory-supplier-quality" in ids)
    check("21k: sbom files are reported", "supply_chain.sbom-found" in ids)

# ---------------------------------------------------------------------------
# Test 22: sensitive_data — deterministic findings plus briefing
# ---------------------------------------------------------------------------
print("\n[22] sensitive_data — deterministic findings plus briefing")

from checks import sensitive_data  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    (root / "AGENTS.md").write_text("# test\n", encoding="utf-8")
    (root / "app.env").write_text(
        "\n".join([
            "OPENAI_API_KEY=sk-" + "a" * 24,
            "CONTACT=owner@example.com",
            "CARD=4111 1111 1111 1111",
            "CUSTOMER_FULL_NAME=Alice Example",
        ]),
        encoding="utf-8",
    )
    findings = sensitive_data.run({"repo_root": root, "since_files": None})
    ids = {item.id for item in findings}
    check("21: detects OpenAI-style key", "sensitive-data.openai-key" in ids)
    check("21: detects email address", "sensitive-data.email-address" in ids)
    check("21: detects Luhn-valid credit card", "sensitive-data.credit-card" in ids)
    briefing = [
        item for item in findings
        if item.id == "sensitive-data.probabilistic-briefing"
    ]
    check("21: emits probabilistic briefing", len(briefing) == 1)
    if briefing:
        check(
            "21: briefing records weak-signal files",
            "app.env" in briefing[0].extra.get("weak_signal_files", []),
            json.dumps(briefing[0].extra, indent=2),
        )
        check(
            "21: briefing includes agent search examples",
            bool(briefing[0].extra.get("deterministic_examples"))
            and bool(briefing[0].extra.get("probabilistic_examples")),
            json.dumps(briefing[0].extra, indent=2),
        )
        sample = "\n".join(
            item
            for example in briefing[0].extra.get("deterministic_examples", [])
            for item in example.get("look_for", [])
        )
        url_pattern = next(
            pattern for pattern in sensitive_data._PATTERNS
            if pattern.id == "sensitive-data.url-credential"
        )
        check(
            "21: briefing examples are filtered as URL placeholder examples",
            not list(sensitive_data._matches(
                root / "AGENTS.md", sample, url_pattern
            )),
            sample,
        )

# ---------------------------------------------------------------------------
# Test 23: fresh derived-project doctor regressions
# ---------------------------------------------------------------------------
print("\n[23] pk-doctor — fresh derived project regressions")

from checks import drift as _drift  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    no_script = _drift.run({"repo_root": root})
    check(
        "23: missing release script is not applicable, not an ERROR",
        len(no_script) == 1 and no_script[0].id == "drift.not-applicable"
        and no_script[0].severity == "INFO",
        [item.to_dict() for item in no_script],
    )

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    subprocess.run(["git", "init", "-q", str(root)], check=True)
    (root / ".gitignore").write_text(".aibox/\n", encoding="utf-8")
    (root / ".aibox").mkdir()
    (root / ".aibox" / "auth.env").write_text(
        "CONTACT=private@example.com\n", encoding="utf-8"
    )
    (root / "project.env").write_text(
        "CONTACT=public@example.com\n", encoding="utf-8"
    )
    findings = sensitive_data.run({"repo_root": root, "since_files": None})
    emails = [
        item.extra.get("path") for item in findings
        if item.id == "sensitive-data.email-address"
    ]
    check(
        "23: empty Git index scans non-ignored files but not ignored state",
        emails == ["project.env"], emails,
    )

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    applied = root / "context" / "migrations" / "applied"
    applied.mkdir(parents=True)
    for name, applied_at in (("MIG-new", "2026-07-19T00:00:00Z"),
                             ("MIG-old", "2020-01-01T00:00:00Z")):
        (applied / f"{name}.md").write_text(
            textwrap.dedent(f"""\
                ---
                apiVersion: processkit.projectious.work/v2
                kind: Migration
                metadata:
                  id: {name}
                  created: {applied_at}
                spec:
                  state: applied
                  applied_at: {applied_at}
                ---
                """),
            encoding="utf-8",
        )
    findings = _context_hygiene_run({"repo_root": root})
    archive = [item for item in findings if item.id == "archive.applied-migrations"]
    check(
        "23: only migrations older than the archive policy are candidates",
        len(archive) == 1 and "1 applied migration" in archive[0].message,
        [item.to_dict() for item in archive],
    )

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print()
if failures:
    print(f"FAILED ({len(failures)} test(s)):", ", ".join(failures))
    sys.exit(1)
else:
    print("All tests passed.")
    sys.exit(0)
