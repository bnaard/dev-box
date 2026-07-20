"""schema_filename check.

For every `context/<kind>/**/*.md`:

- Load the matching schema from src/context/schemas/<kind>.yaml (LIVE source,
  NOT the template mirror under context/templates/).
- Parse the file's YAML frontmatter.
- Validate spec against schema.spec_schema.
- Confirm the filename stem equals metadata.id.
- Confirm any date encoded in the filename matches metadata.created.

Phase 1 is WARN-only for rename suggestions — no auto-fix. Auto-rename +
reindex is deferred to Phase 2.

Known parser quirks:
- YAML datetime fields vs JSON-Schema `string` format=date-time coercion
  mismatches surface as WARN schema.datetime-coercion (owner policy).
"""

from __future__ import annotations

from datetime import datetime, timezone
import re
from pathlib import Path
from random import choices
from typing import Iterator

import yaml  # via uv inline deps in doctor.py

from .common import CheckResult


# Maps schema filename stem → directory under context/ where that kind lives.
# Only kinds with a `default_directory` under context/<dir>/ get walked.
KIND_TO_DIR = {
    "workitem": "workitems",
    "decisionrecord": "decisions",
    "artifact": "artifacts",
    "binding": "bindings",
    "actor": "actors",
    "role": "roles",
    "discussion": "discussions",
    "gate": "gates",
    "migration": "migrations",
    "note": "notes",
    "process": "processes",
    "scope": "scopes",
    "logentry": "logs",
}

FILENAME_DATE_RE = re.compile(r"(\d{8})(?:[_T](\d{4,6}))?")
# aibox-CLI migration docs (e.g. 20260410_1523_0.17.6-to-0.17.9.md) live in
# context/migrations/ but are NOT processkit Migration entities — they are
# CLI upgrade notes. Exempt them from schema validation.
CLI_MIGRATION_RE = re.compile(r"^\d{8}_\d{4}_\d+\.\d+\.\d+-to-\d+\.\d+\.\d+\.md$")

# Actor ID pattern enforcement (DEC-20260421_2036-SoundIvy-two-class-actor-ids)
_ACTOR_IDENTITY_RE = re.compile(
    r"^ACTOR-\d{8}_\d{4}-[A-Z][a-z]+[A-Z][a-z]+-[a-z0-9-]+$"
)
_ACTOR_ROLE_RE = re.compile(r"^ACTOR-[a-z][a-z0-9-]*$")
_MISSING_ACTOR_MARKER = "'actor' is a required property"
_CORRECTION_EVENT_TYPE = "logentry.corrected"
_CORRECTION_GENERIC_REASON = "_schema_filename_all"


def _check_actor_id_pattern(
    entity_id: str, allowed_role_ids: list[str]
) -> str | None:
    """Return None if valid, or an error reason string if invalid."""
    if _ACTOR_IDENTITY_RE.match(entity_id):
        return None  # identity class — OK
    if _ACTOR_ROLE_RE.match(entity_id):
        slug = entity_id[len("ACTOR-"):]
        if slug in allowed_role_ids:
            return None  # role class in allowlist — OK
        return (
            f"role-actor ID '{slug}' not in spec.role_actor_ids allowlist"
        )
    return (
        "does not match identity class "
        "(ACTOR-<datetime>-<WordPair>-<slug>) or "
        "role class (ACTOR-<slug>)"
    )


def _resolve_schemas_dir(repo_root: Path) -> Path | None:
    """Return the schemas directory for ``repo_root`` or ``None``.

    Checks two layouts in order:
      1. ``<repo_root>/src/context/schemas/`` — processkit dogfood (the
         processkit repo itself; schemas live in src/ because they ship
         to consumers).
      2. ``<repo_root>/context/schemas/`` — derived projects (aibox-
         installed; schemas were copied directly into context/).

    A derived-project install never has a ``src/`` tree, so the first
    candidate misses and the function falls through to the second.
    Returning ``None`` is the cue for the caller to surface a single
    "no schemas dir found" WARN rather than silently walking 0 files.
    """
    for candidate in (
        repo_root / "src" / "context" / "schemas",
        repo_root / "context" / "schemas",
    ):
        if candidate.is_dir():
            return candidate
    return None


