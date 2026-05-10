#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""processkit task-router MCP server.

Tools:

    route_task(task_description)
        -> {skill, skill_description_excerpt, process_override?,
            process_override_status?, server, tool, tool_qualified, domain_group, confidence,
            routing_basis, candidate_tools[], also_consider_group?}

Two-phase heuristic routing (no LLM call required):
  Phase 1 — keyword match against domain groups (cheap classifier).
  Phase 2 — token overlap scoring within the matched group's tool list.
  Fallback — skill-finder trigger-phrase scan for cross-domain tasks.

Architecture note: each phase eliminates ~90 % of the candidate space
before passing a constrained set to the next stage (cascaded coarse-to-
fine pattern from MasRouter / Elastic Path). Routing never calls an LLM;
the caller can escalate to LLM confirmation when confidence < 0.5.
"""
from __future__ import annotations

import json
import math
import os
import re
import sys
from collections import Counter
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

from processkit import paths  # noqa: E402

import datetime as _dt  # noqa: E402

# ---------------------------------------------------------------------------
# Route-task marker writing (T2.2 — check_route_task_before_agent.py hook)
# ---------------------------------------------------------------------------

_ROUTE_MARKER_SUBDIR = Path("context") / ".state" / "skill-gate"


def _write_route_task_marker(root: Path) -> None:
    """Write a per-turn marker so the Agent-dispatch hook knows route_task ran.

    File: context/.state/skill-gate/route-task-<pid>-<ts_ms>.routed
    Content: JSON with ts (ISO UTC) so the hook can apply a time-window check.
    Errors are silently swallowed — marker writing must not break routing.
    """
    try:
        marker_dir = root / _ROUTE_MARKER_SUBDIR
        marker_dir.mkdir(parents=True, exist_ok=True)
        now = _dt.datetime.now(_dt.timezone.utc)
        ts_ms = int(now.timestamp() * 1000)
        sid = os.environ.get("PROCESSKIT_SESSION_ID", str(os.getpid()))
        marker = marker_dir / f"route-task-{sid}-{ts_ms}.routed"
        marker.write_text(
            json.dumps({"ts": now.isoformat(), "session_id": sid}) + "\n",
            encoding="utf-8",
        )
        # Prune old markers (> 5 min) to keep the dir tidy
        cutoff = now - _dt.timedelta(minutes=5)
        for old in marker_dir.glob("route-task-*.routed"):
            if old == marker:
                continue
            try:
                data = json.loads(old.read_text(encoding="utf-8"))
                ts = _dt.datetime.fromisoformat(data["ts"])
                if ts.tzinfo is None:
                    ts = ts.replace(tzinfo=_dt.timezone.utc)
                if ts < cutoff:
                    old.unlink(missing_ok=True)
            except Exception:
                pass
    except Exception:
        pass

server = FastMCP("processkit-task-router")

# ---------------------------------------------------------------------------
# Domain group registry
# ---------------------------------------------------------------------------
# Maps domain name → {server, skill, keywords, tools[(name, description)]}.
# keywords: used in Phase 1 group scoring.
# tools:    used in Phase 2 tool scoring within the matched group.
#
# Naming convention for tool_qualified: "{server}__{tool_name}"
# (MetaMCP de-facto standard for collision-free aggregation).
# ---------------------------------------------------------------------------

DOMAIN_GROUPS: dict[str, dict] = {
    "workitem": {
        "server": "processkit-workitem-management",
        "skill": "workitem-management",
        "keywords": [
            "work item", "workitem", "ticket", "backlog", "task", "story",
            "bug", "issue", "todo", "sprint", "feature request",
            "create task", "track work", "open ticket",
        ],
        "tools": [
            ("create_workitem",
             "Create a new work item, ticket, or task in the backlog"),
            ("transition_workitem",
             "Move a work item to a new state: start, complete, close,"
             " reopen"),
            ("query_workitems",
             "Search or list work items by state, assignee, or label"),
            ("get_workitem",
             "Retrieve a specific work item by ID"),
            ("link_workitems",
             "Link two work items as blocking, related, or duplicate"),
        ],
    },
    "decision": {
        "server": "processkit-decision-record",
        "skill": "decision-record",
        "keywords": [
            "decision", "decide", "ADR", "architectural decision",
            "trade-off", "alternative", "rationale", "option",
            "record decision", "document decision", "chose",
        ],
        "tools": [
            ("record_decision",
             "Create a new decision record (ADR)"),
            ("get_decision",
             "Retrieve a specific decision by ID"),
            ("query_decisions",
             "Search decision records by state or keyword"),
            ("transition_decision",
             "Accept, supersede, or deprecate a decision"),
            ("supersede_decision",
             "Mark a decision as replaced by a newer one"),
            ("link_decision_to_workitem",
             "Link a decision to a related work item"),
        ],
    },
    "discussion": {
        "server": "processkit-discussion-management",
        "skill": "devils-advocate",
        "keywords": [
            "discuss", "deliberate", "debate", "open question",
            "brainstorm", "second opinion", "perspectives",
            "explore options", "conversation", "structured discussion",
            "open discussion",
        ],
        "tools": [
            ("open_discussion",
             "Open a new structured discussion with a driving question"),
            ("get_discussion",
             "Retrieve a specific discussion by ID"),
            ("list_discussions",
             "List discussions filtered by state or participant"),
            ("transition_discussion",
             "Resolve or close a discussion"),
            ("add_outcome",
             "Record the outcome or conclusion of a discussion"),
        ],
    },
    "event": {
        "server": "processkit-event-log",
        "skill": "event-log",
        "keywords": [
            "log", "audit", "event", "record event", "track activity",
            "append log", "audit trail", "history", "what happened",
            "log event", "write log",
        ],
        "tools": [
            ("log_event",
             "Append a new auditable log entry to the event log"),
            ("query_events",
             "Search log entries by event type, subject, or date range"),
            ("recent_events",
             "Return the most recent N log entries"),
        ],
    },
    "actor": {
        "server": "processkit-actor-profile",
        "skill": "onboarding-guide",
        "keywords": [
            "actor", "person", "team member", "participant",
            "agent profile", "user profile", "who is", "new member",
            "onboard person",
        ],
        "tools": [
            ("create_actor",
             "Create a new actor profile for a team member or agent"),
            ("get_actor",
             "Retrieve an actor profile by ID"),
            ("list_actors",
             "List all active actors in the project"),
            ("update_actor",
             "Update an actor's profile details"),
            ("deactivate_actor",
             "Deactivate an actor who has left the project"),
        ],
    },
    "scope": {
        "server": "processkit-scope-management",
        "skill": "estimation-planning",
        "keywords": [
            "scope", "milestone", "phase", "sprint scope",
            "project scope", "iteration", "roadmap item",
            "define scope", "scope boundary",
        ],
        "tools": [
            ("create_scope",
             "Create a new project scope, milestone, or sprint"),
            ("get_scope",
             "Retrieve a specific scope by ID"),
            ("list_scopes",
             "List all scopes, optionally filtered by state"),
            ("transition_scope",
             "Move a scope to active, completed, or cancelled"),
        ],
    },
    "index": {
        "server": "processkit-index-management",
        "skill": "status-briefing",
        "keywords": [
            "search", "find entity", "query entity", "index", "catalog",
            "discover", "full text search", "what exists", "reindex",
            "stats", "entity count",
        ],
        "tools": [
            ("get_entity",
             "Retrieve any processkit entity by its ID"),
            ("query_entities",
             "Filter entities by kind, state, label, or assignee"),
            ("search_entities",
             "Full-text search across all indexed entities"),
            ("reindex",
             "Rebuild the entity index after hand-edits to entity files"),
            ("stats",
             "Return entity counts and index health statistics"),
        ],
    },
    "skill": {
        "server": "processkit-skill-finder",
        "skill": "skill-finder",
        "keywords": [
            "skill", "which skill", "find skill", "route task",
            "what to do", "workflow", "capability", "skill catalog",
            "session start", "what skill applies",
        ],
        "tools": [
            ("find_skill",
             "Find the processkit skill matching a task description"),
            ("list_skills",
             "List all skills, optionally filtered by category"),
        ],
    },
    "gate": {
        "server": "processkit-gate-management",
        "skill": "tdd-workflow",
        "keywords": [
            "gate", "quality gate", "pass fail", "criterion",
            "threshold", "smoke test", "gate evaluation",
            "green gate", "check gate",
        ],
        "tools": [
            ("create_gate",
             "Create a quality gate with named pass/fail criteria"),
            ("evaluate_gate",
             "Record the result of a gate evaluation: passed or failed"),
            ("get_gate",
             "Retrieve a gate by ID"),
            ("list_gates",
             "List all gates, optionally filtered by state"),
        ],
    },
    "model": {
        "server": "processkit-model-recommender",
        "skill": "model-recommender",
        "keywords": [
            "model", "LLM", "AI model", "recommend model", "which model",
            "which Claude", "model selection", "compare models",
            "provider", "model pricing", "which LLM",
        ],
        "tools": [
            ("get_profile",
             "Get the capability profile for a specific AI model"),
            ("list_models",
             "List all models in the registry"),
            ("compare_models",
             "Compare two or more models across capability dimensions"),
            ("query_models",
             "Filter models by provider, capability, or context length"),
        ],
    },
    "id": {
        "server": "processkit-id-management",
        "skill": "workitem-management",
        "keywords": [
            "ID", "identifier", "generate ID", "unique ID", "new ID",
            "entity ID", "format ID", "pascal ID",
        ],
        "tools": [
            ("generate_id",
             "Generate a new processkit ID with pascal word-pair and"
             " datetime prefix"),
            ("validate_id",
             "Check whether a string is a valid processkit ID"),
            ("format_info",
             "Return the format specification for a given ID kind"),
            ("list_used_ids",
             "Return all IDs already assigned in this project"),
        ],
    },
    "role": {
        "server": "processkit-role-management",
        "skill": "onboarding-guide",
        "keywords": [
            "role", "responsibility", "RACI", "owner", "permission",
            "access", "assign role", "who owns", "role definition",
        ],
        "tools": [
            ("create_role",
             "Create a new role definition with responsibilities"),
            ("get_role",
             "Retrieve a role by ID"),
            ("list_roles",
             "List all roles in the project"),
            ("link_role_to_actor",
             "Assign a role to an actor"),
            ("update_role",
             "Update a role's definition or responsibilities"),
        ],
    },
    "retro": {
        "server": "processkit-event-log",
        "skill": "retrospective",
        "keywords": [
            "retrospective", "retro", "post-release", "post-mortem",
            "blameless", "what held", "what slipped", "release review",
            "lessons learned", "cycle review", "pk-retro",
        ],
        "tools": [
            ("log_event",
             "Emit the retro.completed LogEntry after the Artifact is saved"),
            ("query_events",
             "Query release.published and doctor.report events for signals"),
        ],
    },
    "binding": {
        "server": "processkit-binding-management",
        "skill": "workitem-management",
        "keywords": [
            "binding", "assign actor", "link actor to role",
            "role binding", "who is assigned", "who is responsible",
            "assignment", "active binding",
        ],
        "tools": [
            ("create_binding",
             "Bind an actor to a role or scope for a given period"),
            ("end_binding",
             "End an active binding when an actor leaves a role"),
            ("query_bindings",
             "List active bindings filtered by actor or role"),
            ("resolve_bindings_for",
             "Find all roles and scopes bound to a given actor"),
        ],
    },
}

COMMAND_SIGNAL_OVERRIDES: dict[str, list[str]] = {
    "pk-release-audit": [
        "release audit", "pre-release", "pre release", "release validation",
        "release checklist", "before release", "patch release",
        "commit push release", "publish release", "make the release",
    ],
    "pk-release": [
        "prepare release", "bump version", "update changelog",
        "stamp provenance", "release notes",
    ],
    "pk-publish": [
        "publish release", "github release", "push tag", "release assets",
    ],
    "pk-doctor": [
        "doctor", "health check", "warnings", "resolve warning",
        "processkit health",
    ],
    "pk-wrapup": [
        "wrap up", "handover", "session handover", "end of session",
    ],
    "pk-route": [
        "route task", "which model", "model routing", "task router",
    ],
}

COMMAND_SIGNAL_THRESHOLD = 0.62
LOW_CONFIDENCE_FALLBACK_THRESHOLD = 0.5

# ---------------------------------------------------------------------------
# v1 → v2 successor table (BACK-20260509_1318-WarmOak / GH #21)
# ---------------------------------------------------------------------------
# Down-weight router candidates that point at v1 skills when a v2 successor
# skill exists for the same primitive kind. The penalty is configurable
# per-project via mcp/user_config.json (see _v1_penalty()); default 0.3.
#
# Keys are skill names whose SKILL.md frontmatter declares
# `apiVersion: processkit.projectious.work/v1`. Values are the v2 successor
# skill(s) the agent should consider instead.
V1_SUCCESSOR_SKILLS: dict[str, list[str]] = {
    "process-management": ["scope-management", "gate-management"],
    "state-machine-management": ["scope-management", "gate-management"],
}

# Keys are DOMAIN_GROUPS group names whose tools target v1 primitive kinds
# (e.g. the `actor` group surfaces create_actor for the v1 Actor entity).
# Successors point at v2 skills.
V1_SUCCESSOR_GROUPS: dict[str, list[str]] = {
    "actor": ["team-manager"],
}

DEFAULT_V1_PENALTY = 0.3

# ---------------------------------------------------------------------------
# Sub-agent dispatch hints (BACK-20260509_1317-WildPanda P2;
#                           BACK-20260510_0344-MerryFox P2 follow-up)
# ---------------------------------------------------------------------------
# Maps domain group → preferred ROLE-<slug> for sub-agent identity.
# Used by `_recommend_team_member()` to surface a TeamMember whose
# `default_role` matches when no skill-level binding exists.
#
# PM groups (product-manager): workitem, decision, discussion, scope, retro
# Engineering groups (software-engineer): all remaining 9 groups — these
# are infrastructure/tooling primitives owned by the engineering function:
#   event   — audit log (dev infra)
#   actor   — identity/profile plumbing
#   index   — full-text search & entity catalog
#   skill   — skill catalog & routing tooling
#   gate    — quality gates / TDD criteria
#   model   — model registry & routing infrastructure
#   id      — ID generation & validation
#   role    — role catalog administration
#   binding — actor-role-scope assignment management
GROUP_PREFERRED_ROLE: dict[str, str] = {
    # PM-coordination groups (unchanged from WildPanda P1)
    "workitem": "ROLE-product-manager",
    "decision": "ROLE-product-manager",
    "discussion": "ROLE-product-manager",
    "scope": "ROLE-product-manager",
    "retro": "ROLE-product-manager",
    # Engineering-owned groups (WildPanda P2 — MerryFox)
    "event": "ROLE-software-engineer",
    "actor": "ROLE-software-engineer",
    "index": "ROLE-software-engineer",
    "skill": "ROLE-software-engineer",
    "gate": "ROLE-software-engineer",
    "model": "ROLE-software-engineer",
    "id": "ROLE-software-engineer",
    "role": "ROLE-software-engineer",
    "binding": "ROLE-software-engineer",
}

# Group → recommended model class. Routing surfaces this so callers
# dispatching sub-agents can pick the cheapest model in the class
# (Haiku < Sonnet < Opus). Defaults to "fast" for read-only / classifier
# work; "deep" is reserved for cross-cutting reasoning. The class names
# match the keys under model-recommender's user_config.json.
GROUP_MODEL_CLASS: dict[str, str] = {
    "workitem": "fast",
    "decision": "deep",
    "discussion": "deep",
    "event": "fast",
    "actor": "fast",
    "scope": "fast",
    "index": "fast",
    "skill": "fast",
    "gate": "fast",
    "model": "fast",
    "id": "fast",
    "role": "fast",
    "retro": "deep",
    "binding": "fast",
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _project_root() -> Path:
    return paths.find_project_root()


def _v1_penalty() -> float:
    """Return the configured v1-entity score multiplier.

    Reads ``v1_entity_penalty`` from
    ``context/skills/processkit/task-router/mcp/user_config.json``.
    Default ``DEFAULT_V1_PENALTY`` (0.3). Set to 1.0 to disable
    (projects intentionally on v1 should disable).
    """
    cfg_path = (
        _project_root()
        / "context" / "skills" / "processkit"
        / "task-router" / "mcp" / "user_config.json"
    )
    try:
        cfg = json.loads(cfg_path.read_text(encoding="utf-8"))
    except Exception:
        return DEFAULT_V1_PENALTY
    val = cfg.get("v1_entity_penalty")
    try:
        penalty = float(val)
    except (TypeError, ValueError):
        return DEFAULT_V1_PENALTY
    if penalty < 0.0 or penalty > 1.0:
        return DEFAULT_V1_PENALTY
    return penalty


def _skill_is_v1(skill_name: str) -> bool:
    """True iff the skill's SKILL.md declares apiVersion ...v1."""
    if skill_name in V1_SUCCESSOR_SKILLS:
        return True
    skills_root = _project_root() / "context" / "skills"
    for cat_dir in skills_root.iterdir():
        if not cat_dir.is_dir() or cat_dir.name.startswith("_"):
            continue
        skill_md = cat_dir / skill_name / "SKILL.md"
        if not skill_md.exists():
            continue
        try:
            text = skill_md.read_text(encoding="utf-8")
        except Exception:
            return False
        # Cheap top-level apiVersion: line check (avoid yaml import here)
        for line in text.splitlines():
            s = line.strip()
            if s.startswith("apiVersion:"):
                return s.endswith("/v1")
            if s == "---" and "apiVersion:" not in text[:512]:
                return False
        return False
    return False


