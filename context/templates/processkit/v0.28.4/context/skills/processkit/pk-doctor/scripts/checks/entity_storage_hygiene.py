"""Entity storage layout and lifecycle policy checks.

This check is intentionally separate from template drift and schema
validation. It answers: does the local ``context/`` tree follow the
current processkit storage policy, and are legacy layouts explained?
"""

from __future__ import annotations

import re
import shutil
from pathlib import Path
from typing import Any

import yaml

from .common import CheckResult


_FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---", re.S)
_HOST_ARTIFACTS = {".DS_Store", "Thumbs.db", "desktop.ini"}
_WORDPAIR_RE = re.compile(r"^[A-Z][a-z]+[A-Z][A-Za-z]+$")
_CLI_MIGRATION_RE = re.compile(
    r"^\d{8}_\d{4}_\d+\.\d+\.\d+-to-\d+\.\d+\.\d+\.md$"
)
_LEGACY_RUNTIME_MIGRATION_RE = re.compile(
    r"^MIG-RUNTIME(?:-DRIFT)?-\d{8}T\d{6}$"
)
_CLI_MIGRATION_COMPLETED_RE = re.compile(
    r"^\*\*Status:\*\*\s*completed\b",
    re.IGNORECASE | re.MULTILINE,
)
_BRIEFING_ARCHIVE_SUBDIR = "context/archive/cli-migration-briefings"

_ENTITY_DIRS = {
    "artifacts": "ART",
    "bindings": "BIND",
    "decisions": "DEC",
    "discussions": "DISC",
    "gates": "GATE",
    "migrations": "MIG",
    "notes": "NOTE",
    "roles": "ROLE",
    "scopes": "SCOPE",
    "team-members": "TEAMMEMBER",
    "workitems": "BACK",
}

_EXPECTED_KIND_BY_DIR = {
    "artifacts": "Artifact",
    "bindings": "Binding",
    "decisions": "DecisionRecord",
    "discussions": "Discussion",
    "gates": "Gate",
    "migrations": "Migration",
    "notes": "Note",
    "roles": "Role",
    "scopes": "Scope",
    "team-members": "TeamMember",
    "workitems": "WorkItem",
}

_DATE_SHARDED_KINDS = {
    "notes": "context/notes/YYYY/MM/",
    "workitems": "context/workitems/YYYY/MM/ for live work; "
    "context/workitems/done/YYYY/MM/ for terminal work",
}

_STATE_BUCKET_KINDS = {
    "migrations": "context/migrations/{pending,in-progress,applied}/",
}

_FLAT_BY_POLICY = {
    "artifacts",
    "bindings",
    "decisions",
    "discussions",
    "gates",
    "roles",
}


def _rel(repo_root: Path, path: Path) -> str:
    try:
        return str(path.relative_to(repo_root))
    except ValueError:
        return str(path)


def _load_frontmatter(path: Path) -> dict[str, Any] | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    match = _FRONTMATTER_RE.match(text)
    if not match:
        return None
    try:
        data = yaml.safe_load(match.group(1))
    except yaml.YAMLError:
        return None
    return data if isinstance(data, dict) else None


def _is_cli_migration_completed(path: Path) -> bool:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return False
    return _CLI_MIGRATION_COMPLETED_RE.search(text) is not None


def _cli_migration_briefings(repo_root: Path) -> list[Path]:
    root = repo_root / "context" / "migrations"
    briefings = [
        p for p in _root_markdown_files(root)
        if _CLI_MIGRATION_RE.match(p.name)
        and not _is_cli_migration_completed(p)
    ]
    return sorted(briefings)


def _briefing_archive_root(repo_root: Path) -> Path:
    return repo_root / _BRIEFING_ARCHIVE_SUBDIR