def _load_schema(schemas_dir: Path, kind: str) -> dict | None:
    candidates = (
        schemas_dir / "_generated" / f"{kind}.yaml",
        schemas_dir / f"{kind}.yaml",
    )
    path = next((item for item in candidates if item.is_file()), None)
    if path is None:
        return None
    try:
        with path.open("r", encoding="utf-8") as f:
            return yaml.safe_load(f)
    except yaml.YAMLError:
        return None


def _parse_frontmatter(path: Path) -> tuple[dict | None, str | None]:
    """Return (frontmatter_dict, parse_error_msg_or_None)."""
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as e:
        return None, f"unreadable: {e}"
    if not text.startswith("---"):
        return None, "no YAML frontmatter"
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None, "unterminated YAML frontmatter"
    try:
        data = yaml.safe_load(parts[1])
    except yaml.YAMLError as e:
        return None, f"YAML parse error: {e}"
    if not isinstance(data, dict):
        return None, "frontmatter is not a mapping"
    return data, None


def _to_isodate(ts: datetime) -> str:
    return ts.replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _safe_slug() -> str:
    return "".join(choices("abcdefghijklmnopqrstuvwxyz0123456789", k=8))


def _iter_log_corrections(repo_root: Path) -> list[Path]:
    log_root = repo_root / "context" / "logs"
    if not log_root.is_dir():
        return []
    return list(log_root.rglob("*.md"))


def _extract_correction_targets(details: dict[str, object]) -> set[str]:
    targets: set[str] = set()
    raw = details.get("corrects")
    if isinstance(raw, str):
        targets.add(raw)
    elif isinstance(raw, dict):
        if isinstance(raw.get("id"), str):
            targets.add(raw.get("id"))
    elif isinstance(raw, list):
        for item in raw:
            if isinstance(item, str):
                targets.add(item)
            elif isinstance(item, dict) and isinstance(item.get("id"), str):
                targets.add(item.get("id"))
    return targets


def _extract_correction_issue_ids(details: dict[str, object]) -> set[str]:
    issues: set[str] = set()
    raw = details.get("issues")
    if isinstance(raw, str):
        issues.add(raw)
    elif isinstance(raw, list):
        for item in raw:
            if isinstance(item, str):
                issues.add(item)
            elif isinstance(item, dict) and isinstance(item.get("id"), str):
                issues.add(item.get("id"))
    return issues or {_CORRECTION_GENERIC_REASON}


def _load_log_correction_map(repo_root: Path) -> dict[str, set[str]]:
    out: dict[str, set[str]] = {}
    for p in _iter_log_corrections(repo_root):
        fm, perr = _parse_frontmatter(p)
        if fm is None:
            continue
        spec = fm.get("spec") if isinstance(fm, dict) else None
        if not isinstance(spec, dict):
            continue
        if spec.get("event_type") != _CORRECTION_EVENT_TYPE:
            continue
        details = spec.get("details")
        if not isinstance(details, dict):
            continue
        targets = _extract_correction_targets(details)
        if not targets:
            continue
        issue_ids = _extract_correction_issue_ids(details)
        for target_id in targets:
            out.setdefault(target_id, set()).update(issue_ids)
    return out


def _already_corrected(
    correction_map: dict[str, set[str]],
    entity_id: str,
    finding_id: str,
) -> bool:
    by_entity = correction_map.get(entity_id)
    if by_entity is None:
        return False
    return finding_id in by_entity or _CORRECTION_GENERIC_REASON in by_entity


def _iter_entity_files(ctx_root: Path, kind_dir: str, since_files: set[Path] | None) -> Iterator[Path]:
    root = ctx_root / kind_dir
    if not root.is_dir():
        return
    for p in root.rglob("*.md"):
        if p.name.startswith("INDEX"):
            continue
        if CLI_MIGRATION_RE.match(p.name):
            # aibox-CLI upgrade doc, not a processkit entity — skip.
            continue
        if since_files is not None and p not in since_files:
            continue
        yield p


