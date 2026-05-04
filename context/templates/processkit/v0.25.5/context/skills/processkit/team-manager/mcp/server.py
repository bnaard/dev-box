#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit team-manager MCP server.

Owns TeamMember lifecycle, a curated international name pool, memory-tree
scaffolding, A2A Agent Card export/import, and 10 consistency checks.
Replaces the deprecated actor-profile server (DEC-20260422_0233-SpryTulip).

Tools
-----

Lifecycle:
    create_team_member(name, type, slug, default_role?, default_seniority?,
                       personality?, email?, handle?, joined_at?) -> {id, path}
    get_team_member(id) -> {...} | {error}
    list_team_members(type?, active_only?, role?, limit?) -> [members]
    get_active_interlocutor(scope?) -> {configured, interlocutor?}
    get_interlocutor_runtime_binding(scope?, observed_model?, observed_effort?,
                                     task_hints?) -> {configured, binding?}
    set_active_interlocutor(id, scope?) -> {ok, interlocutor}
    update_team_member(id, **fields) -> {ok, id, updated}
    deactivate_team_member(id, left_at?) -> {ok, id, left_at}
    reactivate_team_member(id) -> {ok, id}

Name pool:
    reserve_name(name, team_member_slug) -> {ok, name, slug}
    release_name(name) -> {ok, name}
    list_available_names(kind?) -> [names]
    suggest_name(kind?) -> {name, kind}

Memory tree:
    init_memory_tree(slug) -> {slug, path, created}

Export/import:
    export_team_member(slug, output_path?) -> {path, redacted}
    export_claude_subagent(slug, output_dir?, overwrite?, model_policy?) -> {path}
    export_claude_subagents(output_dir?, active_only?, include_humans?,
                            overwrite?, model_policy?) -> {results}
    import_team_member(tarball_path) -> {slug, path}

Consistency:
    check_consistency(slug) -> {slug, findings}
    check_all_consistency() -> {members: {...}, summary}