def _allocate_archive_path(dst_root: Path, source: Path) -> Path:
    target = dst_root / source.name
    if not target.exists():
        return target
    stem = source.stem
    suffix = source.suffix
    for i in range(1, 1000):
        target = dst_root / f"{stem}-archived-{i:03d}{suffix}"
        if not target.exists():
            return target
    raise RuntimeError(
        f"archive path unavailable for {source.name} under {dst_root}"
    )


def _is_v2(path: Path) -> bool:
    fm = _load_frontmatter(path) or {}
    return str(fm.get("apiVersion") or "").endswith("/v2")


def _kind(path: Path) -> str:
    fm = _load_frontmatter(path) or {}
    return str(fm.get("kind") or "")


def _binding_type(path: Path) -> str:
    fm = _load_frontmatter(path) or {}
    spec = fm.get("spec") if isinstance(fm.get("spec"), dict) else {}
    return str(spec.get("type") or "")


def _datetime_token(path: Path, prefix: str) -> str:
    if not path.stem.startswith(prefix + "-"):
        return ""
    token = path.stem[len(prefix) + 1:].split("-", 1)[0]
    return token if re.fullmatch(r"\d{8}_\d{4}", token) else ""


def _workitem_shard_threshold(repo_root: Path) -> int | None:
    path = (
        repo_root / "context" / "skills" / "processkit" /
        "index-management" / "config" / "settings.toml"
    )
    if not path.is_file():
        return None
    in_section = False
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            in_section = line.strip("[]").lower() in {
                "sharding.workitem",
                "sharding.work-item",
            }
            continue
        if not in_section or "=" not in line:
            continue
        key, value = [part.strip() for part in line.split("=", 1)]
        if key != "activate_above_count":
            continue
        try:
            return int(value.strip().strip('"'))
        except ValueError:
            return None
    return None


def _root_markdown_files(base: Path) -> list[Path]:
    if not base.is_dir():
        return []
    return sorted(
        p for p in base.glob("*.md")
        if p.is_file() and not p.name.startswith("INDEX")
    )


def _nested_markdown_files(base: Path) -> list[Path]:
    if not base.is_dir():
        return []
    return sorted(
        p for p in base.rglob("*.md")
        if p.is_file()
        and not p.name.startswith("INDEX")
        and p.parent != base
    )


def _sample(paths: list[Path], repo_root: Path, limit: int = 5) -> list[str]:
    return [_rel(repo_root, p) for p in paths[:limit]]


def _filename_style(stem: str, prefix: str) -> str:
    if prefix == "MIG" and _LEGACY_RUNTIME_MIGRATION_RE.match(stem):
        return "legacy_runtime_migration"
    if not stem.startswith(prefix + "-"):
        return "other"
    body = stem[len(prefix) + 1:]
    if re.search(r"\d{8}_0000", body):
        return "placeholder_time"
    parts = body.split("-", 2)
    if len(parts) >= 2 and re.fullmatch(r"\d{8}_\d{4}", parts[0]):
        if _WORDPAIR_RE.match(parts[1]):
            return "datetime_wordpair"
    return "other"


def _host_artifacts(repo_root: Path) -> list[CheckResult]:
    context = repo_root / "context"
    if not context.is_dir():
        return []
    found = sorted(
        p for p in context.rglob("*")
        if p.is_file() and p.name in _HOST_ARTIFACTS
    )
    if not found:
        return []
    return [CheckResult(
        severity="WARN",
        category="entity_storage_hygiene",
        id="storage.host-artifact",
        message=(
            f"{len(found)} unmanaged host artifact(s) exist under context/"
        ),
        suggested_fix="delete host artifacts such as .DS_Store from context/",
        extra={"sample": _sample(found, repo_root)},
    )]


