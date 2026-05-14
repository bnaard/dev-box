"""v2 contract checks for processkit workflow/projection primitives."""

from __future__ import annotations

import datetime as _dt
import hashlib
from pathlib import Path
from typing import Any

import yaml

from .common import CheckResult


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


def _iter_entities(repo_root: Path, rel_dir: str) -> list[tuple[Path, dict[str, Any]]]:
    base = repo_root / "context" / rel_dir
    if not base.is_dir():
        return []
    out: list[tuple[Path, dict[str, Any]]] = []
    for path in sorted(base.rglob("*.md")):
        if path.name.startswith("INDEX"):
            continue
        data = _load_frontmatter(path)
        if data:
            out.append((path, data))
    return out


def _is_v2(data: dict[str, Any]) -> bool:
    return str(data.get("apiVersion", "")).endswith("/v2")


def _parse_dt(value: Any) -> _dt.datetime | None:
    if not value:
        return None
    if isinstance(value, _dt.datetime):
        return value
    text = str(value)
    if text.endswith("Z"):
        text = text[:-1] + "+00:00"
    try:
        parsed = _dt.datetime.fromisoformat(text)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=_dt.timezone.utc)
    return parsed


def _rel(repo_root: Path, path: Path) -> str:
    return path.relative_to(repo_root).as_posix()


def _artifact_ids_by_kind(
    repo_root: Path,
    kind: str,
) -> dict[str, tuple[Path, dict[str, Any]]]:
    out: dict[str, tuple[Path, dict[str, Any]]] = {}
    for path, data in _iter_entities(repo_root, "artifacts"):
        if not _is_v2(data):
            continue
        spec = data.get("spec", {})
        if isinstance(spec, dict) and spec.get("kind") == kind:
            entity_id = data.get("metadata", {}).get("id")
            if entity_id:
                out[str(entity_id)] = (path, data)
    return out


def _bindings_by_type(repo_root: Path, binding_type: str) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for path, data in _iter_entities(repo_root, "bindings"):
        if not _is_v2(data):
            continue
        spec = data.get("spec", {})
        if isinstance(spec, dict) and spec.get("type") == binding_type:
            out.append({"path": path, "data": data, "spec": spec})
    return out


def _iter_policy_artifacts(
    repo_root: Path,
) -> dict[str, tuple[Path, dict[str, Any]]]:
    out: dict[str, tuple[Path, dict[str, Any]]] = {}
    for path, data in _iter_entities(repo_root, "artifacts"):
        if not _is_v2(data):
            continue
        spec = data.get("spec", {})
        if not isinstance(spec, dict):
            continue
        kind = str(spec.get("kind") or "")
        if kind.endswith("-policy"):
            entity_id = data.get("metadata", {}).get("id")
            if entity_id:
                out[str(entity_id)] = (path, data)
    return out