def _v1_successors(*, skill: str | None = None, group: str | None = None) -> list[str]:
    """Return v2 successor skill names for a v1 skill or v1 domain group."""
    if skill and skill in V1_SUCCESSOR_SKILLS:
        return list(V1_SUCCESSOR_SKILLS[skill])
    if group and group in V1_SUCCESSOR_GROUPS:
        return list(V1_SUCCESSOR_GROUPS[group])
    if skill and _skill_is_v1(skill):
        # Skill marked v1 but not in successor table: no concrete successor,
        # still penalise so v2 candidates win when present.
        return []
    return []


def _score_tokens(task: str, phrases: list[str]) -> float:
    """Token-overlap score. Returns 1.0 on exact substring match."""
    task_lower = task.lower()
    task_tokens = set(re.split(r"\W+", task_lower)) - {""}
    best = 0.0
    for phrase in phrases:
        phrase_lower = phrase.lower()
        if phrase_lower in task_lower:
            return 1.0
        phrase_tokens = set(re.split(r"\W+", phrase_lower)) - {""}
        if not phrase_tokens:
            continue
        overlap = len(task_tokens & phrase_tokens) / len(phrase_tokens)
        if overlap > best:
            best = overlap
    return best


def _char_ngrams(text: str, n: int = 3) -> Counter[str]:
    normalized = re.sub(r"\s+", " ", text.lower()).strip()
    if len(normalized) <= n:
        return Counter([normalized]) if normalized else Counter()
    return Counter(normalized[i:i + n] for i in range(len(normalized) - n + 1))


