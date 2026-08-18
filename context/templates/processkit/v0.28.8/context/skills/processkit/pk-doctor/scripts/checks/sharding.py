"""sharding check.

Two rules:

- Logs under `context/logs/` must live in a `YYYY/MM/` subdir
  (e.g. context/logs/2026/04/LOG-....md). A LogEntry directly under
  context/logs/ is a WARN.
- Migrations under `context/migrations/` must live in their state-bucket
  subdir (pending / in-progress / applied). A Migration directly under
  context/migrations/ or in the wrong bucket (relative to its spec.state)
  is a WARN.

Phase 1: WARN only, no auto-fix. Phase 2 can add a scripted move under `--fix`.
"""

from __future__ import annotations

import re
from pathlib import Path

import yaml

from .common import CheckResult


YYYY_RE = re.compile(r"^\d{4}$")
MM_RE = re.compile(r"^(0[1-9]|1[0-2])$")
VALID_MIG_BUCKETS = {"pending", "in-progress", "applied"}
# aibox-CLI migration docs (e.g. 20260410_1523_0.17.6-to-0.17.9.md) live in
# context/migrations/ but are NOT processkit Migration entities — they are
# CLI upgrade notes. Exempt them from sharding + schema checks.
CLI_MIGRATION_RE = re.compile(r"^\d{8}_\d{4}_\d+\.\d+\.\d+-to-\d+\.\d+\.\d+\.md$")


def _read_workitem_sharding_config(repo_root: Path) -> dict[str, object]:
    path = (
        repo_root / "context" / "skills" / "processkit" /
        "index-management" / "config" / "settings.toml"
    )
    if not path.is_file():
        return {}
    in_section = False
    values: dict[str, object] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line.strip("[]").lower()
            in_section = section in {"sharding.workitem", "sharding.work-item"}
            continue
        if not in_section or "=" not in line:
            continue
        key, value = [part.strip() for part in line.split("=", 1)]
        value = value.strip().strip('"')
        if key == "activate_above_count":
            try:
                values[key] = int(value)
            except ValueError:
                values[key] = value
        else:
            values[key] = value
    return values


def _read_state(path: Path) -> str | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    if not text.startswith("---"):
        return None
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None
    try:
        data = yaml.safe_load(parts[1])
    except yaml.YAMLError:
        return None
    if not isinstance(data, dict):
        return None
    spec = data.get("spec")
    if isinstance(spec, dict):
        state = spec.get("state")
        if isinstance(state, str):
            return state
    return None


def _read_api_version(path: Path) -> str | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    if not text.startswith("---"):
        return None
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None
    try:
        data = yaml.safe_load(parts[1])
    except yaml.YAMLError:
        return None
    if isinstance(data, dict):
        value = data.get("apiVersion")
        return str(value) if value else None
    return None