def _has_calibration(repo_root: Path, eval_artifact_id: str) -> bool:
    for _path, data in _iter_entities(repo_root, "logs"):
        spec = data.get("spec", {})
        if not isinstance(spec, dict):
            continue
        if spec.get("event_type") != "eval.judge.calibrated":
            continue
        if spec.get("subject") == eval_artifact_id:
            return True
        details = spec.get("details", {})
        if isinstance(details, dict) and details.get("eval_artifact_id") == eval_artifact_id:
            return True
    return False


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    since_files: set[Path] | None = ctx.get("since_files")
    results: list[CheckResult] = []
    checked = 0

    for path, data in _iter_entities(repo_root, "workitems"):
        if since_files is not None and path not in since_files:
            continue
        if not _is_v2(data):
            continue
        spec = data.get("spec", {})
        if not isinstance(spec, dict):
            continue
        if spec.get("type") == "process-instance":
            checked += 1
            if not (spec.get("process_definition_artifact") or spec.get("process_definition")):
                results.append(CheckResult(
                    severity="ERROR",
                    category="v2_contracts",
                    id="v2.process-instance-without-definition",
                    message=f"{_rel(repo_root, path)}: process-instance has no definition",
                    entity_ref=_rel(repo_root, path),
                ))

    for binding in _bindings_by_type(repo_root, "time-window"):
        path = binding["path"]
        if since_files is not None and path not in since_files:
            continue
        checked += 1
        conditions = binding["spec"].get("conditions") or {}
        if not isinstance(conditions, dict) or not conditions.get("recurrence_rule"):
            results.append(CheckResult(
                severity="ERROR",
                category="v2_contracts",
                id="v2.schedule-without-rule",
                message=f"{_rel(repo_root, path)}: time-window has no recurrence_rule",
                entity_ref=_rel(repo_root, path),
            ))

    budget_subjects = {
        b["spec"].get("subject")
        for b in _bindings_by_type(repo_root, "budget-application")
    }
    for artifact_id, (path, _data) in _artifact_ids_by_kind(repo_root, "cost-policy").items():
        if since_files is not None and path not in since_files:
            continue
        checked += 1
        if artifact_id not in budget_subjects:
            results.append(CheckResult(
                severity="ERROR",
                category="v2_contracts",
                id="v2.cost-policy-without-budget-application",
                message=f"{_rel(repo_root, path)}: cost-policy is not bound to a budget",
                entity_ref=_rel(repo_root, path),
            ))

    policy_artifacts = _iter_policy_artifacts(repo_root)
    for artifact_id, (path, data) in policy_artifacts.items():
        if since_files is not None and path not in since_files:
            continue
        spec = data.get("spec", {})
        if not isinstance(spec, dict):
            continue
        supersedes = spec.get("supersedes") or []
        if isinstance(supersedes, str):
            supersedes = [supersedes]
        if not isinstance(supersedes, list):
            continue
        for old_id in supersedes:
            checked += 1
            if old_id not in policy_artifacts:
                results.append(CheckResult(
                    severity="ERROR",
                    category="v2_contracts",
                    id="v2.policy-supersedes-chain-break",
                    message=(
                        f"{_rel(repo_root, path)}: policy supersedes "
                        f"unknown policy artifact {old_id!r}"
                    ),
                    entity_ref=_rel(repo_root, path),
                ))
                continue
            _old_path, old_data = policy_artifacts[str(old_id)]
            old_spec = old_data.get("spec", {})
            old_replacement = (
                old_spec.get("superseded_by")
                if isinstance(old_spec, dict)
                else None
            )
            if old_replacement and old_replacement != artifact_id:
                results.append(CheckResult(
                    severity="ERROR",
                    category="v2_contracts",
                    id="v2.policy-supersedes-chain-break",
                    message=(
                        f"{_rel(repo_root, path)}: policy supersedes "
                        f"{old_id!r}, but old policy points to "
                        f"{old_replacement!r}"
                    ),
                    entity_ref=_rel(repo_root, path),
                ))

    for artifact_id, (path, data) in _artifact_ids_by_kind(repo_root, "eval-spec").items():
        if since_files is not None and path not in since_files:
            continue
        spec = data.get("spec", {})
        checked += 1
        if not isinstance(spec, dict):
            continue
        if spec.get("eval_kind") == "llm-as-judge" or spec.get("judge"):
            if not _has_calibration(repo_root, artifact_id):
                results.append(CheckResult(
                    severity="ERROR",
                    category="v2_contracts",
                    id="v2.eval-gate-uncalibrated",
                    message=f"{_rel(repo_root, path)}: LLM judge eval has no calibration log",
                    entity_ref=_rel(repo_root, path),
                ))

    for _artifact_id, (path, data) in _artifact_ids_by_kind(repo_root, "agent-card").items():
        if since_files is not None and path not in since_files:
            continue
        spec = data.get("spec", {})
        if not isinstance(spec, dict):
            continue
        projection_path = spec.get("projection_path")
        projection_checksum = spec.get("projection_checksum")
        if not projection_path and not projection_checksum:
            continue
        checked += 1
        if not projection_path:
            continue
        projected = repo_root / str(projection_path)
        if not projected.is_file():
            results.append(CheckResult(
                severity="ERROR",
                category="v2_contracts",
                id="v2.agent-card-projection-missing",
                message=f"{_rel(repo_root, path)}: projected agent card is missing",
                entity_ref=_rel(repo_root, path),
            ))
            continue
        if projection_checksum:
            digest = hashlib.sha256(projected.read_bytes()).hexdigest()
            if digest != projection_checksum:
                results.append(CheckResult(
                    severity="ERROR",
                    category="v2_contracts",
                    id="v2.agent-card-projection-stale",
                    message=f"{_rel(repo_root, path)}: projected agent card checksum differs",
                entity_ref=_rel(repo_root, path),
            ))

    for binding in _bindings_by_type(repo_root, "triage-classification"):
        path = binding["path"]
        if since_files is not None and path not in since_files:
            continue
        checked += 1
        conditions = binding["spec"].get("conditions") or {}
        mode = conditions.get("injection_mode") if isinstance(conditions, dict) else None
        if mode not in {"interrupt", "ambient", "next-cycle"}:
            results.append(CheckResult(
                severity="ERROR",
                category="v2_contracts",
                id="v2.inbox-injection-mode-untyped",
                message=(
                    f"{_rel(repo_root, path)}: triage-classification "
                    "binding has no valid conditions.injection_mode"
                ),
                entity_ref=_rel(repo_root, path),
            ))

    for path, data in _iter_entities(repo_root, "bindings"):
        if since_files is not None and path not in since_files:
            continue
        if not _is_v2(data):
            continue
        spec = data.get("spec", {})
        if not isinstance(spec, dict):
            continue
        if spec.get("type") == "triage-classification":
            continue
        conditions = spec.get("conditions") or {}
        if isinstance(conditions, dict) and "injection_mode" in conditions:
            checked += 1
            results.append(CheckResult(
                severity="ERROR",
                category="v2_contracts",
                id="v2.inbox-injection-mode-untyped",
                message=(
                    f"{_rel(repo_root, path)}: conditions.injection_mode "
                    "is only valid on triage-classification bindings"
                ),
                entity_ref=_rel(repo_root, path),
            ))

    now = _dt.datetime.now(_dt.timezone.utc)
    for path, data in _iter_entities(repo_root, "notes"):
        if since_files is not None and path not in since_files:
            continue
        if not _is_v2(data):
            continue
        spec = data.get("spec", {})
        if not isinstance(spec, dict):
            continue
        inbox = spec.get("inbox")
        if not isinstance(inbox, dict) or inbox.get("status") != "claimed":
            continue
        checked += 1
        stamp = _parse_dt(inbox.get("claimed_at"))
        if stamp and now - stamp > _dt.timedelta(hours=24):
            results.append(CheckResult(
                severity="WARN",
                category="v2_contracts",
                id="v2.inbox-orphaned-claim",
                message=f"{_rel(repo_root, path)}: inbox claim is older than 24h",
                entity_ref=_rel(repo_root, path),
            ))

    results.append(CheckResult(
        severity="INFO",
        category="v2_contracts",
        id="v2.contracts-checked",
        message=f"checked {checked} v2 contract-bearing entity file(s)",
    ))
    return results
