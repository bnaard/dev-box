"""v1_entity_drift check.

Surfaces v1-frontmatter entity files that sit in directories where a v2
successor pattern exists. Per BACK-20260509_1318-KindSpruce: aibox v0.25.6
ships ~3,450 v1 files in derived projects; the concerning slice is the
72 v1 workitems / 33 v1 decisions / 9 v1 actors / 9 v1 process files
sitting alongside v2 entities with no migration path queued.

Detect logic
------------
For every registered entity directory under context/, walk *.md files,
parse frontmatter, and emit:

- WARN per file with apiVersion `…/v1` whose metadata.kind has a v2
  successor pattern in the SUCCESSOR table. Message names the suggested
  successor so the owner knows where the entity should land.
- INFO (not WARN) for files inside append-only / historical buckets:
  context/logs/ and context/migrations/applied/. Those trees are
  intentionally immutable; emitting WARN there is noise.

Detect-only by default. Under `--fix=v1_entity_drift` (or `--fix-all`)
the check interactively walks each finding and:

- if a v1->v2 Migration entity for that file already exists in the
  pending bucket, records an `apply_migration` fix-intent (same shape
  as `migrations.run_fix` — doctor.py is a plain subprocess so it can
  not call MCP directly).
- otherwise records a `propose_migration` intent. Does NOT auto-create
  migrations — explicit owner action only.

Constraints (per BACK-20260509_1318-KindSpruce):
- The successor table is a hardcoded constant in this module. Promoting
  to a config primitive is a future refactor.
- Module is detect-only; the only mutation hook is the migration
  apply/propose intent record returned by `run_fix`.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Any

import yaml

from .common import CheckResult


# Map kind (as written in entity frontmatter) -> human-readable v2 successor.
# Keep stable; ordering is irrelevant.
SUCCESSOR: dict[str, str] = {
    "Actor": "TeamMember",
    "Process": "Scope + Gate",
    "StateMachine": "lifecycle metadata on the owning entity",
    "Model": "Artifact(kind=model-spec)",
}

# Directories under context/ to walk. The v2 entity directories cover the
# population BACK-20260509_1318-KindSpruce called out (workitems/decisions/
# actors/processes), plus the rest of the v2 entity surface so any future v1
# leak is also surfaced.
ENTITY_DIRS: tuple[str, ...] = (
    "workitems",
    "decisions",
    "notes",
    "artifacts",
    "bindings",
    "actors",
    "team-members",
    "roles",
    "discussions",
    "gates",
    "scopes",
    "processes",
    "state-machines",
    "logs",
    "migrations",
)

# Append-only / historical buckets. v1 entries here are expected and
# downgraded from WARN to INFO. Paths are repo-root-relative posix strings
# matched as a prefix against the file's relative path.
IMMUTABLE_PREFIXES: tuple[str, ...] = (
    "context/logs/",
    "context/migrations/applied/",
)

# aibox-CLI upgrade docs live in context/migrations/ but are not processkit
# entities. Same regex used by sharding.py / migrations.py — keep in sync.
_CLI_MIGRATION_RE = re.compile(
    r"^\d{8}_\d{4}_\d+\.\d+\.\d+-to-\d+\.\d+\.\d+\.md$"
)


def _load_frontmatter(path: Path) -> dict[str, Any] | None:
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
    return data if isinstance(data, dict) else None


def _is_v1(data: dict[str, Any]) -> bool:
    return str(data.get("apiVersion", "")).rstrip("/").endswith("/v1")


def _is_immutable(rel: str) -> bool:
    return any(rel.startswith(prefix) for prefix in IMMUTABLE_PREFIXES)


def _kind_of(data: dict[str, Any]) -> str:
    raw = data.get("kind")
    return str(raw) if raw else ""


def _walk_v1(repo_root: Path) -> list[tuple[Path, str, str]]:
    """Return list of (path, kind, relpath_posix) for v1 entity files."""
    out: list[tuple[Path, str, str]] = []
    ctx_root = repo_root / "context"
    for sub in ENTITY_DIRS:
        base = ctx_root / sub
        if not base.is_dir():
            continue
        for path in sorted(base.rglob("*.md")):
            if path.name.startswith("INDEX"):
                continue
            if _CLI_MIGRATION_RE.match(path.name):
                continue
            data = _load_frontmatter(path)
            if not data or not _is_v1(data):
                continue
            kind = _kind_of(data)
            rel = path.relative_to(repo_root).as_posix()
            out.append((path, kind, rel))
    return out


def _pending_migration_for(repo_root: Path, target_id: str) -> str | None:
    """Return the pending Migration id (if any) that covers target_id.

    Cheap heuristic: any pending Migration whose spec mentions `target_id`
    in `affected_files` / `affected_groups` / `subject` / `targets` is a
    candidate. The owner reviews + confirms in the interactive prompt.
    """
    pending_dirs = [
        repo_root / "context" / "migrations" / "pending",
        repo_root / "context" / "migrations",
    ]
    seen: set[Path] = set()
    for base in pending_dirs:
        if not base.is_dir():
            continue
        for path in sorted(base.glob("*.md")):
            if path in seen or path.name.startswith("INDEX"):
                continue
            if _CLI_MIGRATION_RE.match(path.name):
                continue
            seen.add(path)
            data = _load_frontmatter(path)
            if not data or data.get("kind") != "Migration":
                continue
            spec = data.get("spec") if isinstance(data.get("spec"), dict) else {}
            if str(spec.get("state") or "") not in ("", "pending"):
                continue
            blob = yaml.safe_dump(spec, default_flow_style=False)
            if target_id in blob:
                mid = (
                    data.get("metadata", {}).get("id")
                    if isinstance(data.get("metadata"), dict)
                    else None
                )
                if mid:
                    return str(mid)
    return None


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    since_files: set[Path] | None = ctx.get("since_files")
    results: list[CheckResult] = []

    warned = 0
    info_immutable = 0
    info_no_successor = 0

    for path, kind, rel in _walk_v1(repo_root):
        if since_files is not None and path not in since_files:
            continue

        # Append-only buckets: INFO regardless of kind.
        if _is_immutable(rel):
            info_immutable += 1
            continue

        successor = SUCCESSOR.get(kind)
        if not successor:
            # v1 file in a v2 entity dir but kind has no documented
            # successor (e.g. plain v1 WorkItem still in the v2 workitems/
            # tree). Same WARN pattern; suggest owning team review.
            warned += 1
            results.append(CheckResult(
                severity="WARN",
                category="v1_entity_drift",
                id="v1.entity-still-v1",
                message=(
                    f"{rel}: apiVersion v1 entity (kind={kind!r}) sits in "
                    "a v2 entity directory; queue a v1->v2 migration"
                ),
                entity_ref=rel,
                fixable=True,
                suggested_fix=(
                    "review and apply (or propose) a v1->v2 Migration via "
                    "migration-management MCP"
                ),
                fix_mcp_tool=(
                    "mcp__processkit-migration-management__apply_migration"
                ),
            ))
            continue

        warned += 1
        results.append(CheckResult(
            severity="WARN",
            category="v1_entity_drift",
            id="v1.entity-superseded",
            message=(
                f"{rel}: v1 {kind} is superseded by v2 successor "
                f"{successor!r}; queue a v1->v2 migration"
            ),
            entity_ref=rel,
            fixable=True,
            suggested_fix=(
                f"migrate to {successor} via migration-management MCP "
                "(do not hand-edit)"
            ),
            fix_mcp_tool=(
                "mcp__processkit-migration-management__apply_migration"
            ),
        ))

    if info_immutable:
        results.append(CheckResult(
            severity="INFO",
            category="v1_entity_drift",
            id="v1.immutable-bucket",
            message=(
                f"{info_immutable} v1 entity file(s) in append-only "
                "buckets (logs/, migrations/applied/) — historical, "
                "intentionally not migrated"
            ),
        ))

    if not warned and not info_immutable and not info_no_successor:
        results.append(CheckResult(
            severity="INFO",
            category="v1_entity_drift",
            id="v1.entity-drift-clean",
            message="no v1 entities in v2 entity directories",
        ))

    return results


def run_fix(ctx, results: list[CheckResult]) -> list[dict]:
    """Interactive walk: per WARN finding, record apply or propose intent.

    Phase 1: emit FIX-INTENT records — doctor.py is a plain subprocess
    without an MCP client, so the calling agent (or the user) executes
    the actual MCP call. Mirrors the pattern used by `migrations.run_fix`.
    """
    interactive = ctx.get("interactive", False)
    auto_yes = ctx.get("yes", False)
    repo_root: Path = ctx["repo_root"]
    fixes: list[dict] = []

    if not interactive and not auto_yes:
        fixes.append({
            "category": "v1_entity_drift",
            "status": "skipped",
            "reason": (
                "fix requires interactive prompt; re-run from terminal "
                "or pass --yes"
            ),
        })
        return fixes

    findings = [r for r in results if r.severity == "WARN" and r.entity_ref]
    for r in findings:
        target_id = (Path(r.entity_ref).stem if r.entity_ref else "") or ""
        existing = _pending_migration_for(repo_root, target_id)

        if auto_yes:
            choice = "y"
        else:
            try:
                prompt = (
                    f"  Migrate {target_id}? "
                    f"[y(apply existing MIG-?)/p(ropose new)/N/s(kip)] "
                    if existing
                    else f"  Migrate {target_id}? [p(ropose new)/N/s(kip)] "
                )
                raw = input(prompt).strip().lower()
            except (EOFError, KeyboardInterrupt):
                print(file=sys.stderr)
                fixes.append({
                    "category": "v1_entity_drift",
                    "entity": target_id,
                    "status": "aborted",
                })
                break
            choice = raw or "n"

        if choice == "y" and existing:
            fixes.append({
                "category": "v1_entity_drift",
                "entity": target_id,
                "status": "intent-recorded",
                "mcp_tool": (
                    "mcp__processkit-migration-management__apply_migration"
                ),
                "mcp_args": {"id": existing},
                "note": (
                    f"existing pending Migration {existing} appears to "
                    f"cover {target_id}; owner confirms and applies via "
                    "migration-management MCP"
                ),
            })
        elif choice == "p" or (choice == "y" and not existing):
            fixes.append({
                "category": "v1_entity_drift",
                "entity": target_id,
                "status": "propose-only",
                "mcp_tool": (
                    "mcp__processkit-migration-management__start_migration"
                ),
                "note": (
                    f"no pending Migration found for {target_id}; "
                    "propose a v1->v2 Migration via the migration-management "
                    "MCP. doctor does NOT auto-create."
                ),
            })
        elif choice == "s":
            fixes.append({
                "category": "v1_entity_drift",
                "entity": target_id,
                "status": "skipped",
            })
        else:
            fixes.append({
                "category": "v1_entity_drift",
                "entity": target_id,
                "status": "declined",
            })

    return fixes