def _legacy_locations(repo_root: Path) -> list[CheckResult]:
    results: list[CheckResult] = []
    models = repo_root / "context" / "models"
    if models.is_dir():
        model_files = sorted(models.glob("MODEL-*.md"))
        results.append(CheckResult(
            severity="WARN",
            category="entity_storage_hygiene",
            id="storage.demoted-model-root",
            message=(
                f"context/models contains {len(model_files)} legacy Model "
                "file(s); model specs/profiles are now Artifact entities"
            ),
            suggested_fix=(
                "migrate remaining MODEL-* files to Artifact(kind=model-spec) "
                "or archive the legacy root"
            ),
            extra={"sample": _sample(model_files, repo_root)},
        ))

    archive = repo_root / "context" / "archive"
    if archive.is_dir():
        legacy_dirs = sorted(
            p for p in archive.iterdir()
            if p.is_dir() and p.name.endswith("-v1")
        )
        results.append(CheckResult(
            severity="INFO",
            category="entity_storage_hygiene",
            id="storage.legacy-archive-policy",
            message=(
                "context/archive is a legacy read-only archive root; "
                "*-v1 subdirectories are permitted for superseded v1 "
                "snapshots but should not receive new active entities"
            ),
            extra={"legacy_dirs": [_rel(repo_root, p) for p in legacy_dirs]},
        ))
    return results


def _migration_briefings(repo_root: Path) -> list[CheckResult]:
    briefings = _cli_migration_briefings(repo_root)
    if not briefings:
        return []
    return [CheckResult(
        severity="INFO",
        category="entity_storage_hygiene",
        id="storage.root-migration-briefings",
        message=(
            f"{len(briefings)} root-level migration briefing file(s) exist; "
            "they are historical CLI upgrade notes, not Migration entities"
        ),
        suggested_fix=(
            "archive old CLI migration briefings or move them outside the "
            "Migration entity lifecycle tree"
        ),
        fix_mcp_tool="run_pk_doctor",
        extra={
            "sample": _sample(briefings, repo_root),
            "fix_mcp_args": {
                "check": "entity_storage_hygiene",
                "fix": "entity_storage_hygiene",
                "yes": True,
            },
        },
    )]