def _cosine(a: Counter[str], b: Counter[str]) -> float:
    if not a or not b:
        return 0.0
    dot = sum(v * b.get(k, 0) for k, v in a.items())
    norm_a = math.sqrt(sum(v * v for v in a.values()))
    norm_b = math.sqrt(sum(v * v for v in b.values()))
    if not norm_a or not norm_b:
        return 0.0
    return dot / (norm_a * norm_b)


def _score_embedding_style(task: str, phrases: list[str]) -> float:
    """Local semantic-ish score using character n-gram cosine similarity."""
    task_vec = _char_ngrams(task)
    best = 0.0
    for phrase in phrases:
        best = max(best, _cosine(task_vec, _char_ngrams(phrase)))
    return best


def _score_phrases(task: str, phrases: list[str], mode: str) -> tuple[float, str]:
    token_score = _score_tokens(task, phrases)
    if mode == "token":
        return token_score, "token"
    embedding_score = _score_embedding_style(task, phrases)
    if mode == "embedding":
        return embedding_score, "embedding"
    if embedding_score > token_score:
        return embedding_score, "embedding"
    return token_score, "token"


def _frontmatter_and_body(path: Path) -> tuple[dict, str]:
    try:
        text = path.read_text(encoding="utf-8")
    except Exception:
        return {}, ""
    if not text.startswith("---"):
        return {}, text
    try:
        end = text.index("---", 3)
        import yaml
        fm = yaml.safe_load(text[3:end])
        body = text[end + 3:].strip()
        return (fm if isinstance(fm, dict) else {}), body
    except Exception:
        return {}, text


