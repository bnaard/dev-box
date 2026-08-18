"""team-creator v2 helper library.

Catalog-driven archetype resolution for pk-team-create, pk-team-rebalance,
and pk-team-review. The skill itself is documentation-driven (commands/*.md
narrate the workflow), but the mapping loader is shared by the migration
apply script and the test suite.

Public surface:
  - load_archetype_catalog_mapping(project_root)
  - resolve_archetype(name, mapping)            -> {role, seniority, ...}
  - mapping_source(...)                         -> "kit-default" | "project" | "cli"
  - compute_slot_projection(slot, unit_cost_usd, scope_window,
                            invocations_per_week, avg_tokens)
                                                -> slot_projection dict
  - build_budget_projection(slot_projections, scope_window,
                            drift_threshold_pct, projection_method, notes)
                                                -> budget_projection dict
  - compute_budget_drift(budget_projection, actual_slot_costs)
                                                -> drift dict

The loader implements the layered precedence promised by SKILL.md:

    cli (--archetype-catalog-mapping <path>) > project > kit-default

Project override: context/team/archetype-catalog-mapping.yaml (delta semantics
by default; replace semantics only when the override file declares
``override_semantics: replace`` at the top level).

The kit default ships at
``context/skills/processkit/team-creator/assets/archetype-catalog-mapping.yaml``
and contains the 8 archetypes listed in the SmartPanda design artifact §"Gap 1".

Budget projection (Gap 5 — SUB-4 / BACK-20260509_1837-SwiftReef):
  The ``compute_slot_projection`` and ``build_budget_projection`` helpers
  implement the charter-time heuristic cost model described in the
  SmartPanda design §"Gap 5 — Budget projection".  They are pure functions
  so they can be tested without MCP server state.

  Heuristic formula (per slot):
      weeks = ceil(effective_window_days / 7)
      projected_total_usd = (
          unit_cost_usd                   # USD per token
          × (avg_input + avg_output)      # avg tokens per invocation
          × expected_invocations_per_week
          × weeks
      )

  For consultant slots the effective window is the intersection of the
  consultant's ``engagement_window`` and the chartering Scope window.
  Pass the intersected window via ``scope_window`` — the caller is
  responsible for the intersection (pk-team-create has the full scope +
  consultant data at charter time).
"""
from __future__ import annotations

import datetime as _dt
import math
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml


# The 8 processkit archetypes (kept as a stable tuple for validation).
ARCHETYPES: tuple[str, ...] = (
    "project-manager",
    "senior-architect",
    "senior-researcher",
    "junior-architect",
    "developer",
    "junior-researcher",
    "junior-developer",
    "assistant",
)


_VALID_OVERRIDE_SEMANTICS = {"delta", "replace"}


# ---------------------------------------------------------------------------
# Kit-default discovery
# ---------------------------------------------------------------------------

def kit_default_mapping_path() -> Path:
    """Return the absolute path to the kit-default mapping shipped here."""
    return Path(__file__).resolve().parent.parent / "assets" / "archetype-catalog-mapping.yaml"


def project_override_mapping_path(project_root: Path) -> Path:
    """Return the project override path.

    Always returned regardless of existence — the caller decides whether
    to apply the override based on ``Path.is_file()``.
    """
    return project_root / "context" / "team" / "archetype-catalog-mapping.yaml"


# ---------------------------------------------------------------------------
# Result type
# ---------------------------------------------------------------------------

@dataclass
class ArchetypeMapping:
    """Resolved archetype-to-catalog mapping.

    Attributes
    ----------
    archetypes: dict mapping archetype-name -> {"role": ROLE-id,
        "seniority": <enum>, optional "primary_contact": bool, ...}
    source: "kit-default" | "project" | "cli"
    overrides: list of per-archetype delta entries when source == "project"
        and the project file is in delta mode. Each entry:
        {"archetype": <name>, "field": <name>, "kit_default": ...,
         "project_value": ...}
    semantics: "delta" | "replace" — only meaningful when source == "project"
    """

    archetypes: dict[str, dict[str, Any]]
    source: str = "kit-default"
    overrides: list[dict[str, Any]] = field(default_factory=list)
    semantics: str | None = None
    files: list[str] = field(default_factory=list)


# ---------------------------------------------------------------------------
# Loaders
# ---------------------------------------------------------------------------