def _layout_checks(repo_root: Path) -> list[CheckResult]:
    results: list[CheckResult] = []
    context = repo_root / "context"
    for dirname, prefix in _ENTITY_DIRS.items():
        base = context / dirname
        if not base.is_dir():
            continue
        root_files = _root_markdown_files(base)
        nested_files = _nested_markdown_files(base)
        expected_kind = _EXPECTED_KIND_BY_DIR.get(dirname)
        root_v2 = [
            p for p in root_files
            if _is_v2(p) and (not expected_kind or _kind(p) == expected_kind)
        ]
        nested_v2 = [
            p for p in nested_files
            if _is_v2(p) and (not expected_kind or _kind(p) == expected_kind)
        ]

        if dirname in _DATE_SHARDED_KINDS and root_v2 and nested_v2:
            root_tokens = [
                token for token in (_datetime_token(p, prefix) for p in root_v2)
                if token
            ]
            nested_tokens = [
                token for token in (_datetime_token(p, prefix) for p in nested_v2)
                if token
            ]
            results.append(CheckResult(
                severity="WARN",
                category="entity_storage_hygiene",
                id="storage.mixed-layout",
                message=(
                    f"context/{dirname} mixes root v2 files "
                    f"({len(root_v2)}) and nested v2 files "
                    f"({len(nested_v2)}); canonical layout is "
                    f"{_DATE_SHARDED_KINDS[dirname]}"
                ),
                suggested_fix=(
                    "create and apply a data-fix Migration that moves "
                    "active root files into the canonical sharded layout"
                ),
                extra={
                    "sample": _sample(root_v2, repo_root),
                    "root_token_max": max(root_tokens) if root_tokens else None,
                    "nested_token_min": (
                        min(nested_tokens) if nested_tokens else None
                    ),
                    "activate_above_count": _workitem_shard_threshold(repo_root),
                },
            ))
        elif dirname in _STATE_BUCKET_KINDS and root_v2:
            results.append(CheckResult(
                severity="WARN",
                category="entity_storage_hygiene",
                id="storage.state-bucket-layout",
                message=(
                    f"context/{dirname} has {len(root_v2)} root v2 "
                    f"Migration file(s); canonical layout is "
                    f"{_STATE_BUCKET_KINDS[dirname]}"
                ),
                suggested_fix=(
                    "move Migration entities through migration-management"
                ),
                extra={"sample": _sample(root_v2, repo_root)},
            ))
        elif dirname in _FLAT_BY_POLICY and nested_v2:
            results.append(CheckResult(
                severity="WARN",
                category="entity_storage_hygiene",
                id="storage.unexpected-nested-layout",
                message=(
                    f"context/{dirname} is currently flat-by-policy but "
                    f"contains {len(nested_v2)} nested v2 file(s)"
                ),
                suggested_fix=(
                    "move files back to the flat root or add an explicit "
                    "sharding/archive policy for this entity kind"
                ),
                extra={"sample": _sample(nested_v2, repo_root)},
            ))

        styles: dict[str, list[Path]] = {
            "datetime_wordpair": [],
            "legacy_runtime_migration": [],
            "placeholder_time": [],
            "other": [],
        }
        for path in root_files + nested_files:
            if dirname == "migrations" and _CLI_MIGRATION_RE.match(path.name):
                continue
            style = _filename_style(path.stem, prefix)
            if (
                dirname == "bindings"
                and style == "datetime_wordpair"
                and _binding_type(path) == "role-slot-fill"
            ):
                continue
            styles.setdefault(style, []).append(path)
        if styles["placeholder_time"]:
            results.append(CheckResult(
                severity="WARN",
                category="entity_storage_hygiene",
                id="storage.placeholder-timestamp",
                message=(
                    f"context/{dirname} contains "
                    f"{len(styles['placeholder_time'])} filename(s) with "
                    "placeholder timestamp 0000"
                ),
                suggested_fix=(
                    "create and apply a data-fix Migration that rewrites "
                    "placeholder IDs to canonical IDs and updates references"
                ),
                extra={
                    "sample": _sample(styles["placeholder_time"], repo_root),
                },
            ))
        if styles["legacy_runtime_migration"]:
            results.append(CheckResult(
                severity="INFO",
                category="entity_storage_hygiene",
                id="storage.legacy-runtime-migration-filenames",
                message=(
                    f"context/{dirname} contains "
                    f"{len(styles['legacy_runtime_migration'])} legacy "
                    "runtime Migration filename(s); these are tolerated "
                    "historical runtime-producer IDs and should not be "
                    "renamed outside an explicit history-rewrite migration"
                ),
                extra={
                    "sample": _sample(
                        styles["legacy_runtime_migration"], repo_root
                    ),
                },
            ))
        if styles["datetime_wordpair"] and styles["other"]:
            results.append(CheckResult(
                severity="WARN",
                category="entity_storage_hygiene",
                id="storage.filename-policy-mixed",
                message=(
                    f"context/{dirname} mixes datetime-wordpair filenames "
                    f"({len(styles['datetime_wordpair'])}) with other "
                    f"filename policies ({len(styles['other'])})"
                ),
                suggested_fix=(
                    "use migration-management.normalize_migration_filename "
                    "with a canonical MIG-YYYYMMDD_HHMM-WordPair target ID; "
                    "it updates mutable references and preserves append-only "
                    "historical records"
                ),
                extra={"sample": _sample(styles["other"], repo_root)},
            ))
    return results