def _agent_command_skill(command: str, description: str) -> str:
    match = re.search(r"Use the ([a-z0-9_-]+) skill", description)
    if match:
        return match.group(1)
    return command


def _load_command_specs() -> list[dict]:
    root = _project_root()
    specs: dict[str, dict] = {}
    commands_root = root / "context" / "skills" / "processkit"
    for path in sorted(commands_root.glob("*/commands/pk-*.md")):
        command = path.stem
        fm, body = _frontmatter_and_body(path)
        skill = path.parents[1].name
        specs[command] = {
            "command": command,
            "skill": skill,
            "description": " ".join(body.split()),
            "argument_hint": fm.get("argument-hint") or "",
            "source": str(path.relative_to(root)),
        }

    agent_root = root / ".agents" / "skills"
    for path in sorted(agent_root.glob("pk-*/SKILL.md")):
        command = path.parent.name
        if command in specs:
            continue
        fm, body = _frontmatter_and_body(path)
        description = " ".join(
            str(fm.get("description") or body or "").split()
        )
        specs[command] = {
            "command": command,
            "skill": _agent_command_skill(command, description),
            "description": description,
            "argument_hint": fm.get("argument-hint") or "",
            "source": str(path.relative_to(root)),
        }
    return list(specs.values())


def _command_phrases(spec: dict) -> list[str]:
    command = spec["command"]
    words = command.removeprefix("pk-").replace("-", " ")
    phrases = [
        command,
        "/" + command,
        words,
        spec.get("description") or "",
        spec.get("argument_hint") or "",
    ]
    phrases.extend(COMMAND_SIGNAL_OVERRIDES.get(command, []))
    return [p for p in phrases if p]