def _validate_against_schema(data: dict, schema: dict) -> list[str]:
    """Return list of JSON-schema error messages. Empty → valid.

    Uses jsonschema if available (via uv inline deps). If missing, returns
    a single synthetic error so the operator knows validation was skipped.
    """
    spec = data.get("spec")
    schema_spec = schema.get("spec", {}).get("spec_schema")
    if not schema_spec or not isinstance(spec, dict):
        return []  # nothing enforceable — surface other checks still fire
    try:
        import jsonschema
        from jsonschema import Draft202012Validator
    except ImportError:
        return ["jsonschema not installed (run via uv)"]

    # Coerce YAML datetime objects to ISO strings for JSON-Schema.
    # Record a marker so callers can downgrade the severity for coercion
    # failures per owner policy.
    def _coerce(o):
        if isinstance(o, dict):
            return {k: _coerce(v) for k, v in o.items()}
        if isinstance(o, list):
            return [_coerce(v) for v in o]
        # Handle datetime / date objects without importing datetime globally
        if hasattr(o, "isoformat"):
            return o.isoformat()
        return o

    coerced = _coerce(spec)
    validator = Draft202012Validator(schema_spec)
    errs = []
    for e in validator.iter_errors(coerced):
        path = ".".join(str(x) for x in e.absolute_path) or "<root>"
        errs.append(f"{path}: {e.message}")
    return errs


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    since_files: set[Path] | None = ctx.get("since_files")
    results: list[CheckResult] = []
    correction_map = _load_log_correction_map(repo_root)

    ctx_root = repo_root / "context"
    schemas_dir = _resolve_schemas_dir(repo_root)
    if schemas_dir is None:
        # No schemas anywhere — surface this loudly instead of silently
        # walking 0 files, which masked entity-hygiene problems in
        # derived projects (HappyReef).  Walk continues with no schema
        # validation but still catches structural issues (filename/id,
        # actor-id pattern) below.
        results.append(CheckResult(
            severity="WARN",
            category="schema_filename",
            id="schema.no-schemas-dir",
            message=(
                "no schemas directory found at "
                f"{repo_root / 'src' / 'context' / 'schemas'} or "
                f"{repo_root / 'context' / 'schemas'} — schema validation "
                "is disabled for this run; only filename/id and actor-id "
                "checks fire"
            ),
        ))

    walked_count = 0
    for kind, kind_dir in KIND_TO_DIR.items():
        schema = _load_schema(schemas_dir, kind) if schemas_dir else None
        # Even without a schema we still want to walk the entity files
        # so the filename/id, filename-date, and actor-id checks fire
        # (those don't need spec_schema). Skip only the schema-validation
        # block when schema is None.
        if schemas_dir is None and not (repo_root / "context" / kind_dir).is_dir():
            # No schemas AND no entity dir for this kind — skip silently.
            continue
        # Cache allowed role IDs per kind (actor only, cheap to compute).
        allowed_role_ids: list[str] = []
        if schema:
            schema_spec = schema.get("spec", {})
            allowed_role_ids = (
                schema_spec.get("role_actor_ids")
                or schema.get("x-allowed-role-ids", [])
            )
        for path in _iter_entity_files(ctx_root, kind_dir, since_files):
            walked_count += 1
            fm, parse_err = _parse_frontmatter(path)
            rel = path.relative_to(repo_root)
            if parse_err:
                results.append(CheckResult(
                    severity="ERROR",
                    category="schema_filename",
                    id="schema.parse-error",
                    message=f"{rel}: {parse_err}",
                    entity_ref=str(rel),
                ))
                continue

            md = fm.get("metadata", {}) if isinstance(fm, dict) else {}
            meta_id = md.get("id") if isinstance(md, dict) else None

            # --- filename stem vs metadata.id -----------------------------
            stem = path.stem
            if meta_id and stem != meta_id:
                finding_id = "schema.filename-id-mismatch"
                is_fixed = (
                    kind == "logentry"
                    and _already_corrected(
                        correction_map,
                        str(meta_id),
                        finding_id,
                    )
                )
                results.append(CheckResult(
                    severity="INFO" if is_fixed else "WARN",
                    category="schema_filename",
                    id=finding_id,
                    message=(
                        f"{rel}: filename stem '{stem}' != metadata.id "
                        f"'{meta_id}' (Phase 1: rename deferred)"
                        + (f" (reconciled in correction LogEntry)" if is_fixed else "")
                    ),
                    entity_ref=str(rel),
                    fixable=(kind == "logentry" and not is_fixed),
                    suggested_fix=(
                        f"emit logentry.corrected for {meta_id}"
                        if kind == "logentry" and not is_fixed
                        else None
                    ),
                ))

            # --- Actor: two-class ID pattern check -------------------------
            if kind == "actor" and meta_id:
                reason = _check_actor_id_pattern(meta_id, allowed_role_ids)
                if reason is not None:
                    results.append(CheckResult(
                        severity="ERROR",
                        category="schema_filename",
                        id="schema.filename.invalid_actor_id_pattern",
                        message=(
                            f"{rel}: metadata.id '{meta_id}': {reason}"
                        ),
                        entity_ref=str(rel),
                    ))

            # --- filename date vs metadata.created ------------------------
            m = FILENAME_DATE_RE.search(stem)
            created = md.get("created") if isinstance(md, dict) else None
            if m and created:
                fn_date = m.group(1)  # YYYYMMDD
                created_str = created.isoformat() if hasattr(created, "isoformat") else str(created)
                if fn_date not in created_str.replace("-", ""):
                    finding_id = "schema.filename-date-mismatch"
                    is_fixed = (
                        kind == "logentry"
                        and isinstance(meta_id, str)
                        and _already_corrected(
                            correction_map,
                            meta_id,
                            finding_id,
                        )
                    )
                    results.append(CheckResult(
                        severity="INFO" if is_fixed else "WARN",
                        category="schema_filename",
                        id=finding_id,
                        message=(
                            f"{rel}: filename date '{fn_date}' not in "
                            f"metadata.created '{created_str}'"
                            + (f" (reconciled in correction LogEntry)"
                               if is_fixed else " (Phase 1: rename deferred)")
                        ),
                        entity_ref=str(rel),
                        fixable=(kind == "logentry" and not is_fixed),
                        suggested_fix=(
                            f"emit logentry.corrected for {meta_id}"
                            if kind == "logentry" and not is_fixed
                            else None
                        ),
                    ))

            # --- schema validation ----------------------------------------
            errs = _validate_against_schema(fm, schema) if schema else []
            for err in errs:
                # Owner policy: datetime-coercion errors are WARN (parser quirk),
                # all other schema errors are ERROR.
                finding_id = "schema.invalid"
                if "date-time" in err or "datetime" in err.lower() or "is not of type 'string'" in err and "created" in err:
                    results.append(CheckResult(
                        severity="WARN",
                        category="schema_filename",
                        id="schema.datetime-coercion",
                        message=f"{rel}: {err}",
                        entity_ref=str(rel),
                    ))
                else:
                    if kind == "logentry" and _MISSING_ACTOR_MARKER in err:
                        finding_id = "schema.invalid"
                        is_fixed = _already_corrected(
                            correction_map,
                            str(meta_id) if isinstance(meta_id, str) else path.stem,
                            finding_id,
                        )
                        results.append(CheckResult(
                            severity="INFO" if is_fixed else "ERROR",
                            category="schema_filename",
                            id=finding_id,
                            message=(
                                f"{rel}: {err}"
                                + (
                                    " (reconciled in correction LogEntry)"
                                    if is_fixed
                                    else ""
                                )
                            ),
                            entity_ref=str(rel),
                            fixable=(not is_fixed),
                            suggested_fix=(
                                f"emit logentry.corrected for {meta_id}"
                                if not is_fixed
                                else None
                            ),
                        ))
                        continue
                    results.append(CheckResult(
                        severity="ERROR",
                        category="schema_filename",
                        id=finding_id,
                        message=f"{rel}: {err}",
                        entity_ref=str(rel),
                    ))

    results.append(CheckResult(
        severity="INFO",
        category="schema_filename",
        id="schema.walked",
        message=f"walked {walked_count} entity file(s) across {len(KIND_TO_DIR)} kinds",
    ))
    return results


