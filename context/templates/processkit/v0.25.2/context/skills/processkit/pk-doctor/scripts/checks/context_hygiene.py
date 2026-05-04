"""Context hygiene checks that cut across individual primitive schemas."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

import yaml

from .common import CheckResult


_FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---", re.S)
_TIMESTAMPED_BIND_RE = re.compile(r"^BIND-\d{8}_\d{4}-")
_DETERMINISTIC_BIND_RE = re.compile(r"^BIND-[a-z][a-z0-9-]+-h[0-9a-f]{6}$")
_MODEL_SPEC_ID_RE = re.compile(r"^ART-\d{8}_\d{4}-ModelSpec-[a-z0-9-]+$")


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


def _model_score_ids(repo_root: Path) -> set[str]:
    scores = (
        repo_root
        / "context/skills/processkit/model-recommender/mcp/model_scores.json"
    )
    if not scores.is_file():
        return set()
    try:
        data = json.loads(scores.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return set()
    return {
        str(model.get("id"))
        for model in data.get("models", [])
        if model.get("id")
    }


def _model_spec_artifacts(
    repo_root: Path,
) -> tuple[set[str], set[str], list[CheckResult]]:
    ids: set[str] = set()
    profile_ids: set[str] = set()
    results: list[CheckResult] = []
    for root_name in ("context/artifacts", "src/context/artifacts"):
        artifacts = repo_root / root_name
        if not artifacts.is_dir():
            continue
        for path in sorted(artifacts.glob("ART-*.md")):
            fm = _load_frontmatter(path)
            if not fm:
                continue
            spec = fm.get("spec") if isinstance(fm.get("spec"), dict) else {}
            if fm.get("kind") != "Artifact" or spec.get("kind") != "model-spec":
                continue
            md = fm.get("metadata") if isinstance(fm.get("metadata"), dict) else {}
            artifact_id = str(md.get("id") or "")
            if artifact_id:
                ids.add(artifact_id)
            profile_ids.update(str(x) for x in spec.get("profile_ids", []) or [])
            if not _MODEL_SPEC_ID_RE.match(path.stem):
                results.append(CheckResult(
                    severity="ERROR",
                    category="context_hygiene",
                    id="model-spec.filename-policy",
                    message=(
                        f"{path.relative_to(repo_root)} is a model-spec "
                        "artifact but does not use the timestamped "
                        "ART-YYYYMMDD_HHMM-ModelSpec-* filename policy"
                    ),
                    entity_ref=path.stem,
                    suggested_fix=(
                        "regenerate model-spec artifacts with "
                        "model-recommender/scripts/migrate_models.py"
                    ),
                ))
            if artifact_id and not _MODEL_SPEC_ID_RE.match(artifact_id):
                results.append(CheckResult(
                    severity="ERROR",
                    category="context_hygiene",
                    id="model-spec.id-policy",
                    message=(
                        f"{path.relative_to(repo_root)} metadata.id "
                        f"{artifact_id!r} does not use the timestamped "
                        "ART-YYYYMMDD_HHMM-ModelSpec-* policy"
                    ),
                    entity_ref=artifact_id,
                ))
    return ids, profile_ids, results


def _scan_model_bindings(repo_root: Path, model_artifact_ids: set[str]) -> list[CheckResult]:
    results: list[CheckResult] = []
    for root_name in ("context/bindings", "src/context/bindings"):
        root = repo_root / root_name
        if not root.is_dir():
            continue
        timestamped = deterministic = 0
        for path in sorted(root.glob("*.md")):
            stem = path.stem
            if _TIMESTAMPED_BIND_RE.match(stem):
                timestamped += 1
            if _DETERMINISTIC_BIND_RE.match(stem):
                deterministic += 1
            fm = _load_frontmatter(path)
            if not fm:
                continue
            spec = fm.get("spec") if isinstance(fm.get("spec"), dict) else {}
            if spec.get("type") != "model-assignment":
                continue
            target = str(spec.get("target") or "")
            if target.startswith("MODEL-"):
                results.append(CheckResult(
                    severity="WARN",
                    category="context_hygiene",
                    id="model-binding.legacy-target",
                    message=(
                        f"{path.relative_to(repo_root)} targets legacy {target}; "
                        "model-assignment should target a timestamped "
                        "model-spec Artifact"
                    ),
                    entity_ref=target,
                    suggested_fix="rewrite binding target to model-spec Artifact",
                ))
                continue
            if target.startswith("ART-model-"):
                results.append(CheckResult(
                    severity="ERROR",
                    category="context_hygiene",
                    id="model-binding.legacy-artifact-target",
                    message=(
                        f"{path.relative_to(repo_root)} targets {target}; "
                        "model-spec Artifacts must use timestamped "
                        "ART-YYYYMMDD_HHMM-ModelSpec-* ids"
                    ),
                    entity_ref=target,
                    suggested_fix="rewrite binding target to the timestamped model-spec Artifact",
                ))
                continue
            if not target.startswith("ART-"):
                results.append(CheckResult(
                    severity="WARN",
                    category="context_hygiene",
                    id="model-binding.invalid-target",
                    message=(
                        f"{path.relative_to(repo_root)} target {target!r} is not "
                        "an Artifact(kind=model-spec) id"
                    ),
                    entity_ref=target,
                ))
                continue
            if target not in model_artifact_ids:
                results.append(CheckResult(
                    severity="ERROR",
                    category="context_hygiene",
                    id="model-binding.missing-target",
                    message=(
                        f"{path.relative_to(repo_root)} targets missing "
                        f"model-spec artifact {target}"
                    ),
                    entity_ref=target,
                ))
            if spec.get("target_kind") != "Artifact":
                results.append(CheckResult(
                    severity="WARN",
                    category="context_hygiene",
                    id="model-binding.target-kind",
                    message=(
                        f"{path.relative_to(repo_root)} model-assignment should "
                        "set target_kind: Artifact"
                    ),
                    entity_ref=target,
                ))
        if timestamped and deterministic:
            results.append(CheckResult(
                severity="WARN",
                category="context_hygiene",
                id="binding.filename-style-mixed",
                message=(
                    f"{root_name} mixes timestamped ({timestamped}) and "
                    f"deterministic ({deterministic}) Binding filenames"
                ),
                suggested_fix="normalize shipped/default bindings to one filename policy",
            ))
    return results


def _archive_candidate_count(root: Path, pattern: str) -> int:
    base = root
    if not base.is_dir():
        return 0
    return sum(1 for p in base.rglob(pattern) if p.is_file())


def _sqlite_vec_available() -> bool:
    try:
        import sqlite3
        import sqlite_vec  # type: ignore

        conn = sqlite3.connect(":memory:")
        try:
            conn.enable_load_extension(True)
            sqlite_vec.load(conn)
            conn.enable_load_extension(False)
            return True
        finally:
            conn.close()
    except Exception:
        return False


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    results: list[CheckResult] = []

    for rel in (
        "context/models",
        "context/schemas/model.yaml",
        "context/schemas/metric.yaml",
        "context/schemas/process.yaml",
        "context/schemas/schedule.yaml",
        "context/schemas/statemachine.yaml",
        "context/processes",
    ):
        path = repo_root / rel
        if path.exists():
            results.append(CheckResult(
                severity="WARN",
                category="context_hygiene",
                id="demoted-content.present",
                message=f"{rel} is demoted from current v2 deliverables",
                entity_ref=rel,
                suggested_fix="archive/remove through the release migration path",
            ))

    (
        model_artifact_ids,
        artifact_profile_ids,
        model_spec_results,
    ) = _model_spec_artifacts(repo_root)
    results.extend(model_spec_results)
    score_ids = _model_score_ids(repo_root)
    if score_ids and not model_artifact_ids:
        results.append(CheckResult(
            severity="WARN",
            category="context_hygiene",
            id="model-spec.missing",
            message="model_scores.json exists but no Artifact(kind=model-spec) files exist",
        ))
    missing_profiles = sorted(score_ids - artifact_profile_ids)
    stale_profiles = sorted(artifact_profile_ids - score_ids)
    if missing_profiles:
        results.append(CheckResult(
            severity="WARN",
            category="context_hygiene",
            id="model-spec.projection-missing",
            message=(
                f"{len(missing_profiles)} model_scores.json profile(s) have no "
                "model-spec artifact coverage"
            ),
            extra={"sample": missing_profiles[:10]},
        ))
    if stale_profiles:
        results.append(CheckResult(
            severity="WARN",
            category="context_hygiene",
            id="model-spec.projection-stale",
            message=(
                f"{len(stale_profiles)} model-spec profile_id(s) are absent from "
                "model_scores.json"
            ),
            extra={"sample": stale_profiles[:10]},
        ))

    results.extend(_scan_model_bindings(repo_root, model_artifact_ids))

    artifacts = repo_root / "context" / "artifacts"
    for path in sorted(artifacts.glob("ART-*_0000-*.md")):
        results.append(CheckResult(
            severity="WARN",
            category="context_hygiene",
            id="artifact.suspicious-zero-time",
            message=f"{path.relative_to(repo_root)} uses suspicious _0000 timestamp",
            entity_ref=path.stem,
        ))
    for path in sorted(artifacts.glob("*.html")):
        results.append(CheckResult(
            severity="ERROR",
            category="context_hygiene",
            id="artifact.html-sidecar",
            message=f"{path.relative_to(repo_root)} is an HTML artifact sidecar",
            entity_ref=str(path.relative_to(repo_root)),
            suggested_fix="convert to markdown/pointer artifact or move outside context artifacts",
        ))

    applied_migrations = _archive_candidate_count(
        repo_root / "context" / "migrations" / "applied", "*.md"
    )
    if applied_migrations:
        results.append(CheckResult(
            severity="WARN",
            category="context_hygiene",
            id="archive.applied-migrations",
            message=f"{applied_migrations} applied migration(s) are archive candidates",
            suggested_fix="plan archival via context-archiving policy",
        ))

    if not _sqlite_vec_available():
        results.append(CheckResult(
            severity="WARN",
            category="context_hygiene",
            id="semantic.sqlite-vec-unavailable",
            message=(
                "sqlite-vec is not importable/loadable; semantic_search_entities "
                "will return no vector results and hybrid search falls back to FTS"
            ),
            suggested_fix="install sqlite-vec in the MCP runtime or fix aibox dependency provisioning",
        ))

    if not results:
        results.append(CheckResult(
            severity="INFO",
            category="context_hygiene",
            id="context-hygiene.ok",
            message="context hygiene checks passed",
        ))
    return results