def _command_scores(
    task: str,
    scoring_mode: str,
) -> list[tuple[float, dict, str, list[str]]]:
    scored: list[tuple[float, dict, str, list[str]]] = []
    for spec in _load_command_specs():
        phrases = _command_phrases(spec)
        score, basis = _score_phrases(task, phrases, scoring_mode)
        if score <= 0:
            continue
        matched = [
            phrase for phrase in phrases
            if phrase.lower() in task.lower()
        ][:5]
        scored.append((score, spec, basis, matched))
    scored.sort(key=lambda item: -item[0])
    return scored


def _command_route_result(
    score: float,
    spec: dict,
    basis: str,
    matched: list[str],
    *,
    candidates: list[tuple[float, dict, str, list[str]]] | None = None,
) -> dict:
    skill_name = spec["skill"]
    result = {
        "route_type": "command",
        "command": spec["command"],
        "skill": skill_name,
        "skill_description_excerpt": _read_skill_description(skill_name),
        "server": "processkit-skill-finder",
        "tool": "find_skill",
        "tool_qualified": "processkit-skill-finder__find_skill",
        "domain_group": "command",
        "confidence": round(score, 2),
        "routing_basis": "command_signal",
        "scoring_basis": {"command": basis},
        "matched_signals": matched,
        "source": spec.get("source"),
        "recommended_team_member_slug": _recommend_team_member(
            _group_for_skill(skill_name)
        ),
        "recommended_model_class": _recommend_model_class(
            _group_for_skill(skill_name)
        ),
    }
    po = _find_process_override(skill_name)
    if po:
        result["process_override"] = po
        result["process_override_status"] = "legacy-v1"
    if candidates:
        result["candidate_routes"] = [
            {
                "route_type": "command",
                "command": cand["command"],
                "skill": cand["skill"],
                "confidence": round(sc, 2),
                "scoring_basis": cand_basis,
                "matched_signals": cand_matched,
            }
            for sc, cand, cand_basis, cand_matched in candidates[:5]
        ]
    return result


def _list_active_team_members() -> list[dict]:
    """Cheap roster read: parse team-member.md frontmatter under
    context/team-members/. Avoids importing the team-manager package
    (this server stays a leaf-skill stdio MCP). Returns dicts with
    slug, default_role, default_seniority, active.
    """
    tm_root = _project_root() / "context" / "team-members"
    if not tm_root.is_dir():
        return []
    out: list[dict] = []
    for child in sorted(tm_root.iterdir()):
        if not child.is_dir():
            continue
        p = child / "team-member.md"
        if not p.is_file():
            continue
        fm, _ = _frontmatter_and_body(p)
        if not isinstance(fm, dict):
            continue
        spec = fm.get("spec") if isinstance(fm.get("spec"), dict) else {}
        if not spec.get("active", True):
            continue
        out.append({
            "slug": spec.get("slug") or child.name,
            "default_role": spec.get("default_role"),
            "default_seniority": spec.get("default_seniority"),
            "type": spec.get("type"),
        })
    return out


# Pure-ordinal seniority ranking (DEC-20260422_0234-BraveFalcon).
_SENIORITY_RANK = {
    "principal": 5,
    "senior": 4,
    "expert": 3,
    "specialist": 2,
    "junior": 1,
}


def _recommend_team_member(domain_group: str | None) -> str | None:
    """Return the slug of the highest-priority active TeamMember whose
    `default_role` matches the group's preferred role, or None.

    Picks the highest seniority among matches; AI agents are preferred
    over humans (humans typically only dispatch as ad-hoc owners).
    """
    if not domain_group:
        return None
    role = GROUP_PREFERRED_ROLE.get(domain_group)
    if not role:
        return None
    candidates = [
        tm for tm in _list_active_team_members()
        if tm.get("default_role") == role
    ]
    if not candidates:
        return None
    candidates.sort(
        key=lambda tm: (
            tm.get("type") != "ai-agent",  # ai-agent first
            -_SENIORITY_RANK.get(tm.get("default_seniority") or "", 0),
        )
    )
    return candidates[0].get("slug")


def _recommend_model_class(domain_group: str | None) -> str | None:
    if not domain_group:
        return None
    return GROUP_MODEL_CLASS.get(domain_group)


def _group_for_skill(skill_name: str | None) -> str | None:
    """Reverse-lookup: skill name → first DOMAIN_GROUPS entry that owns
    the skill. Used by the command-route path so commands surface the
    same TeamMember/model-class hints as their underlying domain group.
    """
    if not skill_name:
        return None
    for name, g in DOMAIN_GROUPS.items():
        if g.get("skill") == skill_name:
            return name
    return None


def _cheap_model_hint() -> dict:
    cfg_path = (
        _project_root()
        / "context"
        / "skills"
        / "processkit"
        / "model-recommender"
        / "mcp"
        / "user_config.json"
    )
    try:
        cfg = json.loads(cfg_path.read_text(encoding="utf-8"))
    except Exception:
        cfg = {}
    configured = (cfg.get("model_classes") or {}).get("fast")
    return {
        "configured": bool(configured),
        "model_class": "fast",
        "model": configured,
    }