# --- narrow autofix (append-only log correction) -----------------------------
#
# Scope:
# - LogEntry filename-id mismatch
# - LogEntry filename date mismatch
# - LogEntry schema-invalid "actor is required" marker


def run_fix(ctx, results: list[CheckResult]) -> list[dict]:
    """Write correction LogEntries instead of editing source log files."""
    repo_root: Path = ctx["repo_root"]
    fixes: list[dict] = []
    correction_map = _load_log_correction_map(repo_root)

    fix_targets: dict[Path, set[str]] = {}
    for r in results:
        if r.category != "schema_filename":
            continue
        if r.entity_ref is None:
            continue
        rel = Path(r.entity_ref)
        if rel.parts[:2] != ("context", "logs"):
            continue
        if r.id not in {
            "schema.filename-id-mismatch",
            "schema.filename-date-mismatch",
            "schema.invalid",
        }:
            continue
        if r.id == "schema.invalid" and _MISSING_ACTOR_MARKER not in (
            r.message or ""
        ):
            continue
        fix_targets.setdefault(rel, set()).add(r.id)

    for rel, ids in fix_targets.items():
        abs_path = repo_root / rel
        if not abs_path.is_file():
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": "file not found",
            })
            continue

        fm, perr = _parse_frontmatter(abs_path)
        if fm is None:
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": f"frontmatter parse failed: {perr}",
            })
            continue

        md = fm.get("metadata") if isinstance(fm, dict) else None
        if not isinstance(md, dict):
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": "no metadata block in frontmatter",
            })
            continue

        target_id = str(md.get("id", abs_path.stem))
        unresolved = {
            fid for fid in ids if not _already_corrected(
                correction_map, target_id, fid
            )
        }
        if not unresolved:
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": "already corrected",
            })
            continue

        now = datetime.now(timezone.utc)
        log_id = f"LOG-{now.strftime('%Y%m%d_%H%M')}-Fix-{_safe_slug()}"
        correction_path = (
            repo_root / "context" / "logs" / now.strftime("%Y") / now.strftime("%m")
            / f"{log_id}.md"
        )
        details = {
            "corrects": {
                "kind": "LogEntry",
                "id": target_id,
                "path": str(rel),
            },
            "issues": sorted(unresolved),
        }
        spec: dict[str, object] = {
            "event_type": _CORRECTION_EVENT_TYPE,
            "timestamp": _to_isodate(now),
            "actor": "system",
            "subject": target_id,
            "subject_kind": "LogEntry",
            "summary": f"Correction for LogEntry {target_id}",
            "details": details,
        }
        fm_text = yaml.safe_dump(
            {
                "apiVersion": "processkit.projectious.work/v2",
                "kind": "LogEntry",
                "metadata": {
                    "id": log_id,
                    "created": _to_isodate(now),
                },
                "spec": spec,
            },
            sort_keys=False,
            allow_unicode=True,
        )
        correction_path.parent.mkdir(parents=True, exist_ok=True)
        correction_path.write_text(f"---\n{fm_text}---\n", encoding="utf-8")

        fixes.append({
            "category": "schema_filename",
            "entity": str(rel),
            "status": "applied",
            "correction_log_id": log_id,
            "correction_path": str(correction_path.relative_to(repo_root)),
            "issue_ids": sorted(unresolved),
            "rationale": "append-only logentry.corrected emitted",
        })
        correction_map.setdefault(target_id, set()).update(unresolved)

    return fixes