def _team_member_policy(repo_root: Path) -> list[CheckResult]:
    root = repo_root / "context" / "team-members"
    if not root.is_dir():
        return []
    private_dirs = sorted(
        p for p in root.glob("*/private")
        if p.is_dir()
    )
    results: list[CheckResult] = []
    if private_dirs:
        results.append(CheckResult(
            severity="INFO",
            category="entity_storage_hygiene",
            id="storage.team-member-private-policy",
            message=(
                "team-member private/ directories are developer-local, "
                "gitignored memory/state and are not canonical entity data"
            ),
            extra={"sample": _sample(private_dirs, repo_root)},
        ))

    human_slugs: list[str] = []
    for entity_file in sorted(root.glob("*/team-member.md")):
        fm = _load_frontmatter(entity_file) or {}
        spec = fm.get("spec") if isinstance(fm.get("spec"), dict) else {}
        if spec.get("type") == "human":
            slug = str(spec.get("slug") or entity_file.parent.name)
            if "-" in slug:
                human_slugs.append(slug)
    if human_slugs:
        results.append(CheckResult(
            severity="INFO",
            category="entity_storage_hygiene",
            id="storage.human-team-member-slug-policy",
            message=(
                "human TeamMember slugs may be privacy-preserving aliases; "
                "AI-agent TeamMembers may use human-readable pool names"
            ),
            extra={"human_slugs": human_slugs[:5]},
        ))
    return results


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    results: list[CheckResult] = []
    results.extend(_host_artifacts(repo_root))
    results.extend(_legacy_locations(repo_root))
    results.extend(_migration_briefings(repo_root))
    results.extend(_layout_checks(repo_root))
    results.extend(_team_member_policy(repo_root))
    results.append(CheckResult(
        severity="INFO",
        category="entity_storage_hygiene",
        id="storage.policy-summary",
        message=(
            "entity storage policy checked: logs/notes/workitems use "
            "date/state-aware layouts where configured; decisions, "
            "artifacts, roles, bindings, discussions, and gates remain "
            "flat-by-policy until a dedicated archive migration exists"
        ),
    ))
    return results


def run_fix(ctx, results: list[CheckResult]) -> list[dict]:
    """Move non-completed CLI migration briefing files to archive.

    They are operational upgrade notes, not processkit entity files.
    Archiving keeps historical content while removing lifecycle noise.
    """
    interactive = ctx.get("interactive", False)
    auto_yes = ctx.get("yes", False)
    repo_root: Path = ctx["repo_root"]
    fixes: list[dict] = []

    if not interactive and not auto_yes:
        fixes.append({
            "category": "entity_storage_hygiene",
            "status": "skipped",
            "reason": (
                "fix requires interactive prompt; re-run from terminal "
                "or pass --yes"
            ),
        })
        return fixes

    briefings = _cli_migration_briefings(repo_root)

    if not briefings:
        fixes.append({
            "category": "entity_storage_hygiene",
            "status": "skipped",
            "reason": "no uncompleted migration briefings found",
        })
        return fixes

    archive_root = _briefing_archive_root(repo_root)
    archive_root.mkdir(parents=True, exist_ok=True)

    for source in briefings:
        if not source.is_file():
            fixes.append({
                "category": "entity_storage_hygiene",
                "entity": str(source.relative_to(repo_root)),
                "status": "skipped",
                "reason": "source not found",
            })
            continue
        if not _CLI_MIGRATION_RE.match(source.name):
            continue
        if _is_cli_migration_completed(source):
            fixes.append({
                "category": "entity_storage_hygiene",
                "entity": str(source.relative_to(repo_root)),
                "status": "skipped",
                "reason": "already completed",
            })
            continue
        try:
            target = _allocate_archive_path(archive_root, source)
            shutil.move(str(source), str(target))
        except (OSError, RuntimeError) as exc:
            fixes.append({
                "category": "entity_storage_hygiene",
                "entity": str(source.relative_to(repo_root)),
                "status": "failed",
                "reason": str(exc),
            })
            continue
        fixes.append({
            "category": "entity_storage_hygiene",
            "entity": str(source.relative_to(repo_root)),
            "status": "archived",
            "archived_to": str(_rel(repo_root, target)),
            "rationale": (
                "moved non-entity CLI migration briefing out of migration "
                "lifecycle root"
            ),
        })

    return fixes