def _phase1_groups(
    task: str,
    scoring_mode: str = "auto",
    trace: list[str] | None = None,
) -> list[tuple[float, str, str]]:
    """Score all domain groups. Returns [(score, group_name, basis)].

    Applies the v1-entity penalty (BACK-20260509_1318-WarmOak): groups
    whose tools target v1 primitive kinds with v2 successors get their
    score multiplied by ``_v1_penalty()``. When ``trace`` is supplied,
    appends a one-line note for each penalised group.
    """
    penalty = _v1_penalty()
    scored = []
    for name, g in DOMAIN_GROUPS.items():
        score, basis = _score_phrases(task, g["keywords"], scoring_mode)
        if score <= 0:
            continue
        if name in V1_SUCCESSOR_GROUPS and penalty < 1.0:
            successors = V1_SUCCESSOR_GROUPS[name]
            adjusted = score * penalty
            if trace is not None:
                trace.append(
                    f"matched group '{name}'; v1-entity penalty {penalty} "
                    f"applied; consider v2 successors: {', '.join(successors)}"
                )
            score = adjusted
        scored.append((score, name, basis))
    scored.sort(key=lambda x: -x[0])
    return scored


def _phase2_tools(
    task: str, group_name: str, scoring_mode: str = "auto"
) -> list[tuple[float, str, str, str]]:
    """Score tools within a group."""
    group = DOMAIN_GROUPS[group_name]
    scored = []
    for tool_name, tool_desc in group["tools"]:
        phrases = [tool_name.replace("_", " "), tool_desc]
        s, basis = _score_phrases(task, phrases, scoring_mode)
        if s > 0:
            name_score = _score_tokens(task, [tool_name.replace("_", " ")])
            rationale = (
                f"task vocabulary matches tool name '{tool_name}'"
                if name_score >= 0.5
                else f"task matches tool description via {basis} scoring"
            )
            scored.append((s, tool_name, rationale, basis))
    scored.sort(key=lambda x: -x[0])
    return scored


def _read_skill_description(skill_name: str) -> str:
    """Return first 150 chars of a skill's description from SKILL.md."""
    skills_root = _project_root() / "context" / "skills"
    for cat_dir in skills_root.iterdir():
        if not cat_dir.is_dir() or cat_dir.name.startswith("_"):
            continue
        skill_md = cat_dir / skill_name / "SKILL.md"
        if skill_md.exists():
            try:
                import yaml
                text = skill_md.read_text(encoding="utf-8")
                if text.startswith("---"):
                    end = text.index("---", 3)
                    fm = yaml.safe_load(text[3:end])
                    if isinstance(fm, dict):
                        desc = (fm.get("description") or "").strip()
                        # Flatten block scalars and truncate
                        desc = " ".join(desc.split())
                        return desc[:150]
            except Exception:
                pass
    return ""


def _find_process_override(skill_name: str) -> str | None:
    """Return legacy v1 context/processes/<name>.md override if present.

    v2 processkit releases do not ship first-class Process entities or
    src/context/processes/. Existing projects may still carry live
    context/processes/*.md files as migration-source compatibility data.
    """
    processes_dir = _project_root() / "context" / "processes"
    if not processes_dir.exists():
        return None
    # Exact match first
    candidate = processes_dir / f"{skill_name}.md"
    if candidate.exists():
        return str(candidate.relative_to(_project_root()))
    # Prefix match: "release-semver" → "release.md"
    base = skill_name.split("-")[0]
    for p in sorted(processes_dir.glob(f"{base}*.md")):
        if p.name != "INDEX.md":
            return str(p.relative_to(_project_root()))
    return None


def _skill_finder_fallback(
    task: str, trace: list[str] | None = None
) -> dict | None:
    """
    Fallback: parse skill-finder trigger table when group score < 0.3.
    Returns a partial result dict or None.

    Applies the v1-entity penalty (BACK-20260509_1318-WarmOak) to skills
    whose SKILL.md declares apiVersion ...v1 with a known v2 successor.
    """
    finder_md = (
        _project_root()
        / "context" / "skills" / "processkit" / "skill-finder" / "SKILL.md"
    )
    if not finder_md.exists():
        return None
    try:
        text = finder_md.read_text(encoding="utf-8")
    except Exception:
        return None

    rows: list[tuple[list[str], str]] = []
    in_table = False
    for line in text.splitlines():
        s = line.strip()
        if "If the user says" in s and "Skill to load" in s:
            in_table = True
            continue
        if not in_table:
            continue
        if s.startswith("|---") or s == "":
            continue
        if not s.startswith("|"):
            in_table = False
            continue
        parts = [p.strip() for p in s.strip("|").split("|")]
        if len(parts) < 2:
            continue
        skill = parts[1].strip("`").strip()
        if not skill or skill == "Skill to load":
            continue
        phrases = [
            p.strip().strip('"')
            for p in parts[0].split('", "')
            if p.strip().strip('"')
        ] or [parts[0].strip('"')]
        rows.append((phrases, skill))

    penalty = _v1_penalty()
    scored: list[tuple[float, str]] = []
    for phrases, sk in rows:
        sc = _score_tokens(task, phrases)
        if sc <= 0:
            continue
        if penalty < 1.0 and (sk in V1_SUCCESSOR_SKILLS or _skill_is_v1(sk)):
            successors = _v1_successors(skill=sk)
            adjusted = sc * penalty
            if trace is not None:
                hint = (
                    f"; consider v2 successors: {', '.join(successors)}"
                    if successors else ""
                )
                trace.append(
                    f"matched skill '{sk}'; v1-entity penalty {penalty} "
                    f"applied{hint}"
                )
            sc = adjusted
        scored.append((sc, sk))
    if not scored:
        return None
    scored.sort(key=lambda x: -x[0])
    best_score, best_skill = scored[0]
    return {
        "skill": best_skill,
        "skill_description_excerpt": _read_skill_description(best_skill),
        "confidence": round(best_score, 2),
        "routing_basis": "skill_finder_trigger_table",
    }


