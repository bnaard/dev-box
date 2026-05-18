#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit eval-gate-authoring MCP server."""
from __future__ import annotations

import datetime as _dt
import os
import sys
from pathlib import Path


def _find_lib() -> Path:
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for c in (here / "src" / "lib", here / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import config, entity, ids, index, log, paths, schema  # noqa: E402

server = FastMCP("processkit-eval-gate-authoring")

_EVAL_KINDS = {"manual", "deterministic", "llm-as-judge", "hybrid"}


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _existing(kind: str) -> set[str]:
    db = index.open_db()
    try:
        return index.existing_ids(db, kind)
    finally:
        db.close()


def _write(ent: entity.Entity, root: Path) -> Path:
    target = paths.entity_path(ent.kind, ent.id, None, root)
    ent.write(target)
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()
    return target


def _load_artifact(id: str) -> entity.Entity | None:
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="Artifact")
    finally:
        db.close()
    if row and row.get("path"):
        return entity.load(row["path"])
    return None


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def collect_run_outputs(run_root: str = "context/runs", limit: int = 20) -> list[dict]:
    """List recent run-output files that may seed eval cases."""
    root = paths.find_project_root()
    base = (root / run_root).resolve()
    if root.resolve() not in base.parents and base != root.resolve():
        return [{"error": "run_root must stay inside the project root"}]
    if not base.is_dir():
        return []
    out: list[dict] = []
    for p in sorted(base.rglob("*"), key=lambda f: f.stat().st_mtime, reverse=True):
        if not p.is_file():
            continue
        rel = p.relative_to(root).as_posix()
        out.append({"path": rel, "bytes": p.stat().st_size})
        if len(out) >= limit:
            break
    return out


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def codify_eval(
    category_id: str,
    name: str,
    eval_kind: str,
    description: str,
    judge: str | None = None,
) -> dict:
    """Create an eval-spec Artifact and paired Gate."""
    if eval_kind not in _EVAL_KINDS:
        return {"error": f"invalid eval_kind {eval_kind!r}; valid: {sorted(_EVAL_KINDS)}"}

    root = paths.find_project_root()
    cfg = config.load_config(root)
    artifact_id = ids.generate_id(
        "Artifact",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=name if cfg.id_slug else None,
        existing=_existing("Artifact"),
    )
    artifact_spec = {
        "name": name,
        "kind": "eval-spec",
        "format": "processkit-eval-spec",
        "produced_at": _now_iso(),
        "category_id": category_id,
        "eval_kind": eval_kind,
        "description": description,
    }
    if judge:
        artifact_spec["judge"] = judge
    errors = schema.validate_spec("Artifact", artifact_spec)
    if errors:
        return {"error": "artifact schema validation failed", "details": errors}
    art = entity.new("Artifact", artifact_id, artifact_spec, body=description)
    artifact_path = _write(art, root)

    gate_id = ids.generate_id(
        "Gate",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=name if cfg.id_slug else None,
        existing=_existing("Gate"),
    )
    gate_kind = "hybrid" if eval_kind == "llm-as-judge" else "automated"
    gate_spec = {
        "name": name,
        "description": description,
        "kind": gate_kind,
        "validator": f"Evaluate run output against eval-spec {artifact_id}.",
        "blocking": True,
        "evidence_required": True,
        "eval_artifact": artifact_id,
    }
    errors = schema.validate_spec("Gate", gate_spec)
    if errors:
        return {"error": "gate schema validation failed", "details": errors}
    gate = entity.new("Gate", gate_id, gate_spec)
    gate_path = _write(gate, root)

    log.log_side_effect(
        "Artifact", artifact_id, "eval.spec.created",
        f"Created eval-spec {artifact_id!r} and Gate {gate_id!r}",
        root=root,
        actor=artifact_id,
        details={"gate_id": gate_id, "eval_kind": eval_kind},
    )
    return {
        "artifact_id": artifact_id,
        "artifact_path": str(artifact_path),
        "gate_id": gate_id,
        "gate_path": str(gate_path),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def calibrate_judge(
    judge_id: str,
    eval_artifact_id: str | None = None,
    agreement: float | None = None,
    sample_n: int = 0,
    evidence: str | None = None,
) -> dict:
    """Record LLM-as-judge calibration evidence as a LogEntry."""
    root = paths.find_project_root()
    details = {
        "judge_id": judge_id,
        "eval_artifact_id": eval_artifact_id,
        "agreement": agreement,
        "sample_n": sample_n,
        "evidence": evidence,
    }
    log_id = log.log_side_effect(
        "Artifact", eval_artifact_id or judge_id, "eval.judge.calibrated",
        f"Calibrated judge {judge_id!r}",
        root=root,
        actor=judge_id,
        details=details,
    )
    return {"ok": True, "log_id": log_id, "details": details}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def bind_eval_to_runs(
    eval_artifact_id: str,
    target: str,
    description: str | None = None,
) -> dict:
    """Bind an eval-spec Artifact to a run target."""
    root = paths.find_project_root()
    art = _load_artifact(eval_artifact_id)
    if art is None:
        return {"error": f"Artifact not found: {eval_artifact_id!r}"}
    if art.spec.get("kind") != "eval-spec":
        return {"error": f"Artifact {eval_artifact_id!r} is not kind='eval-spec'"}

    cfg = config.load_config(root)
    bind_id = ids.generate_id(
        "Binding",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text="policy-application" if cfg.id_slug else None,
        existing=_existing("Binding"),
    )
    spec = {
        "type": "policy-application",
        "subject": eval_artifact_id,
        "target": target,
        "conditions": {
            "enforcement_point": "run-eval",
            "eval_kind": art.spec.get("eval_kind"),
        },
    }
    if description:
        spec["description"] = description
    errors = schema.validate_spec("Binding", spec)
    if errors:
        return {"error": "binding schema validation failed", "details": errors}
    bind = entity.new("Binding", bind_id, spec)
    bind_path = _write(bind, root)
    log.log_side_effect(
        "Binding", bind_id, "eval.bound",
        f"Bound eval-spec {eval_artifact_id!r} to {target!r}",
        root=root,
        actor=eval_artifact_id,
    )
    return {"id": bind_id, "path": str(bind_path)}


if __name__ == "__main__":
    server.run(transport="stdio")
