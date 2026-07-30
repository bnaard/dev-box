#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""processkit security-projections MCP server."""
from __future__ import annotations

import hashlib
import json
import os
import sys
from pathlib import Path
from typing import Any

import yaml


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

from processkit import entity, index, log, paths  # noqa: E402

server = FastMCP("processkit-security-projections")


def _load_artifact(id: str) -> entity.Entity | None:
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="Artifact")
    finally:
        db.close()
    if row and row.get("path"):
        return entity.load(row["path"])
    return None


def _parse_body(body: str) -> dict[str, Any]:
    text = (body or "").strip()
    if not text:
        return {}
    try:
        data = json.loads(text)
    except json.JSONDecodeError:
        data = yaml.safe_load(text)
    return data if isinstance(data, dict) else {}


def _project_path(root: Path, output_path: str) -> Path:
    path = Path(output_path)
    if not path.is_absolute():
        path = root / path
    resolved = path.resolve()
    if root.resolve() not in resolved.parents and resolved != root.resolve():
        raise ValueError("output_path must stay inside the project root")
    return resolved


def _write(path: Path, payload: str, write: bool) -> str:
    checksum = hashlib.sha256(payload.encode("utf-8")).hexdigest()
    if write:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(payload, encoding="utf-8")
    return checksum


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def project_agent_ids_rule(
    artifact_id: str,
    output_path: str | None = None,
    write: bool = True,
) -> dict:
    """Project an agent-ids-rule Artifact to canonical JSON."""
    root = paths.find_project_root()
    artifact = _load_artifact(artifact_id)
    if artifact is None:
        return {"error": f"Artifact not found: {artifact_id!r}"}
    if artifact.spec.get("kind") != "agent-ids-rule":
        return {"error": f"Artifact {artifact_id!r} is not kind='agent-ids-rule'"}
    rule = artifact.spec.get("rule")
    if not isinstance(rule, dict):
        rule = _parse_body(artifact.body)
    if not rule:
        return {"error": "agent-ids-rule Artifact must provide spec.rule or body"}
    rule.setdefault("id", artifact.id)
    rule.setdefault("name", artifact.spec.get("name") or artifact.id)
    payload = json.dumps(rule, sort_keys=True, indent=2) + "\n"
    out = output_path or f"build/security/{artifact.id}-agent-ids-rule.json"
    path = _project_path(root, out)
    checksum = _write(path, payload, write)
    if write:
        log.log_side_effect(
            "Artifact", artifact.id, "security.agent_ids_rule_projected",
            f"Projected Agent-IDS rule {artifact.id!r}",
            root=root,
            actor=artifact.id,
            details={"output_path": out, "sha256": checksum},
        )
    return {"rule": rule, "checksum": checksum, "output_path": str(path), "written": write}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def project_tetragon_tracing_policy(
    artifact_id: str,
    output_path: str | None = None,
    write: bool = True,
) -> dict:
    """Project a policy Artifact to Tetragon-style TracingPolicy YAML."""
    root = paths.find_project_root()
    artifact = _load_artifact(artifact_id)
    if artifact is None:
        return {"error": f"Artifact not found: {artifact_id!r}"}
    if artifact.spec.get("kind") not in {"agent-ids-rule", "image-provenance-policy"}:
        return {
            "error": (
                "Artifact must be kind='agent-ids-rule' or "
                "kind='image-provenance-policy'"
            )
        }
    source = artifact.spec.get("tetragon")
    if not isinstance(source, dict):
        source = _parse_body(artifact.body)
    kprobes = source.get("kprobes") if isinstance(source, dict) else None
    policy = {
        "apiVersion": "cilium.io/v1alpha1",
        "kind": "TracingPolicy",
        "metadata": {
            "name": (artifact.spec.get("name") or artifact.id).lower().replace("_", "-"),
            "annotations": {"processkit.artifact": artifact.id},
        },
        "spec": {"kprobes": kprobes or []},
    }
    payload = yaml.safe_dump(policy, sort_keys=False)
    out = output_path or f"build/security/{artifact.id}-tracing-policy.yaml"
    path = _project_path(root, out)
    checksum = _write(path, payload, write)
    if write:
        log.log_side_effect(
            "Artifact", artifact.id, "security.tetragon_policy_projected",
            f"Projected Tetragon policy {artifact.id!r}",
            root=root,
            actor=artifact.id,
            details={"output_path": out, "sha256": checksum},
        )
    return {
        "policy": policy,
        "checksum": checksum,
        "output_path": str(path),
        "written": write,
    }


if __name__ == "__main__":
    server.run(transport="stdio")