# ---------------------------------------------------------------------------
# Tool
# ---------------------------------------------------------------------------


@server.tool(
    annotations=ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    )
)
def route_task(
    task_description: str,
    allow_llm_escalation: bool = False,
    scoring_mode: str = "auto",
    intent_signals: list[str] | None = None,
) -> dict:
    """Route a task to the matching processkit skill, process override,
    and MCP tool — in a single call, without an LLM.

    Prerequisite: call this tool at the start of any processkit domain
    task — before calling create_*, transition_*, link_*, or record_*
    tools — to confirm the right skill and any project-specific process
    override for this derived project.

    Two-phase heuristic routing:
      Phase 1 — keyword match to a domain group (eliminates ~90 %
                 of candidates; no LLM needed).
      Phase 2 — token-overlap or local embedding-style scoring within the
                 matched group's tools.
      Fallback — skill-finder trigger-phrase table for cross-domain
                 tasks not covered by any domain group.

    When ``confidence < 0.5`` (routing_basis == "needs_llm_confirm"),
    surface ``candidate_tools`` to the user or an LLM for confirmation.
    If ``allow_llm_escalation`` is true, the response includes the
    configured fast-model class hint, but this server does not make
    provider network calls itself. The router never blocks — it always
    returns its best guess. 1% rule: if there is a 1% chance a processkit
    skill covers this task, call route_task before acting.

    Parameters
    ----------
    task_description :
        What the agent or user wants to do, in natural language.
        More specific phrasing → better match quality.

    Returns
    -------
    On match:
      skill                    — processkit skill name to load
      skill_description_excerpt — first 150 chars of skill description
      process_override         — legacy v1 path to a project-specific
                                 process file (only present when one exists)
      process_override_status  — present with process_override; currently
                                 "legacy-v1"
      server                   — MCP server to connect to
      tool                     — recommended tool name
      tool_qualified           — "{server}__{tool}" collision-safe form
      domain_group             — routing group (workitem, decision, …)
      confidence               — 0.0–1.0 combined routing confidence
      routing_basis            — keyword_match | skill_finder_trigger_table
                                 | needs_llm_confirm
      candidate_tools[]        — top-3 scored tools with rationales
      recommended_team_member_slug — slug of the highest-priority active
                                 TeamMember whose default_role matches
                                 the routed group's preferred role, or
                                 None when no binding resolves. Use this
                                 as the sub-agent's identity at dispatch
                                 (per the compliance contract sub-agent-
                                 dispatch clause).
      recommended_model_class  — "fast" | "deep" | None. Hint for picking
                                 the cheapest concrete model in the class
                                 (Haiku < Sonnet < Opus) when dispatching
                                 a sub-agent. Currently a static per-
                                 group mapping; future revisions may
                                 derive this from skill metadata.

    On no match:
      error, hint
    """
    task = task_description.strip()
    if not task:
        return {"error": "task_description must not be empty"}
    if scoring_mode not in ("auto", "token", "embedding"):
        return {"error": "scoring_mode must be auto, token, or embedding"}
    signals = [str(s).strip() for s in (intent_signals or []) if str(s).strip()]
    scored_task = " ".join([task, *signals])
    trace: list[str] = []

    command_candidates = _command_scores(scored_task, scoring_mode)
    if (
        command_candidates
        and command_candidates[0][0] >= COMMAND_SIGNAL_THRESHOLD
        and command_candidates[0][3]
    ):
        score, spec, basis, matched = command_candidates[0]
        result = _command_route_result(
            score,
            spec,
            basis,
            matched,
            candidates=command_candidates,
        )
        if signals:
            result["intent_signals"] = signals
        try:
            _write_route_task_marker(paths.find_project_root())
        except Exception:
            pass
        return result

    # ── Phase 1: domain group scoring ─────────────────────────────────────
    group_scores = _phase1_groups(
        scored_task, scoring_mode=scoring_mode, trace=trace
    )

    if not group_scores or group_scores[0][0] < 0.3:
        # Fallback: skill-finder trigger table
        fb = _skill_finder_fallback(scored_task, trace=trace)
        if fb:
            po = _find_process_override(fb["skill"])
            result = dict(fb)
            if po:
                result["process_override"] = po
                result["process_override_status"] = "legacy-v1"
            result["hint"] = (
                "No domain group matched; routed via skill-finder trigger "
                "table. Load the skill SKILL.md before proceeding."
            )
            inferred_group = _group_for_skill(fb.get("skill"))
            result["recommended_team_member_slug"] = (
                _recommend_team_member(inferred_group)
            )
            result["recommended_model_class"] = (
                _recommend_model_class(inferred_group)
            )
            if trace:
                result["trace"] = list(trace)
            return result
        return {
            "error": "no matching skill or tool found",
            "hint": (
                "Rephrase in natural language, e.g. 'create a work item "
                "for the login bug' or 'record a decision about the DB'."
            ),
        }

    best_group_score, best_group, group_basis = group_scores[0]
    group = DOMAIN_GROUPS[best_group]
    server_name: str = group["server"]
    skill_name: str = group["skill"]

    # ── Phase 2: tool scoring within group ────────────────────────────────
    tool_scores = _phase2_tools(scored_task, best_group, scoring_mode=scoring_mode)

    if tool_scores:
        best_tool_score, best_tool, best_rationale, tool_basis = tool_scores[0]
    else:
        best_tool, _ = group["tools"][0]
        best_tool_score = best_group_score * 0.5
        best_rationale = f"defaulted to first tool in '{best_group}' group"
        tool_basis = group_basis

    # Confidence = geometric mean of group and tool scores
    confidence = round(math.sqrt(best_group_score * best_tool_score), 2)
    routing_basis = (
        "keyword_match" if confidence >= 0.5 else "needs_llm_confirm"
    )

    fallback = None
    if confidence < LOW_CONFIDENCE_FALLBACK_THRESHOLD:
        fallback = _skill_finder_fallback(scored_task, trace=trace)
        if fallback and fallback.get("confidence", 0) >= confidence:
            po = _find_process_override(fallback["skill"])
            result = dict(fallback)
            if po:
                result["process_override"] = po
                result["process_override_status"] = "legacy-v1"
            result["also_consider_route"] = {
                "route_type": "mcp_tool",
                "skill": skill_name,
                "server": server_name,
                "tool": best_tool,
                "tool_qualified": f"{server_name}__{best_tool}",
                "domain_group": best_group,
                "confidence": confidence,
            }
            result["recommended_team_member_slug"] = (
                _recommend_team_member(best_group)
            )
            result["recommended_model_class"] = (
                _recommend_model_class(best_group)
            )
            if command_candidates:
                result["candidate_routes"] = [
                    {
                        "route_type": "command",
                        "command": cand["command"],
                        "skill": cand["skill"],
                        "confidence": round(sc, 2),
                        "scoring_basis": cand_basis,
                        "matched_signals": cand_matched,
                    }
                    for sc, cand, cand_basis, cand_matched
                    in command_candidates[:5]
                ]
            if signals:
                result["intent_signals"] = signals
            if allow_llm_escalation and result.get("confidence", 0) < 0.5:
                hint = _cheap_model_hint()
                result["llm_escalation"] = {
                    "status": (
                        "configured_but_not_invoked"
                        if hint["configured"] else "not_configured"
                    ),
                    "reason": (
                        "task-router is a read-only stdio MCP server and "
                        "does not make provider network calls; caller should "
                        "confirm among candidate routes or invoke the "
                        "configured fast model."
                    ),
                    **hint,
                }
            if trace:
                result["trace"] = list(trace)
            return result

    # ── Skill description excerpt ──────────────────────────────────────────
    skill_excerpt = _read_skill_description(skill_name)

    # ── Process override ───────────────────────────────────────────────────
    process_override = _find_process_override(skill_name)

    # ── Candidate tools (top-3 for transparency + fallback) ───────────────
    candidates: list[dict] = []
    seen: set[str] = set()
    for score, tool_name, rationale, basis in tool_scores[:3]:
        candidates.append({
            "tool_qualified": f"{server_name}__{tool_name}",
            "score": round(score, 2),
            "rationale": rationale,
            "scoring_basis": basis,
        })
        seen.add(tool_name)

    # Pad to 3 with unscored tools from the group
    if len(candidates) < 3:
        for tool_name, _ in group["tools"]:
            if tool_name in seen:
                continue
            candidates.append({
                "tool_qualified": f"{server_name}__{tool_name}",
                "score": 0.0,
                "rationale": "in domain group; not vocabulary-matched",
            })
            if len(candidates) >= 3:
                break

    result: dict = {
        "skill": skill_name,
        "skill_description_excerpt": skill_excerpt,
        "server": server_name,
        "tool": best_tool,
        "tool_qualified": f"{server_name}__{best_tool}",
        "domain_group": best_group,
        "confidence": confidence,
        "routing_basis": routing_basis,
        "scoring_basis": {"group": group_basis, "tool": tool_basis},
        "candidate_tools": candidates,
        "recommended_team_member_slug": _recommend_team_member(best_group),
        "recommended_model_class": _recommend_model_class(best_group),
    }
    if signals:
        result["intent_signals"] = signals
    if fallback:
        result["also_consider_skill"] = {
            "skill": fallback["skill"],
            "confidence": fallback.get("confidence", 0),
            "routing_basis": fallback.get("routing_basis"),
        }
    if command_candidates:
        result["candidate_routes"] = [
            {
                "route_type": "command",
                "command": cand["command"],
                "skill": cand["skill"],
                "confidence": round(sc, 2),
                "scoring_basis": cand_basis,
                "matched_signals": cand_matched,
            }
            for sc, cand, cand_basis, cand_matched in command_candidates[:5]
        ]

    if process_override:
        result["process_override"] = process_override
        result["process_override_status"] = "legacy-v1"

    # Surface close runner-up group when scores are within 20 %
    if len(group_scores) > 1 and group_scores[1][0] >= best_group_score * 0.8:
        result["also_consider_group"] = group_scores[1][1]

    if allow_llm_escalation and routing_basis == "needs_llm_confirm":
        hint = _cheap_model_hint()
        result["llm_escalation"] = {
            "status": (
                "configured_but_not_invoked"
                if hint["configured"] else "not_configured"
            ),
            "reason": (
                "task-router is a read-only stdio MCP server and does not "
                "make provider network calls; caller should confirm among "
                "candidate_tools or invoke the configured fast model."
            ),
            **hint,
        }

    if trace:
        result["trace"] = list(trace)

    # Write per-turn marker so check_route_task_before_agent.py hook passes.
    try:
        _write_route_task_marker(paths.find_project_root())
    except Exception:
        pass  # marker write failure must not break routing

    return result


if __name__ == "__main__":
    server.run(transport="stdio")
