"""v2 schema vocabulary checks.

Validates closed subtype vocabularies declared on Schema entities for v2
content. Existing v1 entities are left to the explicit v1->v2 migration
path; once migrated, unknown Artifact kinds, Binding types, WorkItem types,
LogEntry event types, and incomplete Migration version metadata are errors.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml

from .common import CheckResult


VOCAB_CHECKS = {
    "Artifact": (
        "artifacts",
        "known_kinds",
        "kind",
        "schema.unknown-kind-without-schema-entry",
    ),
    "Binding": (
        "bindings",
        "known_types",
        "type",
        "schema.unknown-type-without-schema-entry",
    ),
    "WorkItem": (
        "workitems",
        "known_types",
        "type",
        "schema.unknown-type-without-schema-entry",
    ),
    "LogEntry": (
        "logs",
        "known_event_types",
        "event_type",
        "schema.unknown-event-type-without-schema-entry",
    ),
    "Migration": (
        "migrations",
        "known_kinds",
        "kind",
        "schema.unknown-migration-kind-without-schema-entry",
    ),
}

MIGRATION_VERSION_FIELDS = {
    "source_api_version",
    "source_processkit_version",
    "target_api_version",
    "target_processkit_version",
    "apply_mode",
}

DEMOTED_SRC_SCHEMA_KINDS = {
    "Model": "model.yaml",
    "Process": "process.yaml",
    "Schedule": "schedule.yaml",
    "StateMachine": "statemachine.yaml",
}


def _load_frontmatter(path: Path) -> dict[str, Any] | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    if text.startswith("---"):
        parts = text.split("---", 2)
        payload = parts[1] if len(parts) >= 3 else text
    else:
        payload = text
    try:
        data = yaml.safe_load(payload)
    except yaml.YAMLError:
        return None
    return data if isinstance(data, dict) else None


def _load_schema_vocab(repo_root: Path, kind: str, field: str) -> set[str]:
    filename = kind.lower().replace("decisionrecord", "decisionrecord")
    if kind == "TeamMember":
        filename = "team-member"
    schema_path = repo_root / "context" / "schemas" / f"{filename}.yaml"
    if not schema_path.is_file():
        schema_path = (
            repo_root / "src" / "context" / "schemas" / f"{filename}.yaml"
        )
    data = _load_frontmatter(schema_path)
    spec = data.get("spec", {}) if data else {}
    values = spec.get(field) or []
    return {str(v) for v in values}


def _iter_kind_files(repo_root: Path, kind_dir: str) -> list[Path]:
    root = repo_root / "context" / kind_dir
    if not root.is_dir():
        return []
    return sorted(
        p for p in root.rglob("*.md")
        if not p.name.startswith("INDEX")
    )


def _is_v2(data: dict[str, Any]) -> bool:
    return str(data.get("apiVersion", "")).endswith("/v2")


def _check_demoted_src_schemas(repo_root: Path) -> list[CheckResult]:
    schemas_root = repo_root / "src" / "context" / "schemas"
    if not schemas_root.is_dir():
        return []

    results: list[CheckResult] = []
    index_text = ""
    index_path = schemas_root / "INDEX.md"
    try:
        index_text = index_path.read_text(encoding="utf-8")
    except OSError:
        pass

    for kind, filename in DEMOTED_SRC_SCHEMA_KINDS.items():
        schema_path = schemas_root / filename
        if schema_path.is_file():
            rel = schema_path.relative_to(repo_root)
            results.append(CheckResult(
                severity="ERROR",
                category="schema_vocabulary",
                id="schema.demoted-kind-still-shipped",
                message=(
                    f"{rel}: {kind} must not remain in the shipped src/ "
                    "entity vocabulary after v2 demotion"
                ),
                entity_ref=str(rel),
            ))

        if index_text and filename in index_text:
            rel = index_path.relative_to(repo_root)
            results.append(CheckResult(
                severity="ERROR",
                category="schema_vocabulary",
                id="schema.demoted-kind-still-indexed",
                message=(
                    f"{rel}: {kind} still appears in the shipped src/ "
                    "schema index after v2 demotion"
                ),
                entity_ref=str(rel),
            ))

    return results


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    since_files: set[Path] | None = ctx.get("since_files")
    results: list[CheckResult] = []
    checked = 0

    results.extend(_check_demoted_src_schemas(repo_root))

    for (
        kind,
        (kind_dir, vocab_field, spec_field, finding_id),
    ) in VOCAB_CHECKS.items():
        allowed = _load_schema_vocab(repo_root, kind, vocab_field)
        if not allowed:
            results.append(CheckResult(
                severity="WARN",
                category="schema_vocabulary",
                id=f"schema.{vocab_field}.missing",
                message=f"{kind} schema declares no {vocab_field}",
            ))
            continue
        for path in _iter_kind_files(repo_root, kind_dir):
            if since_files is not None and path not in since_files:
                continue
            data = _load_frontmatter(path)
            if not data or not _is_v2(data):
                continue
            spec = data.get("spec", {})
            value = spec.get(spec_field) if isinstance(spec, dict) else None
            checked += 1
            if value and value not in allowed:
                rel = path.relative_to(repo_root)
                results.append(CheckResult(
                    severity="ERROR",
                    category="schema_vocabulary",
                    id=finding_id,
                    message=(
                        f"{rel}: {kind}.{spec_field}={value!r} is not "
                        f"declared in Schema.{vocab_field}"
                    ),
                    entity_ref=str(rel),
                ))

    for path in _iter_kind_files(repo_root, "migrations"):
        if since_files is not None and path not in since_files:
            continue
        data = _load_frontmatter(path)
        if not data or not _is_v2(data):
            continue
        spec = data.get("spec", {})
        if not isinstance(spec, dict):
            continue
        checked += 1
        missing = sorted(f for f in MIGRATION_VERSION_FIELDS if not spec.get(f))
        if missing:
            rel = path.relative_to(repo_root)
            results.append(CheckResult(
                severity="ERROR",
                category="schema_vocabulary",
                id="schema.migration-without-source-version",
                message=f"{rel}: Migration missing required v2 field(s): {missing}",
                entity_ref=str(rel),
            ))

    results.append(CheckResult(
        severity="INFO",
        category="schema_vocabulary",
        id="schema.vocabulary-checked",
        message=f"checked {checked} v2 entity file(s) for closed vocabulary",
    ))
    return results