def _load_yaml(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8")
    data = yaml.safe_load(text) or {}
    if not isinstance(data, dict):
        raise ValueError(f"{path}: top-level YAML must be a mapping")
    return data


def _extract_archetypes(doc: dict[str, Any], path: Path) -> dict[str, dict[str, Any]]:
    """Pull the ``spec.archetypes`` block (or top-level ``archetypes``)."""
    spec = doc.get("spec") or doc
    archetypes = spec.get("archetypes")
    if not isinstance(archetypes, dict):
        raise ValueError(f"{path}: missing or invalid 'archetypes' map")
    out: dict[str, dict[str, Any]] = {}
    for name, body in archetypes.items():
        if not isinstance(body, dict):
            raise ValueError(f"{path}: archetype {name!r} must be a mapping")
        role = body.get("role")
        seniority = body.get("seniority")
        if not role or not str(role).startswith("ROLE-"):
            raise ValueError(
                f"{path}: archetype {name!r} requires a 'role' starting with 'ROLE-'"
            )
        if not seniority:
            raise ValueError(f"{path}: archetype {name!r} requires a 'seniority'")
        entry: dict[str, Any] = {"role": role, "seniority": seniority}
        for opt in ("primary_contact", "default_model_profile",
                    "effort_floor", "effort_ceiling", "rationale"):
            if opt in body:
                entry[opt] = body[opt]
        out[name] = entry
    return out


def _load_kit_default() -> dict[str, dict[str, Any]]:
    path = kit_default_mapping_path()
    if not path.is_file():
        raise FileNotFoundError(
            f"kit-default mapping missing at {path} — team-creator install corrupted?"
        )
    doc = _load_yaml(path)
    return _extract_archetypes(doc, path)


def _validate_replace_completeness(archetypes: dict[str, dict[str, Any]],
                                    path: Path) -> None:
    missing = [a for a in ARCHETYPES if a not in archetypes]
    if missing:
        raise ValueError(
            f"{path}: replace-mode override missing archetypes: {missing!r}"
        )


def load_archetype_catalog_mapping(
    project_root: Path,
    cli_path: Path | None = None,
) -> ArchetypeMapping:
    """Load the archetype→catalog mapping with three-level precedence.

    Order:
      1. ``cli_path`` (if supplied; mode determined by file's
         ``override_semantics``)
      2. ``<project_root>/context/team/archetype-catalog-mapping.yaml``
      3. kit default shipped under team-creator/assets/

    Returns an :class:`ArchetypeMapping` describing the resolved set,
    its source, and any per-archetype delta entries.
    """
    project_root = Path(project_root)
    kit_archetypes = _load_kit_default()
    files: list[str] = [str(kit_default_mapping_path())]

    # --- Layer 1: CLI ------------------------------------------------------
    if cli_path is not None:
        cli_path = Path(cli_path)
        if not cli_path.is_file():
            raise FileNotFoundError(
                f"--archetype-catalog-mapping {cli_path} not found"
            )
        doc = _load_yaml(cli_path)
        semantics = doc.get("override_semantics", "replace")
        if semantics not in _VALID_OVERRIDE_SEMANTICS:
            raise ValueError(
                f"{cli_path}: override_semantics must be 'delta' or 'replace'; "
                f"got {semantics!r}"
            )
        cli_archetypes = _extract_archetypes(doc, cli_path)
        if semantics == "replace":
            _validate_replace_completeness(cli_archetypes, cli_path)
            merged = cli_archetypes
        else:
            merged = {**kit_archetypes, **cli_archetypes}
        overrides = _delta_overrides(kit_archetypes, cli_archetypes)
        files.append(str(cli_path))
        return ArchetypeMapping(
            archetypes=merged,
            source="cli",
            overrides=overrides,
            semantics=semantics,
            files=files,
        )

    # --- Layer 2: Project override ----------------------------------------
    proj_path = project_override_mapping_path(project_root)
    if proj_path.is_file():
        doc = _load_yaml(proj_path)
        semantics = doc.get("override_semantics", "delta")
        if semantics not in _VALID_OVERRIDE_SEMANTICS:
            raise ValueError(
                f"{proj_path}: override_semantics must be 'delta' or 'replace'; "
                f"got {semantics!r}"
            )
        proj_archetypes = _extract_archetypes(doc, proj_path)
        if semantics == "replace":
            _validate_replace_completeness(proj_archetypes, proj_path)
            merged = proj_archetypes
        else:
            merged = {**kit_archetypes, **proj_archetypes}
        overrides = _delta_overrides(kit_archetypes, proj_archetypes)
        files.append(str(proj_path))
        return ArchetypeMapping(
            archetypes=merged,
            source="project",
            overrides=overrides,
            semantics=semantics,
            files=files,
        )

    # --- Layer 3: kit default ---------------------------------------------
    return ArchetypeMapping(
        archetypes=kit_archetypes,
        source="kit-default",
        overrides=[],
        semantics=None,
        files=files,
    )


def _delta_overrides(
    kit: dict[str, dict[str, Any]],
    layer: dict[str, dict[str, Any]],
) -> list[dict[str, Any]]:
    """Return per-archetype-per-field delta entries.

    Used for the chartering DEC's ``inputs_snapshot.archetype_catalog_overrides``
    audit field.
    """
    out: list[dict[str, Any]] = []
    for name, entry in layer.items():
        kit_entry = kit.get(name) or {}
        for fld, val in entry.items():
            kit_val = kit_entry.get(fld)
            if kit_val != val:
                out.append({
                    "archetype": name,
                    "field": fld,
                    "kit_default": kit_val,
                    "project_value": val,
                })
        # New archetype not in kit: treat as a single "add" entry
        if name not in kit:
            out.append({
                "archetype": name,
                "field": "*",
                "kit_default": None,
                "project_value": "<new archetype>",
            })
    return out


def resolve_archetype(name: str, mapping: ArchetypeMapping) -> dict[str, Any]:
    """Look up an archetype in the resolved mapping. KeyError on miss."""
    if name not in mapping.archetypes:
        raise KeyError(
            f"archetype {name!r} not found in mapping (source={mapping.source}); "
            f"known archetypes: {sorted(mapping.archetypes)}"
        )
    return mapping.archetypes[name]


def archetype_for_role_slot(
    role: str, seniority: str, mapping: ArchetypeMapping,
) -> str | None:
    """Reverse lookup: given (role, seniority), return the archetype name.

    Used by pk-team-review for human-readable diff labels. Returns None
    when no archetype maps to the (role, seniority) pair.
    """
    for name, entry in mapping.archetypes.items():
        if entry["role"] == role and entry["seniority"] == seniority:
            return name
    return None


# ---------------------------------------------------------------------------
# Budget projection helpers (Gap 5 — SUB-4 / BACK-20260509_1837-SwiftReef)
# ---------------------------------------------------------------------------

_VALID_PROJECTION_METHODS = {"heuristic", "model-recommender-quote", "manual"}


def _parse_date(value: str) -> _dt.date:
    """Parse an ISO date string (YYYY-MM-DD) into a date object."""
    if isinstance(value, _dt.datetime):
        return value.date()
    if isinstance(value, _dt.date):
        return value
    return _dt.date.fromisoformat(str(value)[:10])


def _window_weeks(starts_at: str, ends_at: str) -> float:
    """Return the number of weeks (float) spanned by [starts_at, ends_at]."""
    start = _parse_date(starts_at)
    end = _parse_date(ends_at)
    days = max(0, (end - start).days)
    return math.ceil(days / 7) if days > 0 else 0.0


def intersect_windows(
    scope_starts: str,
    scope_ends: str,
    slot_starts: str | None,
    slot_ends: str | None,
) -> tuple[str, str] | None:
    """Intersect a Scope window with an optional slot-specific window.

    Returns (starts_at, ends_at) as ISO strings if the intersection is
    non-empty, otherwise ``None``.  When ``slot_starts``/``slot_ends``
    are both ``None`` the Scope window is returned unchanged.
    """
    scope_s = _parse_date(scope_starts)
    scope_e = _parse_date(scope_ends)
    if slot_starts is None and slot_ends is None:
        return str(scope_s), str(scope_e)
    slot_s = _parse_date(slot_starts) if slot_starts else scope_s
    slot_e = _parse_date(slot_ends) if slot_ends else scope_e
    eff_s = max(scope_s, slot_s)
    eff_e = min(scope_e, slot_e)
    if eff_e < eff_s:
        return None  # no overlap
    return str(eff_s), str(eff_e)


def compute_slot_projection(
    slot_id: str,
    role: str,
    seniority: str,
    model_profile: str | None,
    expected_invocations_per_week: int,
    avg_tokens: dict[str, int],
    unit_cost_usd: float,
    effective_window: tuple[str, str],
) -> dict[str, Any]:
    """Compute a per-slot budget projection row.

    Parameters
    ----------
    slot_id:                       SLOT-* id
    role:                          ROLE-* id
    seniority:                     enum string
    model_profile:                 ART-*-ModelProfile-* or None
    expected_invocations_per_week: int
    avg_tokens:                    {"input": <int>, "output": <int>}
    unit_cost_usd:                 price per token (USD) from get_pricing at charter time
    effective_window:              (starts_at, ends_at) ISO date strings

    Returns a slot_projections[] row as a plain dict.
    """
    avg_input = avg_tokens.get("input", 0)
    avg_output = avg_tokens.get("output", 0)
    weeks = _window_weeks(effective_window[0], effective_window[1])
    projected_total_usd = (
        unit_cost_usd
        * (avg_input + avg_output)
        * expected_invocations_per_week
        * weeks
    )
    return {
        "slot": slot_id,
        "role": role,
        "seniority": seniority,
        "model_profile": model_profile,
        "expected_invocations_per_week": expected_invocations_per_week,
        "avg_tokens_per_invocation": {"input": avg_input, "output": avg_output},
        "unit_cost_usd": unit_cost_usd,
        "projected_total_usd": round(projected_total_usd, 6),
        "effective_window": {
            "starts_at": effective_window[0],
            "ends_at": effective_window[1],
        },
    }


def build_budget_projection(
    slot_projections: list[dict[str, Any]],
    scope_window: dict[str, str],
    drift_threshold_pct: float = 20.0,
    projection_method: str = "heuristic",
    notes: str | None = None,
) -> dict[str, Any]:
    """Assemble the full budget_projection block for the chartering DecisionRecord.

    Parameters
    ----------
    slot_projections:    list of dicts produced by compute_slot_projection()
    scope_window:        {"starts_at": <ISO>, "ends_at": <ISO>}
    drift_threshold_pct: alert threshold, default 20.0
    projection_method:   "heuristic" | "model-recommender-quote" | "manual"
    notes:               optional free-text

    Returns the ``inputs_snapshot.budget_projection`` block.
    """
    if projection_method not in _VALID_PROJECTION_METHODS:
        raise ValueError(
            f"projection_method must be one of {sorted(_VALID_PROJECTION_METHODS)}; "
            f"got {projection_method!r}"
        )
    projected_total = round(
        sum(p.get("projected_total_usd", 0.0) for p in slot_projections), 6
    )
    block: dict[str, Any] = {
        "currency": "USD",
        "window": {
            "starts_at": scope_window["starts_at"],
            "ends_at": scope_window["ends_at"],
        },
        "projected_total": projected_total,
        "projection_method": projection_method,
        "slot_projections": slot_projections,
        "drift_threshold_pct": drift_threshold_pct,
    }
    if notes:
        block["notes"] = notes
    return block


def compute_budget_drift(
    budget_projection: dict[str, Any],
    actual_slot_costs: dict[str, float],
) -> dict[str, Any]:
    """Compute drift between projected and actual costs.

    Parameters
    ----------
    budget_projection:  the ``inputs_snapshot.budget_projection`` block
    actual_slot_costs:  mapping {slot_id -> actual_cost_usd}

    Returns a drift report dict:
    {
        "projected_total": <float>,
        "actual_total":    <float>,
        "drift_pct":       <float>,       # (actual - projected) / projected * 100
        "drift_direction": "over" | "under" | "on-track",
        "threshold_exceeded": <bool>,
        "threshold_pct":   <float>,
        "per_slot": [
            {slot, projected_total_usd, actual_cost_usd,
             slot_drift_pct, direction}, ...
        ],
        "finding_code":   "team.budget.drift" | None,
        "severity":       "warning" | "info" | None,
    }
    """
    threshold = float(budget_projection.get("drift_threshold_pct", 20.0))
    projected_total = float(budget_projection.get("projected_total", 0.0))
    actual_total = sum(actual_slot_costs.values())

    if projected_total <= 0:
        drift_pct = 0.0
    else:
        drift_pct = (actual_total - projected_total) / projected_total * 100.0

    threshold_exceeded = abs(drift_pct) > threshold

    if drift_pct > 0:
        drift_direction = "over"
    elif drift_pct < 0:
        drift_direction = "under"
    else:
        drift_direction = "on-track"

    per_slot: list[dict[str, Any]] = []
    for row in budget_projection.get("slot_projections", []):
        slot_id = row["slot"]
        proj = float(row.get("projected_total_usd", 0.0))
        actual = float(actual_slot_costs.get(slot_id, 0.0))
        if proj <= 0:
            slot_drift_pct = 0.0
        else:
            slot_drift_pct = (actual - proj) / proj * 100.0
        per_slot.append({
            "slot": slot_id,
            "projected_total_usd": proj,
            "actual_cost_usd": actual,
            "slot_drift_pct": round(slot_drift_pct, 2),
            "direction": "over" if slot_drift_pct > 0 else ("under" if slot_drift_pct < 0 else "on-track"),
        })

    # finding code + severity
    if threshold_exceeded:
        finding_code: str | None = "team.budget.drift"
        if drift_direction == "over":
            severity: str | None = "warning"   # over-spend = actionable
        else:
            severity = "info"                   # under-spend = informational
    else:
        finding_code = None
        severity = None

    return {
        "projected_total": projected_total,
        "actual_total": round(actual_total, 6),
        "drift_pct": round(drift_pct, 2),
        "drift_direction": drift_direction,
        "threshold_exceeded": threshold_exceeded,
        "threshold_pct": threshold,
        "per_slot": per_slot,
        "finding_code": finding_code,
        "severity": severity,
    }