def _is_v2(path: Path) -> bool:
    return (_read_api_version(path) or "").endswith("/v2")


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    since_files: set[Path] | None = ctx.get("since_files")
    results: list[CheckResult] = []
    ctx_root = repo_root / "context"

    # ------------------ logs: YYYY/MM --------------------
    logs_dir = ctx_root / "logs"
    if logs_dir.is_dir():
        for p in logs_dir.rglob("*.md"):
            if since_files is not None and p not in since_files:
                continue
            if p.name.startswith("INDEX"):
                continue
            rel_parts = p.relative_to(logs_dir).parts
            # Expect [YYYY, MM, <LOG-...>.md]
            if len(rel_parts) < 3 or not YYYY_RE.match(rel_parts[0]) or not MM_RE.match(rel_parts[1]):
                results.append(CheckResult(
                    severity="WARN",
                    category="sharding",
                    id="sharding.log-wrong-bucket",
                    message=(
                        f"{p.relative_to(repo_root)}: LogEntry not under "
                        f"context/logs/YYYY/MM/"
                    ),
                    entity_ref=str(p.relative_to(repo_root)),
                ))

    # ------------------ migrations: state bucket ---------
    mig_dir = ctx_root / "migrations"
    if mig_dir.is_dir():
        for p in mig_dir.rglob("*.md"):
            if since_files is not None and p not in since_files:
                continue
            if p.name.startswith("INDEX"):
                continue
            if CLI_MIGRATION_RE.match(p.name):
                # aibox-CLI upgrade doc, not a processkit Migration entity.
                continue
            rel_parts = p.relative_to(mig_dir).parts
            if len(rel_parts) < 2 or rel_parts[0] not in VALID_MIG_BUCKETS:
                results.append(CheckResult(
                    severity="WARN",
                    category="sharding",
                    id="sharding.migration-state-shard",
                    message=(
                        f"{p.relative_to(repo_root)}: Migration not under "
                        f"pending/ | in-progress/ | applied/"
                    ),
                    entity_ref=str(p.relative_to(repo_root)),
                ))
                continue
            bucket = rel_parts[0]
            state = _read_state(p)
            # Rejected migrations park under applied/ per existing convention.
            if state in ("pending", "in-progress", "applied") and state != bucket:
                results.append(CheckResult(
                    severity="WARN",
                    category="sharding",
                    id="sharding.migration-state-shard",
                    message=(
                        f"{p.relative_to(repo_root)}: spec.state='{state}' "
                        f"but file is in '{bucket}/'"
                    ),
                    entity_ref=str(p.relative_to(repo_root)),
                ))

    # ------------------ v2 notes: YYYY/MM --------------------
    notes_dir = ctx_root / "notes"
    if notes_dir.is_dir():
        for p in notes_dir.rglob("*.md"):
            if since_files is not None and p not in since_files:
                continue
            if p.name.startswith("INDEX") or not _is_v2(p):
                continue
            rel_parts = p.relative_to(notes_dir).parts
            if len(rel_parts) < 3 or not YYYY_RE.match(rel_parts[0]) or not MM_RE.match(rel_parts[1]):
                results.append(CheckResult(
                    severity="ERROR",
                    category="sharding",
                    id="sharding.note-sharded-layout",
                    message=(
                        f"{p.relative_to(repo_root)}: v2 Note not under "
                        f"context/notes/YYYY/MM/"
                    ),
                    entity_ref=str(p.relative_to(repo_root)),
                ))

    # ------------------ v2 terminal WorkItems/Scopes: done/YYYY/MM ---------
    archive_specs = [
        ("workitems", {"done", "cancelled"}, "sharding.workitem-archive-consistent"),
        ("scopes", {"completed", "cancelled"}, "sharding.scope-archive-consistent"),
    ]
    for dirname, terminal_states, finding_id in archive_specs:
        base = ctx_root / dirname
        if not base.is_dir():
            continue
        for p in base.rglob("*.md"):
            if since_files is not None and p not in since_files:
                continue
            if p.name.startswith("INDEX") or not _is_v2(p):
                continue
            state = _read_state(p)
            if state not in terminal_states:
                continue
            rel_parts = p.relative_to(base).parts
            if len(rel_parts) < 4 or rel_parts[0] != "done" or not YYYY_RE.match(rel_parts[1]) or not MM_RE.match(rel_parts[2]):
                results.append(CheckResult(
                    severity="ERROR",
                    category="sharding",
                    id=finding_id,
                    message=(
                        f"{p.relative_to(repo_root)}: terminal v2 entity "
                        f"state '{state}' not under {dirname}/done/YYYY/MM/"
                    ),
                    entity_ref=str(p.relative_to(repo_root)),
                ))

    # ------------------ workitem threshold activation ---------------------
    workitem_cfg = _read_workitem_sharding_config(repo_root)
    threshold = workitem_cfg.get("activate_above_count")
    if threshold is not None:
        if not isinstance(threshold, int) or threshold < 1:
            results.append(CheckResult(
                severity="ERROR",
                category="sharding",
                id="sharding.workitem-shard-threshold",
                message=(
                    "index-management sharding.workitem.activate_above_count "
                    f"must be a positive integer, got {threshold!r}"
                ),
            ))
        elif (ctx_root / "workitems").is_dir():
            pattern = str(
                workitem_cfg.get("pattern") or workitem_cfg.get("scheme") or ""
            )
            flat_live = 0
            sharded_live = 0
            for p in (ctx_root / "workitems").rglob("*.md"):
                if since_files is not None and p not in since_files:
                    continue
                if p.name.startswith("INDEX") or not _is_v2(p):
                    continue
                state = _read_state(p)
                if state in {"done", "cancelled"}:
                    continue
                rel_parts = p.relative_to(ctx_root / "workitems").parts
                if len(rel_parts) == 1:
                    flat_live += 1
                elif (
                    len(rel_parts) >= 3
                    and YYYY_RE.match(rel_parts[0])
                    and MM_RE.match(rel_parts[1])
                ):
                    sharded_live += 1
            if flat_live > threshold and pattern not in {"date", "date-shard"}:
                results.append(CheckResult(
                    severity="ERROR",
                    category="sharding",
                    id="sharding.workitem-shard-threshold",
                    message=(
                        f"flat live WorkItem count {flat_live} exceeds "
                        f"activate_above_count {threshold}, but pattern "
                        f"is {pattern!r}, not date-shard"
                    ),
                ))
            if flat_live > threshold and sharded_live == 0:
                results.append(CheckResult(
                    severity="ERROR",
                    category="sharding",
                    id="sharding.workitem-shard-threshold",
                    message=(
                        f"flat live WorkItem count {flat_live} exceeds "
                        f"activate_above_count {threshold}, but no live "
                        "WorkItems exist under workitems/YYYY/MM/"
                    ),
                ))

    results.append(CheckResult(
        severity="INFO",
        category="sharding",
        id="sharding.checked",
        message="sharding check complete (logs YYYY/MM + migrations state-bucket)",
    ))
    return results