"""
from __future__ import annotations

import datetime as _dt
import importlib.util
import json
import os
import re
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

# Expose scripts/ for reuse of helper modules
_SCRIPTS_DIR = Path(__file__).resolve().parent.parent / "scripts"
if _SCRIPTS_DIR.is_dir():
    sys.path.insert(0, str(_SCRIPTS_DIR))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import config, entity, index, log, paths, schema  # noqa: E402

import consistency as _consistency  # noqa: E402
import memory_tree as _memory_tree  # noqa: E402
import name_pool as _name_pool  # noqa: E402
import export_import as _export_import  # noqa: E402


server = FastMCP("processkit-team-manager")

_VALID_TYPES = {"human", "ai-agent", "service"}
_VALID_SENIORITY = {"junior", "specialist", "expert", "senior", "principal"}
_UPDATABLE_FIELDS = {
    "name", "email", "handle", "default_role", "default_seniority",
    "personality", "memory", "relationships", "exportable",
    "export_policy", "active", "joined_at", "left_at",
}

_SLUG_RE = __import__("re").compile(r"^[a-z][a-z0-9-]*$")
_CLAUDE_AGENT_NAME_RE = re.compile(r"^[a-z][a-z0-9-]*$")
_MODEL_RESOLVER = None
_MODEL_RESOLVER_ERROR: str | None = None
_CLAUDE_MODEL_POLICIES = {"inherit", "resolved"}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _tm_dir(root: Path, slug: str) -> Path:
    return paths.context_dir("TeamMember", root) / slug


def _tm_entity_path(root: Path, slug: str) -> Path:
    return _tm_dir(root, slug) / "team-member.md"


def _load_tm_by_slug(root: Path, slug: str) -> entity.Entity | None:
    p = _tm_entity_path(root, slug)
    if p.is_file():
        return entity.load(p)
    return None


def _load_tm(root: Path, id_or_slug: str) -> entity.Entity | None:
    slug = id_or_slug[len("TEAMMEMBER-"):] if id_or_slug.startswith("TEAMMEMBER-") else id_or_slug
    ent = _load_tm_by_slug(root, slug)
    if ent is not None:
        return ent
    # Fallback: try index
    try:
        db = index.open_db()
        try:
            row = index.get_entity(db, id_or_slug if id_or_slug.startswith("TEAMMEMBER-") else f"TEAMMEMBER-{slug}")
        finally:
            db.close()
        if row and row.get("path"):
            return entity.load(row["path"])
    except Exception:
        pass
    return None


def _pool_path() -> Path:
    return Path(__file__).resolve().parent.parent / "data" / "name-pool.yaml"


def _identity_path(root: Path) -> Path:
    return root / "context" / "team" / "session-identity.json"


def _team_member_summary(ent: entity.Entity) -> dict:
    spec = ent.spec or {}
    role = spec.get("default_role") or "ephemeral-role"
    seniority = spec.get("default_seniority") or "unspecified"
    return {
        "id": ent.id,
        "type": spec.get("type"),
        "name": spec.get("name"),
        "slug": spec.get("slug"),
        "default_role": spec.get("default_role"),
        "default_seniority": spec.get("default_seniority"),
        "active": spec.get("active", True),
        "label": f"{spec.get('name')} ({ent.id}; {role}/{seniority})",
        "speaker_prefix": f"{spec.get('name')} [{ent.id}]",
    }


def _read_json(path: Path) -> dict:
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def _write_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _load_model_resolver():
    global _MODEL_RESOLVER, _MODEL_RESOLVER_ERROR
    if _MODEL_RESOLVER is not None:
        return _MODEL_RESOLVER
    if _MODEL_RESOLVER_ERROR is not None:
        return None

    resolver_path = (
        Path(__file__).resolve().parents[2]
        / "model-recommender"
        / "scripts"
        / "resolver.py"
    )
    try:
        spec = importlib.util.spec_from_file_location(
            "processkit_model_recommender_resolver",
            resolver_path,
        )
        if spec is None or spec.loader is None:
            raise RuntimeError(f"could not load import spec for {resolver_path}")
        module = importlib.util.module_from_spec(spec)
        sys.modules.setdefault("processkit_model_recommender_resolver", module)
        spec.loader.exec_module(module)
        _MODEL_RESOLVER = module
        return module
    except Exception as exc:
        _MODEL_RESOLVER_ERROR = str(exc)
        return None


def _strip_frontmatter(text: str) -> str:
    if not text.startswith("---\n"):
        return text.strip()
    end = text.find("\n---\n", 4)
    if end == -1:
        return text.strip()
    return text[end + 5:].strip()


def _yaml_scalar(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def _normalize_runtime_name(value: str | None) -> str:
    if not value:
        return ""
    return str(value).strip().lower().replace("_", "-")


def _harness_binding_mode(harnesses: list[str]) -> dict:
    normalized = [_normalize_runtime_name(h) for h in harnesses]
    primary_agent = "claude" in normalized
    launch_conformance = bool(
        {"claude", "codex", "gemini", "aider", "continue", "cursor",
         "opencode", "hermes"} & set(normalized)
    )
    if primary_agent:
        mode = "primary-agent"
        reason = "harness supports project subagent files with model metadata"
    elif launch_conformance:
        mode = "launch-conform"
        reason = "harness can be launched or configured with a model externally"
    else:
        mode = "identity-only"
        reason = "no supported primary-agent or launch-time model interface found"
    return {
        "mode": mode,
        "primary_agent_supported": primary_agent,
        "launch_conformance_supported": launch_conformance,
        "subagent_mcp_supported": False,
        "subagent_mcp_reason": (
            "disabled until MCP server lifecycle is stable in subagent sessions"
        ),
        "reason": reason,
    }


def _candidate_summary(candidate) -> dict:
    return {
        "model_id": candidate.model_id,
        "version_id": candidate.version_id,
        "effort": candidate.effort,
        "rank": candidate.rank,
        "source_layer": candidate.source_layer,
        "rationale": candidate.rationale,
        "profile_id": candidate.profile_id,
        "profile_rank": candidate.profile_rank,
    }


def _runtime_mismatch(
    resolved: dict | None,
    observed_model: str | None,
    observed_effort: str | None,
) -> dict:
    if not observed_model and not observed_effort:
        return {
            "known": False,
            "severity": "unknown",
            "model": None,
            "effort": None,
            "message": "observed harness runtime was not supplied by caller",
        }
    if not resolved:
        return {
            "known": True,
            "severity": "warn",
            "model": None,
            "effort": None,
            "message": "no resolved processkit runtime is available for comparison",
        }

    resolved_model = _normalize_runtime_name(
        resolved.get("vendor_model_id")
        or resolved.get("profile_id")
        or resolved.get("model_id")
    )
    observed_model_norm = _normalize_runtime_name(observed_model)
    model_match = None
    if observed_model_norm:
        model_match = (
            observed_model_norm == resolved_model
            or observed_model_norm in resolved.get("aliases", [])
        )

    resolved_effort = _normalize_runtime_name(resolved.get("effort"))
    observed_effort_norm = _normalize_runtime_name(observed_effort)
    effort_match = None
    if observed_effort_norm:
        effort_match = observed_effort_norm == resolved_effort

    has_mismatch = model_match is False or effort_match is False
    return {
        "known": True,
        "severity": "warn" if has_mismatch else "info",
        "model": model_match,
        "effort": effort_match,
        "message": (
            "observed harness runtime differs from processkit binding"
            if has_mismatch else
            "observed harness runtime matches supplied processkit fields"
        ),
    }


def _model_spec_from_id(model_id: str) -> dict | None:
    try:
        db = index.open_db()
        try:
            row = index.get_entity(db, model_id)
        finally:
            db.close()
        if row and row.get("path"):
            ent = entity.load(row["path"])
            return ent.spec or {}
    except Exception:
        pass

    root = paths.find_project_root()
    path = root / "context" / "artifacts" / f"{model_id}.md"
    if path.is_file():
        try:
            return entity.load(path).spec or {}
        except Exception:
            return None
    return None


def _version_summary(model: dict | None, version_id: str | None) -> dict:
    if not model or not version_id:
        return {}
    for version in model.get("versions") or []:
        if str(version.get("version_id")) == str(version_id):
            return {
                "version_id": version.get("version_id"),
                "status": version.get("status"),
                "lifecycle": version.get("lifecycle"),
                "vendor_model_id": version.get("vendor_model_id"),
            }
    return {}


def _model_aliases(
    model: dict | None,
    model_id: str,
    version_id: str | None,
) -> list[str]:
    aliases = {_normalize_runtime_name(model_id)}
    if model:
        provider = _normalize_runtime_name(model.get("provider"))
        family = _normalize_runtime_name(model.get("family"))
        if provider and family:
            aliases.add(f"{provider}-{family}")
            aliases.add(f"model-{provider}-{family}")
        if family:
            aliases.add(family)
            if version_id:
                aliases.add(f"{family}-{version_id}")
        for profile_id in model.get("profile_ids") or []:
            aliases.add(_normalize_runtime_name(profile_id))
        for version in model.get("versions") or []:
            vendor_model_id = version.get("vendor_model_id")
            if vendor_model_id:
                aliases.add(_normalize_runtime_name(vendor_model_id))
    return sorted(a for a in aliases if a)


def _resolved_runtime_for(
    ent: entity.Entity,
    scope: str | None,
    task_hints: dict | None,
) -> tuple[dict | None, list[dict], str | None]:
    resolver = _load_model_resolver()
    if resolver is None:
        return None, [], _MODEL_RESOLVER_ERROR or "model resolver unavailable"

    spec = ent.spec or {}
    role = spec.get("default_role")
    if not role:
        return None, [], "team-member has no default_role"
    try:
        candidates, trace = resolver.resolve(
            role=role,
            seniority=spec.get("default_seniority"),
            team_member=ent.id,
            scope=scope,
            task_hints=task_hints or {},
            explain=True,
        )
        if not candidates:
            return None, trace, "model resolver returned no candidates"
        candidate = candidates[0]
        model = _model_spec_from_id(candidate.model_id)
        version = _version_summary(model, candidate.version_id)
        summary = _candidate_summary(candidate)
        summary["provider"] = model.get("provider") if model else None
        summary["family"] = model.get("family") if model else None
        summary["version_status"] = version.get("status")
        summary["version_lifecycle"] = version.get("lifecycle")
        summary["vendor_model_id"] = version.get("vendor_model_id")
        summary["aliases"] = _model_aliases(
            model,
            candidate.model_id,
            candidate.version_id,
        )
        return summary, trace, None
    except Exception as exc:
        return None, [], str(exc)


def _runtime_binding_for(
    ent: entity.Entity,
    scope: str | None,
    observed_model: str | None,
    observed_effort: str | None,
    task_hints: dict | None,
) -> dict:
    resolver = _load_model_resolver()
    if resolver is not None:
        try:
            runtime_context = resolver._runtime_context(task_hints or {})
        except Exception as exc:
            runtime_context = {"error": str(exc), "harnesses": []}
    else:
        runtime_context = {
            "error": _MODEL_RESOLVER_ERROR or "model resolver unavailable",
            "harnesses": [],
        }

    resolved, trace, error = _resolved_runtime_for(ent, scope, task_hints)
    capabilities = _harness_binding_mode(runtime_context.get("harnesses") or [])
    observed = {
        "model": observed_model,
        "effort": observed_effort,
        "source": "caller" if observed_model or observed_effort else "unknown",
    }
    return {
        "policy": "capability-negotiated",
        "mode": capabilities["mode"],
        "capabilities": capabilities,
        "runtime_context": {
            "harnesses": runtime_context.get("harnesses") or [],
            "allowed_providers": runtime_context.get("allowed_providers") or [],
            "preferred_providers": runtime_context.get("preferred_providers") or [],
            "provider_source": runtime_context.get("provider_source"),
            "error": runtime_context.get("error"),
        },
        "resolved": resolved,
        "resolution_error": error,
        "trace": trace,
        "observed": observed,
        "mismatch": _runtime_mismatch(resolved, observed_model, observed_effort),
        "notes": [
            "processkit cannot hot-swap the current primary harness session",
            "identity-only fallback remains valid when runtime control is absent",
        ],
    }


def _claude_frontmatter_model(
    ent: entity.Entity,
    model_policy: str,
) -> tuple[list[str], dict | None]:
    if model_policy not in _CLAUDE_MODEL_POLICIES:
        return [], {
            "error": f"model_policy must be one of {sorted(_CLAUDE_MODEL_POLICIES)}"
        }
    if model_policy == "inherit":
        return ["model: inherit"], None

    binding = _runtime_binding_for(
        ent=ent,
        scope=None,
        observed_model=None,
        observed_effort=None,
        task_hints={"available_providers": ["anthropic"]},
    )
    resolved = binding.get("resolved")
    if not resolved:
        return [], {
            "error": "could not resolve Claude model for TeamMember",
            "details": binding.get("resolution_error"),
        }
    if _normalize_runtime_name(resolved.get("provider")) != "anthropic":
        return [], {
            "error": "resolved model is not an Anthropic Claude model",
            "resolved": resolved,
        }
    family = _normalize_runtime_name(resolved.get("family"))
    version = str(resolved.get("version_id") or "").strip()
    if not family:
        return [], {"error": "resolved Anthropic model has no family"}
    model_name = f"{family}-{version}" if version else family
    lines = [f"model: {model_name}"]
    if resolved.get("effort"):
        lines.append(f"effort: {resolved['effort']}")
    return lines, {"binding": binding}


def _claude_agent_prompt(root: Path, ent: entity.Entity) -> str:
    spec = ent.spec or {}
    slug = spec.get("slug") or ent.id.removeprefix("TEAMMEMBER-")
    persona_path = _tm_dir(root, slug) / "persona.md"
    persona = ""
    if persona_path.is_file():
        persona = _strip_frontmatter(persona_path.read_text(encoding="utf-8"))

    role = spec.get("default_role") or "ephemeral-role"
    seniority = spec.get("default_seniority") or "unspecified"
    lines = [
        f"You are {spec.get('name')}, processkit TeamMember {ent.id}.",
        "",
        "Processkit identity:",
        f"- Type: {spec.get('type')}",
        f"- Canonical slug: {slug}",
        f"- Default role: {role}",
        f"- Default seniority: {seniority}",
        "",
        "Use this subagent when the requested work matches this TeamMember's",
        "role, persona, or durable project memory. Treat this file as a",
        "Claude Code adapter over processkit's provider-neutral TeamMember",
        "model, not as the canonical identity record.",
        "",
        "Do not claim to be the session's active interlocutor unless",
        "`team-manager.get_active_interlocutor` returns this TeamMember.",
    ]
    if persona:
        lines.extend(["", "Persona:", "", persona])
    return "\n".join(lines).rstrip() + "\n"


def _write_claude_agent(
    root: Path,
    ent: entity.Entity,
    dest_dir: Path,
    overwrite: bool,
    model_policy: str = "inherit",
) -> dict:
    spec = ent.spec or {}
    slug = spec.get("slug") or ent.id.removeprefix("TEAMMEMBER-")
    if not _CLAUDE_AGENT_NAME_RE.match(slug):
        return {"id": ent.id, "slug": slug, "error": "slug is not a valid Claude Code agent name"}
    if spec.get("exportable") is False:
        return {"id": ent.id, "slug": slug, "skipped": "exportable=false"}

    dest_dir.mkdir(parents=True, exist_ok=True)
    dest = dest_dir / f"{slug}.md"
    if dest.exists() and not overwrite:
        return {"id": ent.id, "slug": slug, "path": str(dest), "skipped": "exists"}

    model_lines, model_details = _claude_frontmatter_model(ent, model_policy)
    if model_details and model_details.get("error"):
        return {"id": ent.id, "slug": slug, **model_details}

    role = spec.get("default_role") or "ephemeral-role"
    seniority = spec.get("default_seniority") or "unspecified"
    description = (
        f"Use {spec.get('name')} for processkit work matching "
        f"{role}/{seniority}; derived from TeamMember {ent.id}."
    )
    body = [
        "---",
        f"name: {slug}",
        f"description: {_yaml_scalar(description)}",
        *model_lines,
        "---",
        "",
        _claude_agent_prompt(root, ent).rstrip(),
        "",
    ]
    dest.write_text("\n".join(body), encoding="utf-8")
    result = {
        "id": ent.id,
        "slug": slug,
        "path": str(dest),
        "written": True,
        "model_policy": model_policy,
    }
    if model_details:
        result.update(model_details)
    return result


def _claude_output_dir(root: Path, output_dir: str | None) -> Path | dict:
    if output_dir:
        raw = Path(output_dir)
        dest = raw if raw.is_absolute() else root / raw
    else:
        dest = root / ".claude" / "agents"
    resolved_root = root.resolve()
    resolved_dest = dest.resolve()
    if not resolved_dest.is_relative_to(resolved_root):
        return {"error": f"output_dir must stay under project root: {root}"}
    return dest


# ---------------------------------------------------------------------------
# Lifecycle tools
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def create_team_member(
    name: str,
    type: str,
    slug: str,
    default_role: str | None = None,
    default_seniority: str | None = None,
    personality: dict | None = None,
    email: str | None = None,
    handle: str | None = None,
    joined_at: str | None = None,
) -> dict:
    """Create a new TeamMember entity.

    Parameters
    ----------
    name:               display name (e.g. "Atlas" or "Alice Chen")
    type:               "human", "ai-agent", or "service"
    slug:               canonical kebab-case slug; becomes TEAMMEMBER-<slug>
    default_role:       optional ROLE-<id>
    default_seniority:  optional enum (junior|specialist|expert|senior|principal)
    personality:        optional dict (communication_style, voice, ...)
    email:              optional; humans only
    handle:             optional; GitHub/Slack handle
    joined_at:          ISO datetime; defaults to now

    For type=ai-agent, slug must match a name in the name pool and will be
    auto-reserved on success.
    """
    if type not in _VALID_TYPES:
        return {"error": f"invalid type {type!r}; must be one of {sorted(_VALID_TYPES)}"}
    if default_seniority is not None and default_seniority not in _VALID_SENIORITY:
        return {"error": f"invalid seniority {default_seniority!r}"}
    if not _SLUG_RE.match(slug):
        return {"error": f"invalid slug {slug!r}; must match ^[a-z][a-z0-9-]*$"}

    root = paths.find_project_root()
    tm_path = _tm_entity_path(root, slug)
    if tm_path.is_file():
        return {"error": f"team-member {slug!r} already exists at {tm_path}"}

    # For ai-agent, verify slug is reserveable from pool
    pool_reservation_done = False
    if type == "ai-agent":
        pool = _name_pool.load_pool(_pool_path())
        # slug derived from a pool name — lower-cased
        pool_names = _name_pool.all_names(pool)
        match = next((n for n in pool_names if n.lower() == slug.lower()), None)
        if match is None:
            return {
                "error": f"slug {slug!r} is not in the name pool; "
                "call list_available_names(kind?) or suggest_name(kind?) first"
            }
        reserved = pool["spec"].get("reserved") or {}
        if match in reserved:
            return {"error": f"name {match!r} is already reserved by slug {reserved[match]!r}"}
        _name_pool.reserve(_pool_path(), match, slug)
        pool_reservation_done = True

    new_id = f"TEAMMEMBER-{slug}"

    spec: dict = {
        "type": type,
        "name": name,
        "slug": slug,
        "active": True,
        "joined_at": joined_at or _now_iso(),
    }
    if email:
        spec["email"] = email
    if handle:
        spec["handle"] = handle
    if default_role:
        spec["default_role"] = default_role
    if default_seniority:
        spec["default_seniority"] = default_seniority
    if personality:
        spec["personality"] = dict(personality)

    errors = schema.validate_spec("TeamMember", spec)
    if errors:
        if pool_reservation_done:
            _name_pool.release(_pool_path(), match)
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("TeamMember", new_id, spec)
    ent.write(tm_path)

    try:
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
    except Exception:
        pass

    log.log_side_effect(
        "TeamMember", new_id, "team_member.created",
        f"Created TeamMember {new_id!r}: {name!r} ({type})",
        root=root,
        actor=new_id,
    )
    return {"id": new_id, "path": str(tm_path), "type": type, "name": name, "slug": slug}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_team_member(id: str) -> dict:
    """Return the full TeamMember entity by ID or slug."""
    root = paths.find_project_root()
    ent = _load_tm(root, id)
    if ent is None:
        return {"error": f"team-member {id!r} not found"}
    return {
        "id": ent.id,
        "type": ent.spec.get("type"),
        "name": ent.spec.get("name"),
        "slug": ent.spec.get("slug"),
        "email": ent.spec.get("email"),
        "handle": ent.spec.get("handle"),
        "default_role": ent.spec.get("default_role"),
        "default_seniority": ent.spec.get("default_seniority"),
        "personality": ent.spec.get("personality", {}),
        "memory": ent.spec.get("memory", {}),
        "relationships": ent.spec.get("relationships", []),
        "active": ent.spec.get("active", True),
        "joined_at": ent.spec.get("joined_at"),
        "left_at": ent.spec.get("left_at"),
        "path": str(ent.path) if ent.path else None,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_team_members(
    type: str | None = None,
    active_only: bool = True,
    role: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """List TeamMembers with optional filters by type, active state, default_role."""
    root = paths.find_project_root()
    tm_root = paths.context_dir("TeamMember", root)
    if not tm_root.is_dir():
        return []
    out: list[dict] = []
    for child in sorted(tm_root.iterdir()):
        if not child.is_dir():
            continue
        p = child / "team-member.md"
        if not p.is_file():
            continue
        try:
            ent = entity.load(p)
        except Exception:
            continue
        if ent.kind != "TeamMember":
            continue
        spec = ent.spec or {}
        if type and spec.get("type") != type:
            continue
        if active_only and not spec.get("active", True):
            continue
        if role and spec.get("default_role") != role:
            continue
        out.append({
            "id": ent.id,
            "type": spec.get("type"),
            "name": spec.get("name"),
            "slug": spec.get("slug"),
            "default_role": spec.get("default_role"),
            "default_seniority": spec.get("default_seniority"),
            "active": spec.get("active", True),
            "path": str(ent.path) if ent.path else None,
        })
        if len(out) >= limit:
            break
    return out


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_active_interlocutor(scope: str | None = None) -> dict:
    """Return the configured active interlocutor for harness sessions.

    This is session identity, not lifecycle state. `TeamMember.spec.active`
    still means whether a TeamMember can participate at all.
    """
    root = paths.find_project_root()
    identity_file = _identity_path(root)
    configured_scope = scope or "default"
    try:
        data = _read_json(identity_file)
    except (OSError, json.JSONDecodeError) as e:
        return {"error": f"could not read {identity_file}: {e}"}

    entries = data.get("scopes") or {}
    entry = entries.get(configured_scope) or entries.get("default")
    if not entry:
        return {
            "configured": False,
            "scope": configured_scope,
            "path": str(identity_file),
            "message": "no active interlocutor configured; call set_active_interlocutor",
        }

    member_id = entry.get("team_member")
    ent = _load_tm(root, member_id or "")
    if ent is None:
        return {
            "configured": True,
            "scope": configured_scope,
            "path": str(identity_file),
            "error": f"configured team-member {member_id!r} not found",
        }

    summary = _team_member_summary(ent)
    return {
        "configured": True,
        "scope": configured_scope,
        "path": str(identity_file),
        "updated_at": entry.get("updated_at"),
        "interlocutor": summary,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_interlocutor_runtime_binding(
    scope: str | None = None,
    observed_model: str | None = None,
    observed_effort: str | None = None,
    task_hints: dict | None = None,
) -> dict:
    """Return active interlocutor identity plus runtime binding status.

    `observed_model` and `observed_effort` are caller-supplied facts from
    the current harness. processkit cannot read or hot-swap the already
    running primary model, so mismatch reporting is informational.
    """
    active = get_active_interlocutor(scope)
    if active.get("error") or not active.get("configured"):
        return active

    root = paths.find_project_root()
    member_id = (active.get("interlocutor") or {}).get("id")
    ent = _load_tm(root, member_id or "")
    if ent is None:
        return {
            "configured": True,
            "scope": active.get("scope"),
            "error": f"configured team-member {member_id!r} not found",
        }

    return {
        **active,
        "binding": _runtime_binding_for(
            ent=ent,
            scope=scope,
            observed_model=observed_model,
            observed_effort=observed_effort,
            task_hints=task_hints,
        ),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def set_active_interlocutor(id: str, scope: str | None = None) -> dict:
    """Set the active interlocutor shown at harness/session start."""
    root = paths.find_project_root()
    ent = _load_tm(root, id)
    if ent is None:
        return {"error": f"team-member {id!r} not found"}
    if not ent.spec.get("active", True):
        return {"error": f"team-member {ent.id!r} is inactive"}

    identity_file = _identity_path(root)
    try:
        data = _read_json(identity_file)
    except (OSError, json.JSONDecodeError) as e:
        return {"error": f"could not read {identity_file}: {e}"}

    configured_scope = scope or "default"
    data.setdefault("version", 1)
    data.setdefault("scopes", {})
    data["scopes"][configured_scope] = {
        "team_member": ent.id,
        "updated_at": _now_iso(),
        "binding_policy": "capability-negotiated",
        "subagent_mcp_supported": False,
    }
    _write_json(identity_file, data)

    log.log_side_effect(
        "TeamMember", ent.id, "team_member.active_interlocutor_set",
        f"Set active interlocutor for scope {configured_scope!r} to {ent.id!r}",
        root=root,
        actor=ent.id,
    )
    return {
        "ok": True,
        "scope": configured_scope,
        "path": str(identity_file),
        "interlocutor": _team_member_summary(ent),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def update_team_member(
    id: str,
    name: str | None = None,
    email: str | None = None,
    handle: str | None = None,
    default_role: str | None = None,
    default_seniority: str | None = None,
    personality: dict | None = None,
    memory: dict | None = None,
    relationships: list | None = None,
    exportable: bool | None = None,
    export_policy: dict | None = None,
    active: bool | None = None,
) -> dict:
    """Update fields on an existing TeamMember. Only supplied fields change."""
    root = paths.find_project_root()
    ent = _load_tm(root, id)
    if ent is None:
        return {"error": f"team-member {id!r} not found"}

    updated: list[str] = []
    locals_map = {
        "name": name, "email": email, "handle": handle,
        "default_role": default_role, "default_seniority": default_seniority,
        "personality": personality, "memory": memory,
        "relationships": relationships, "exportable": exportable,
        "export_policy": export_policy, "active": active,
    }
    for k, v in locals_map.items():
        if v is None:
            continue
        if k == "default_seniority" and v not in _VALID_SENIORITY:
            return {"error": f"invalid seniority {v!r}"}
        ent.spec[k] = v
        updated.append(k)

    if not updated:
        return {"ok": True, "id": ent.id, "updated": []}

    errors = schema.validate_spec("TeamMember", ent.spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent.write()
    try:
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
    except Exception:
        pass
    return {"ok": True, "id": ent.id, "updated": updated}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=True,
    openWorldHint=False,
))
def deactivate_team_member(id: str, left_at: str | None = None) -> dict:
    """Mark a TeamMember inactive. Sets active=false and left_at."""
    root = paths.find_project_root()
    ent = _load_tm(root, id)
    if ent is None:
        return {"error": f"team-member {id!r} not found"}
    ent.spec["active"] = False
    ent.spec["left_at"] = left_at or _now_iso()
    ent.write()
    try:
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
    except Exception:
        pass
    log.log_side_effect(
        "TeamMember", ent.id, "team_member.deactivated",
        f"Deactivated TeamMember {ent.id!r}",
        root=root,
        actor=ent.id,
    )
    return {"ok": True, "id": ent.id, "left_at": ent.spec["left_at"]}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def reactivate_team_member(id: str) -> dict:
    """Reactivate a deactivated TeamMember (active=true, clears left_at)."""
    root = paths.find_project_root()
    ent = _load_tm(root, id)
    if ent is None:
        return {"error": f"team-member {id!r} not found"}
    ent.spec["active"] = True
    ent.spec.pop("left_at", None)
    ent.write()
    try:
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
    except Exception:
        pass
    log.log_side_effect(
        "TeamMember", ent.id, "team_member.reactivated",
        f"Reactivated TeamMember {ent.id!r}",
        root=root,
        actor=ent.id,
    )
    return {"ok": True, "id": ent.id}


# ---------------------------------------------------------------------------
# Name-pool tools
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def reserve_name(name: str, team_member_slug: str) -> dict:
    """Reserve a pool name for a team-member slug."""
    try:
        _name_pool.reserve(_pool_path(), name, team_member_slug)
    except _name_pool.NamePoolError as e:
        return {"error": str(e)}
    return {"ok": True, "name": name, "slug": team_member_slug}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=True,
    openWorldHint=False,
))
def release_name(name: str) -> dict:
    """Release a pool name reservation."""
    try:
        _name_pool.release(_pool_path(), name)
    except _name_pool.NamePoolError as e:
        return {"error": str(e)}
    return {"ok": True, "name": name}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_available_names(kind: str | None = None) -> list[str]:
    """List pool names that are not reserved. Optional kind filter:
    feminine | masculine | neutral."""
    pool = _name_pool.load_pool(_pool_path())
    return _name_pool.available(pool, kind)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def suggest_name(kind: str | None = None) -> dict:
    """Suggest one available pool name at random. Does not reserve."""
    pool = _name_pool.load_pool(_pool_path())
    chosen = _name_pool.suggest(pool, kind)
    if chosen is None:
        return {"error": f"no available names in pool (kind={kind!r})"}
    bucket = _name_pool.kind_of(pool, chosen)
    return {"name": chosen, "kind": bucket}


# ---------------------------------------------------------------------------
# Memory-tree tool
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def init_memory_tree(slug: str) -> dict:
    """Scaffold the memory-tree layout for an existing team-member slug.

    Creates tier subdirectories with .gitkeep, plus persona.md, card.json,
    and team-member.md (if the entity does not already exist). Idempotent.
    """
    root = paths.find_project_root()
    tm_dir = _tm_dir(root, slug)
    created = _memory_tree.init_tree(tm_dir, slug, _load_tm_by_slug(root, slug))
    return {"slug": slug, "path": str(tm_dir), "created": created}


# ---------------------------------------------------------------------------
# Export / import
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def export_team_member(slug: str, output_path: str | None = None) -> dict:
    """Build a tar.gz bundle for a team-member.

    Includes persona.md, card.json, team-member.md, and knowledge/,
    skills/, lessons/. Excludes journal/, relations/, private/. Redacts
    memory files whose frontmatter declares sensitivity=confidential|pii.
    """
    root = paths.find_project_root()
    tm_dir = _tm_dir(root, slug)
    if not tm_dir.is_dir():
        return {"error": f"team-member directory not found: {tm_dir}"}
    dest = Path(output_path) if output_path else Path.cwd() / f"{slug}-export-{_dt.date.today().isoformat()}.tar.gz"
    summary = _export_import.export(tm_dir, dest)
    log.log_side_effect(
        "TeamMember", f"TEAMMEMBER-{slug}", "team_member.exported",
        f"Exported TeamMember {slug!r} to {dest}",
        root=root,
        actor=f"TEAMMEMBER-{slug}",
    )
    return {"path": str(dest), "redacted": summary["redacted"], "included": summary["included"]}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def export_claude_subagent(
    slug: str,
    output_dir: str | None = None,
    overwrite: bool = True,
    model_policy: str = "inherit",
) -> dict:
    """Export one TeamMember as a Claude Code project subagent adapter.

    Writes `.claude/agents/<slug>.md` by default. The generated file uses
    `model: inherit` by default and omits `tools`, so Claude Code inherits
    the main session tool permissions. Set `model_policy="resolved"` to
    emit a Claude model line from processkit routing when an Anthropic
    candidate can be resolved.
    """
    root = paths.find_project_root()
    ent = _load_tm(root, slug)
    if ent is None:
        return {"error": f"team-member {slug!r} not found"}
    dest_dir = _claude_output_dir(root, output_dir)
    if isinstance(dest_dir, dict):
        return dest_dir
    result = _write_claude_agent(root, ent, dest_dir, overwrite, model_policy)
    if result.get("written"):
        log.log_side_effect(
            "TeamMember", ent.id, "team_member.claude_subagent_exported",
            f"Exported TeamMember {ent.id!r} as Claude Code subagent",
            root=root,
            actor=ent.id,
        )
    return result


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def export_claude_subagents(
    output_dir: str | None = None,
    active_only: bool = True,
    include_humans: bool = False,
    overwrite: bool = True,
    model_policy: str = "inherit",
) -> dict:
    """Export provider-neutral TeamMembers to Claude Code subagent files."""
    root = paths.find_project_root()
    dest_dir = _claude_output_dir(root, output_dir)
    if isinstance(dest_dir, dict):
        return dest_dir
    members = list_team_members(active_only=active_only, limit=1000)
    results: list[dict] = []
    for member in members:
        if member.get("type") == "human" and not include_humans:
            results.append({
                "id": member.get("id"),
                "slug": member.get("slug"),
                "skipped": "human",
            })
            continue
        ent = _load_tm(root, member["id"])
        if ent is None:
            results.append({
                "id": member.get("id"),
                "slug": member.get("slug"),
                "error": "not found",
            })
            continue
        results.append(
            _write_claude_agent(root, ent, dest_dir, overwrite, model_policy)
        )

    written = [r for r in results if r.get("written")]
    if written:
        log.log_side_effect(
            "TeamMember", "TEAMMEMBER", "team_member.claude_subagents_exported",
            f"Exported {len(written)} TeamMember Claude Code subagent adapter(s)",
            root=root,
            actor="system",
        )
    return {"output_dir": str(dest_dir), "count": len(written), "results": results}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def import_team_member(tarball_path: str) -> dict:
    """Import a team-member bundle. Validates schema + signature-field presence."""
    root = paths.find_project_root()
    tm_root = paths.context_dir("TeamMember", root)
    assets = Path(__file__).resolve().parent.parent / "assets"
    try:
        result = _export_import.import_bundle(Path(tarball_path), tm_root, assets)
    except _export_import.ImportError as e:
        return {"error": str(e)}
    slug = result["slug"]
    log.log_side_effect(
        "TeamMember", f"TEAMMEMBER-{slug}", "team_member.imported",
        f"Imported TeamMember {slug!r} from {tarball_path}",
        root=root,
        actor=f"TEAMMEMBER-{slug}",
    )
    return result


# ---------------------------------------------------------------------------
# Consistency
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def check_consistency(slug: str) -> dict:
    """Run the 10 consistency checks on a single team-member."""
    root = paths.find_project_root()
    tm_dir = _tm_dir(root, slug)
    assets = Path(__file__).resolve().parent.parent / "assets"
    findings = _consistency.check_one(root, tm_dir, slug, _pool_path(), assets)
    return {"slug": slug, "findings": findings}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def check_all_consistency() -> dict:
    """Run the 10 consistency checks across every team-member."""
    root = paths.find_project_root()
    tm_root = paths.context_dir("TeamMember", root)
    assets = Path(__file__).resolve().parent.parent / "assets"
    return _consistency.check_all(root, tm_root, _pool_path(), assets)


if __name__ == "__main__":
    server.run(transport="stdio")
