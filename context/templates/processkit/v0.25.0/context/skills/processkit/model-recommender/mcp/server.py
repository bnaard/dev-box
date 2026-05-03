#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "httpx>=0.27",
#   "pyyaml>=6.0",
# ]
# ///
"""
model-recommender MCP server

Provides structured queries over the model roster and user config.
Data lives in model_scores.json and user_config.json alongside this file.
"""

import json
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Optional

from mcp.server.fastmcp import FastMCP
from mcp.types import ToolAnnotations

HERE = Path(__file__).parent
SCORES_FILE = HERE / "model_scores.json"
CONFIG_FILE = HERE / "user_config.json"

# Walk up to repo root to find legacy context/models/ entity cards.
# The MCP lives at <repo>/context/skills/processkit/model-recommender/mcp/server.py
# so HERE.parents[4] is the repo root:
#   [0] model-recommender  [1] processkit  [2] skills
#   [3] context            [4] <repo>
REPO_ROOT = HERE.parents[4]
MODELS_DIR = REPO_ROOT / "context" / "models"

# Expose scripts/ so we can import resolver.py
_SCRIPTS_DIR = HERE.parent / "scripts"
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

server = FastMCP("processkit-model-recommender")

# ── helpers ────────────────────────────────────────────────────────────────

# Reverse map from equivalent_tier (or dims) back to the free-form "tier"
# string used by legacy tools — approximate.
_TIER_LABEL = {
    "xxl": "Frontier flagship",
    "xl":  "Frontier",
    "l":   "Mid-tier",
    "m":   "Mid-tier",
    "s":   "Small / fast",
    "xs":  "Small",
}

MODEL_CLASSES = ("fast", "standard", "powerful")

_DIM_KEY_TO_LETTER = {
    "reasoning": "R",
    "engineering": "E",
    "speed": "S",
    "breadth": "B",
    "reliability": "L",
    "governance": "G",
}


def _parse_frontmatter(text: str) -> dict | None:
    """Extract YAML frontmatter as a dict. Returns None if unparseable."""
    try:
        import yaml  # optional; only needed for artifact mode
    except ImportError:
        return None
    parts = text.split("---")
    if len(parts) < 3:
        return None
    try:
        return yaml.safe_load(parts[1])
    except Exception:
        return None


def _entity_to_legacy(entity: dict) -> list[dict]:
    """Convert a legacy Model entity card into one JSON-style dict per
    version, so the existing server tools work unchanged.
    """
    spec = entity.get("spec", {})
    provider_slug = spec.get("provider", "")
    family = spec.get("family", "")
    dims_flat = spec.get("dimensions", {}) or {}
    # Re-inflate into the legacy {score, sub} shape. Sub-dimensions are
    # lost in the artifact model; we synthesize flat sub-scores equal
    # to the top-level score so consumers keying on sub-dims still work.
    dims = {}
    for key, letter in _DIM_KEY_TO_LETTER.items():
        score = int(dims_flat.get(key, 0) or 0)
        dims[letter] = {"score": score, "sub": {}}

    tier_label = _TIER_LABEL.get(spec.get("equivalent_tier", ""), "")
    provider_display = {
        "anthropic": "Anthropic",
        "openai": "OpenAI",
        "google": "Google",
        "xai": "xAI",
        "deepseek": "DeepSeek",
        "meta": "Meta",
        "mistral": "Mistral",
        "minimax": "MiniMax",
        "alibaba": "Alibaba / Open",
        "microsoft": "Microsoft",
        "cohere": "Cohere",
    }.get(provider_slug, provider_slug.title())

    legacy_entries: list[dict] = []
    for ver in spec.get("versions", []) or []:
        vid = ver.get("version_id", "")
        legacy_id = f"{family}-{vid}" if vid and vid != "1" else family
        ctx_tokens = ver.get("context_window") or 0
        ctx_k = int(ctx_tokens // 1000) if ctx_tokens else 0
        pricing_map = ver.get("pricing_usd_per_1m", {}) or {}
        pricing = {
            "input_per_1m": pricing_map.get("input"),
            "output_per_1m": pricing_map.get("output"),
            "currency": "USD",
            "hosting": "api",
            "note": ver.get("pricing_note", ""),
        }
        legacy_entries.append({
            "id": legacy_id,
            "name": f"{family} {vid}".strip(),
            "provider": provider_display,
            "tier": spec.get("rationale") or tier_label,
            "model_classes": spec.get("model_classes") or [],
            "architecture": spec.get("architecture") or {},
            "license": spec.get("license") or {},
            "deployment": spec.get("deployment") or {},
            "supported_parameters": spec.get("supported_parameters") or [],
            "context_k": ctx_k,
            "governance_warning": ver.get("governance_warning"),
            "pricing": pricing,
            "dimensions": dims,
            "_estimated": ver.get("status") in ("preview", "beta"),
            "best_for": [],
            "avoid_for": [],
            # RoyalFern fields (DEC-20260425_2256). Pass through whatever
            # is present on the version entry; absence is meaningful (the
            # query_models filters reject models missing the field).
            "vendor_model_id": ver.get("vendor_model_id"),
            "knowledge_cutoff": ver.get("knowledge_cutoff"),
            "latency_p50_ms": ver.get("latency_p50_ms"),
            "token_accounting": ver.get("token_accounting") or {},
            "rate_limits": ver.get("rate_limits") or {},
            "jurisdiction": ver.get("jurisdiction") or {},
            "data_privacy": ver.get("data_privacy") or {},
        })
    return legacy_entries


def _load_from_artifacts() -> dict | None:
    """Load models from legacy `context/models/*.md` entity cards.

    Returns a dict in the same shape as model_scores.json (with "_meta"
    and "models") when the directory exists and has at least one
    parseable card; returns None otherwise.
    """
    if not MODELS_DIR.is_dir():
        return None
    md_files = sorted(MODELS_DIR.glob("*.md"))
    if not md_files:
        return None
    entities: list[dict] = []
    for path in md_files:
        try:
            entity = _parse_frontmatter(path.read_text())
        except Exception:
            continue
        if not entity or entity.get("kind") != "Model":
            continue
        entities.extend(_entity_to_legacy(entity))
    if not entities:
        return None
    # Preserve _meta block from JSON if present (pricing pages, status
    # pages, sub-dimension catalogue) so tools that read meta still work.
    meta: dict = {}
    if SCORES_FILE.exists():
        try:
            meta = json.loads(SCORES_FILE.read_text()).get("_meta", {})
        except Exception:
            meta = {}
    return {"_meta": meta, "models": entities}


def _load_scores() -> dict:
    # v2 treats the roster as this skill's data, not a first-class
    # processkit primitive. Prefer the packaged roster and keep the old
    # Model-entity loader only as an install/back-compat fallback.
    if SCORES_FILE.exists():
        with open(SCORES_FILE) as f:
            return json.load(f)
    from_artifacts = _load_from_artifacts()
    if from_artifacts is not None:
        return from_artifacts
    raise FileNotFoundError(f"model roster not found: {SCORES_FILE}")

def _load_config() -> dict:
    if CONFIG_FILE.exists():
        with open(CONFIG_FILE) as f:
            return json.load(f)
    return {
        "available_models": [],
        "blocked_models": [],
        "require_governance_min": 0,
        "budget_tier": "any",
        "preferred_providers": [],
        "model_classes": {},
    }

def _save_config(cfg: dict):
    with open(CONFIG_FILE, "w") as f:
        json.dump(cfg, f, indent=2)

def _bar(score: int) -> str:
    filled = score * 2
    empty = 10 - filled
    return "▓" * filled + "░" * empty

def _label(score: int) -> str:
    return {5: "Exceptional", 4: "Strong", 3: "Moderate", 2: "Limited", 1: "Minimal"}.get(score, "—")

def _apply_user_filter(models: list[dict], cfg: dict) -> list[dict]:
    available = cfg.get("available_models", [])
    blocked = cfg.get("blocked_models", [])
    preferred = cfg.get("preferred_providers", [])
    gov_min = cfg.get("require_governance_min", 0)
    budget = cfg.get("budget_tier", "any")

    result = []
    for m in models:
        if blocked and m["id"] in blocked:
            continue
        if available and m["id"] not in available:
            continue
        if gov_min and m["dimensions"]["G"]["score"] < gov_min:
            continue
        cost_score = m["dimensions"]["S"].get("sub", {}).get(
            "cost_efficiency", m["dimensions"]["S"]["score"]
        )
        if budget == "low" and cost_score < 3:
            continue
        result.append(m)

    # Prefer specified providers (stable sort — keep original order within groups)
    if preferred:
        result.sort(key=lambda m: 0 if m["provider"] in preferred else 1)
    return result

def _price_for_sort(m: dict) -> float:
    price = (m.get("pricing") or {}).get("output_per_1m")
    return float(price) if price is not None else 0.0

def _model_class_score(model_class: str, m: dict) -> float:
    dims = m["dimensions"]
    R = dims["R"]["score"]
    E = dims["E"]["score"]
    S = dims["S"]["score"]
    B = dims["B"]["score"]
    L = dims["L"]["score"]
    G = dims["G"]["score"]
    price = _price_for_sort(m)
    price_penalty = min(price / 10.0, 5.0)
    declared_bonus = 1.0 if model_class in m.get("model_classes", []) else 0.0
    if model_class == "fast":
        return (3 * S) + L + E + declared_bonus - price_penalty
    if model_class == "standard":
        return E + L + R + S + (0.5 * G) + declared_bonus - (0.5 * price_penalty)
    if model_class == "powerful":
        return (2 * R) + (2 * E) + L + B + declared_bonus - (0.25 * price_penalty)
    return 0.0

def _configured_model_ids_for_class(cfg: dict, model_class: str) -> list[str]:
    configured = (cfg.get("model_classes") or {}).get(model_class)
    if configured is None:
        return []
    if isinstance(configured, str):
        return [configured]
    if isinstance(configured, list):
        return [str(x) for x in configured]
    return []

def _select_model_for_class(
    model_class: str,
    data: dict,
    cfg: dict,
    *,
    apply_user_filter: bool,
) -> tuple[dict | None, str, list[dict]]:
    models = list(data["models"])
    visible = _apply_user_filter(models, cfg) if apply_user_filter else models
    visible_by_id = {m["id"]: m for m in visible}
    all_by_id = {m["id"]: m for m in models}

    configured = _configured_model_ids_for_class(cfg, model_class)
    for mid in configured:
        if mid in visible_by_id:
            alternatives = [
                _profile_block(m, "summary")
                for m in visible
                if m["id"] != mid
            ][:3]
            return visible_by_id[mid], "configured", alternatives

    scored = sorted(
        visible,
        key=lambda m: (-_model_class_score(model_class, m), _price_for_sort(m), m["id"]),
    )
    if not scored:
        return None, "none", []
    alternatives = [_profile_block(m, "summary") for m in scored[1:4]]
    basis = "scored"
    if configured and any(mid in all_by_id for mid in configured):
        basis = "configured_model_not_accessible"
    elif configured:
        basis = "configured_model_unknown"
    return scored[0], basis, alternatives

def _profile_block(m: dict, scope: str = "summary") -> dict:
    dims = m["dimensions"]
    top = {d: {"score": dims[d]["score"], "label": _label(dims[d]["score"]), "bar": _bar(dims[d]["score"])} for d in dims}
    out = {
        "id": m["id"],
        "name": m["name"],
        "provider": m["provider"],
        "tier": m["tier"],
        "context_k": m["context_k"],
        "governance_warning": m.get("governance_warning"),
        "dimensions": top,
    }
    # RoyalFern fields surface on every scope when present, so callers
    # can verify the fact a filter was acting on without a second call.
    for k in ("vendor_model_id", "knowledge_cutoff", "latency_p50_ms"):
        if m.get(k) is not None:
            out[k] = m[k]
    for k in (
        "model_classes",
        "architecture",
        "license",
        "deployment",
        "supported_parameters",
        "token_accounting",
        "rate_limits",
    ):
        if m.get(k):
            out[k] = m[k]
    if m.get("jurisdiction"):
        out["jurisdiction"] = m["jurisdiction"]
    if m.get("data_privacy"):
        out["data_privacy"] = m["data_privacy"]
    if scope == "full":
        out["sub_dimensions"] = {d: dims[d]["sub"] for d in dims}
        out["best_for"] = m.get("best_for", [])
        out["avoid_for"] = m.get("avoid_for", [])
    elif scope in ("R", "E", "S", "B", "L", "G"):
        out["sub_dimensions"] = {scope: dims[scope]["sub"]}
    return out

# ── tools ──────────────────────────────────────────────────────────────────

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_models(
    R: int = 0,
    E: int = 0,
    S: int = 0,
    B: int = 0,
    L: int = 0,
    G: int = 0,
    sub_R: Optional[dict] = None,
    sub_E: Optional[dict] = None,
    sub_B: Optional[dict] = None,
    min_context_k: int = 0,
    max_price_per_1m_output: Optional[float] = None,
    # ── Governance / data-privacy filters (RoyalFern, DEC-20260425_2256) ──
    phi_hipaa_eligible: Optional[bool] = None,
    pii_eligible: Optional[bool] = None,
    gdpr_eligible: Optional[bool] = None,
    training_on_customer_data: Optional[str] = None,
    data_retention_days_max: Optional[int] = None,
    jurisdiction_country_in: Optional[list[str]] = None,
    legal_regime_in: Optional[list[str]] = None,
    data_residency_in: Optional[list[str]] = None,
    max_latency_p50_ms: Optional[int] = None,
    apply_user_filter: bool = True,
    include_estimated: bool = True,
    limit: int = 5,
) -> list[dict]:
    """
    Find models that meet minimum dimension scores, ranked by overall fit.

    min_context_k: minimum context window in thousands of tokens (e.g. 200 for 200K).
      Use this when the task requires a specific context window size.
    max_price_per_1m_output: maximum acceptable output price in USD per 1M tokens.
      Self-hosted models (null price) are always included regardless of this filter.
    include_estimated: if True (default), includes models with estimated/unvalidated
      scores (marked _estimated in model_scores.json). Set False for production routing.

    Top-level requirements: pass minimum scores per dimension (1–5, 0 = no requirement).
    Sub-dimension requirements: pass dicts like sub_R={"math_reasoning": 4} to filter on
    specific sub-dimensions within Reasoning. Supported: sub_R, sub_E, sub_B.
    apply_user_filter: if True (default), respects user_config.json access/preference settings.
    limit: max results to return (default 5).

    RoyalFern governance / data-privacy filters — back the G dimension with
    auditable hard requirements pulled from each model entity's
    ``data_privacy`` / ``jurisdiction`` blocks. A model that is missing the
    relevant field is REJECTED (conservative default; absence ≠ allowed):

      phi_hipaa_eligible      — True = require vendor HIPAA BAA covers this model.
      pii_eligible            — True = require vendor explicitly permits PII.
      gdpr_eligible           — True = require GDPR-compliant processing.
      training_on_customer_data
                              — exact match against the enum
                                ('never' | 'opt-in' | 'opt-out' | 'always').
                                Use 'never' to hard-require no training on prompts.
      data_retention_days_max — integer ceiling. Matches numeric retention
                                ≤ ceiling, plus the string literal 'zero'
                                when ceiling ≥ 0. Rejects 'unknown'.
      jurisdiction_country_in — list of ISO-3166-1 alpha-2 codes; vendor HQ
                                must match one (e.g. ["US","CA","FR"]).
      legal_regime_in         — list of regime slugs (EU-GDPR, US-HIPAA,
                                CN-DSL, ...); model must declare ≥1 match.
      data_residency_in       — list of residency region codes the vendor
                                must offer (any-match).
      max_latency_p50_ms      — integer ceiling on the vendor-published p50
                                latency.

    Returns ranked list of matching model profiles (summary level).
    """
    data = _load_scores()
    cfg = _load_config() if apply_user_filter else {}
    mins = {"R": R, "E": E, "S": S, "B": B, "L": L, "G": G}
    sub_mins = {"R": sub_R or {}, "E": sub_E or {}, "B": sub_B or {}}

    candidates = []
    for m in data["models"]:
        dims = m["dimensions"]
        # Filter estimated models if requested
        if not include_estimated and m.get("_estimated"):
            continue
        # Check context window
        if min_context_k > 0 and m.get("context_k", 0) < min_context_k:
            continue
        # Check price ceiling (self-hosted models with null price always pass)
        if max_price_per_1m_output is not None:
            price = m.get("pricing", {}).get("output_per_1m")
            if price is not None and price > max_price_per_1m_output:
                continue
        # Check top-level minimums
        if any(dims[d]["score"] < mins[d] for d in mins if mins[d] > 0):
            continue
        # Check sub-dimension minimums
        skip = False
        for dim, sub_req in sub_mins.items():
            for sub_key, min_val in sub_req.items():
                if dims[dim]["sub"].get(sub_key, 0) < min_val:
                    skip = True
                    break
            if skip:
                break
        if skip:
            continue
        # RoyalFern governance / data-privacy filters
        if not _passes_royalfern_filters(
            m,
            phi_hipaa_eligible=phi_hipaa_eligible,
            pii_eligible=pii_eligible,
            gdpr_eligible=gdpr_eligible,
            training_on_customer_data=training_on_customer_data,
            data_retention_days_max=data_retention_days_max,
            jurisdiction_country_in=jurisdiction_country_in,
            legal_regime_in=legal_regime_in,
            data_residency_in=data_residency_in,
            max_latency_p50_ms=max_latency_p50_ms,
        ):
            continue
        # Compute fit score: sum of (actual - required) for required dims
        fit = sum(dims[d]["score"] - mins[d] for d in mins if mins[d] > 0)
        candidates.append((fit, m))

    candidates.sort(key=lambda x: -x[0])
    models = [m for _, m in candidates[:limit]]
    if apply_user_filter and cfg:
        models = _apply_user_filter(models, cfg)

    return [_profile_block(m, "summary") for m in models]


def _passes_royalfern_filters(
    m: dict,
    *,
    phi_hipaa_eligible: Optional[bool],
    pii_eligible: Optional[bool],
    gdpr_eligible: Optional[bool],
    training_on_customer_data: Optional[str],
    data_retention_days_max: Optional[int],
    jurisdiction_country_in: Optional[list[str]],
    legal_regime_in: Optional[list[str]],
    data_residency_in: Optional[list[str]],
    max_latency_p50_ms: Optional[int],
) -> bool:
    """Apply the RoyalFern governance filters to a single model entry.

    Returns True if the model satisfies every requested filter, False if
    any filter rejects (or if a required field is missing — conservative
    default).
    """
    privacy = m.get("data_privacy") or {}
    jurisdiction = m.get("jurisdiction") or {}

    def _bool_required(field: str, want: bool) -> bool:
        val = privacy.get(field)
        if val is None:
            return False  # missing → reject
        return bool(val) == want

    if phi_hipaa_eligible is not None and not _bool_required(
        "phi_hipaa_eligible", phi_hipaa_eligible
    ):
        return False
    if pii_eligible is not None and not _bool_required(
        "pii_eligible", pii_eligible
    ):
        return False
    if gdpr_eligible is not None and not _bool_required(
        "gdpr_eligible", gdpr_eligible
    ):
        return False

    if training_on_customer_data is not None:
        val = privacy.get("training_on_customer_data")
        if val != training_on_customer_data:
            return False

    if data_retention_days_max is not None:
        val = privacy.get("data_retention_days")
        if val is None:
            return False
        if isinstance(val, str):
            if val == "zero":
                if data_retention_days_max < 0:
                    return False
            else:
                # 'unknown' (or any other string) is not satisfiable.
                return False
        else:
            if val > data_retention_days_max:
                return False

    if jurisdiction_country_in:
        country = jurisdiction.get("vendor_hq_country")
        if country is None or country not in jurisdiction_country_in:
            return False

    if legal_regime_in:
        regimes = jurisdiction.get("applicable_legal_regimes") or []
        if not any(r in regimes for r in legal_regime_in):
            return False

    if data_residency_in:
        regions = jurisdiction.get("data_residency_regions") or []
        if not any(r in regions for r in data_residency_in):
            return False

    if max_latency_p50_ms is not None:
        latency = m.get("latency_p50_ms")
        if latency is None or latency > max_latency_p50_ms:
            return False

    return True


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_profile(model_id: str, scope: str = "full") -> dict:
    """
    Return the profile for a specific model.

    model_id: the model's id field (e.g. "claude-sonnet-4.6")
    scope: "summary" (top-level only), "full" (all sub-dimensions + best/avoid lists),
           or a single dimension letter "R", "E", "S", "B", "L", "G" (drill into that
           dimension's sub-scores only).

    Returns the profile dict or an error if the model is not found.
    """
    data = _load_scores()
    for m in data["models"]:
        if m["id"] == model_id:
            return _profile_block(m, scope)
    return {"error": f"Model '{model_id}' not found. Call list_models() to see all IDs."}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_models(apply_user_filter: bool = True) -> list[dict]:
    """
    List all models in the roster with their top-level dimension scores.

    apply_user_filter: if True (default), only returns models accessible per user_config.json.
    Set to False to see the full roster regardless of user config.

    Returns a compact list: id, name, provider, tier, context_k, dimension scores.
    """
    data = _load_scores()
    models = data["models"]
    cfg = _load_config() if apply_user_filter else {}
    if apply_user_filter and cfg.get("available_models") or cfg.get("blocked_models") or cfg.get("require_governance_min"):
        models = _apply_user_filter(models, cfg)

    return [
        {
            "id": m["id"],
            "name": m["name"],
            "provider": m["provider"],
            "tier": m["tier"],
            "context_k": m["context_k"],
            "scores": {d: m["dimensions"][d]["score"] for d in m["dimensions"]},
            "governance_warning": m.get("governance_warning"),
        }
        for m in models
    ]


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def compare_models(model_ids: list[str], scope: str = "top_level") -> dict:
    """
    Compare two or more models side by side.

    model_ids: list of model id strings (e.g. ["claude-sonnet-4.6", "gemini-2.5-pro"])
    scope: "top_level" (6 dimension scores), or a single dimension letter
           "R", "E", "S", "B", "L", "G" (compare all sub-dimensions within that dim).

    Returns a comparison dict with rows = dimensions/sub-dimensions, columns = models.
    """
    data = _load_scores()
    by_id = {m["id"]: m for m in data["models"]}

    missing = [mid for mid in model_ids if mid not in by_id]
    if missing:
        return {"error": f"Models not found: {missing}. Call list_models() to see all IDs."}

    models = [by_id[mid] for mid in model_ids]
    meta = _load_scores()["_meta"]

    if scope == "top_level":
        rows = {}
        for d in ("R", "E", "S", "B", "L", "G"):
            rows[meta["dimension_names"][d]] = {
                m["name"]: {"score": m["dimensions"][d]["score"], "label": _label(m["dimensions"][d]["score"])}
                for m in models
            }
        return {
            "scope": "top_level",
            "models": [m["name"] for m in models],
            "rows": rows,
            "winner_per_dimension": {
                meta["dimension_names"][d]: max(models, key=lambda m: m["dimensions"][d]["score"])["name"]
                for d in ("R", "E", "S", "B", "L", "G")
            },
        }
    elif scope in ("R", "E", "S", "B", "L", "G"):
        sub_keys = meta["sub_dimensions"][scope]
        rows = {}
        for sk in sub_keys:
            rows[sk] = {
                m["name"]: m["dimensions"][scope]["sub"].get(sk, 0)
                for m in models
            }
        return {
            "scope": f"sub-dimension:{scope}",
            "dimension": meta["dimension_names"][scope],
            "models": [m["name"] for m in models],
            "rows": rows,
            "top_level_scores": {m["name"]: m["dimensions"][scope]["score"] for m in models},
        }
    else:
        return {"error": f"Invalid scope '{scope}'. Use 'top_level' or a dimension letter: R E S B L G"}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_pricing(model_ids: Optional[list[str]] = None, sort_by: str = "output_per_1m") -> list[dict]:
    """
    Return pricing data for models, with a value score (capability per dollar).

    model_ids: optional list of model IDs to include. If omitted, returns all
               models accessible per user_config.
    sort_by: "output_per_1m" (default, cheapest output first), "input_per_1m",
             "value_score" (capability per dollar, highest first), "name".

    Value score = (R + E + L) / 3 divided by output cost per 1M tokens.
    Higher value score = more capability per dollar. Self-hosted models (null price)
    are listed separately since their cost depends on your infrastructure.

    Returns a list sorted by the requested field.
    """
    data = _load_scores()
    cfg = _load_config()

    if model_ids:
        models = [m for m in data["models"] if m["id"] in model_ids]
    else:
        models = _apply_user_filter(data["models"], cfg) if cfg else data["models"]

    results = []
    for m in models:
        p = m.get("pricing", {})
        inp = p.get("input_per_1m")
        out = p.get("output_per_1m")
        dims = m["dimensions"]
        # Capability score: average of capability-focused dimensions
        cap = (dims["R"]["score"] + dims["E"]["score"] + dims["L"]["score"]) / 3
        value_score = None
        if out and out > 0:
            value_score = round(cap / out * 10, 2)  # scale for readability

        results.append({
            "id": m["id"],
            "name": m["name"],
            "provider": m["provider"],
            "input_per_1m_usd": inp,
            "output_per_1m_usd": out,
            "hosting": p.get("hosting", "api"),
            "capability_score": round(cap, 1),
            "value_score": value_score,
            "value_score_note": "capability / output_cost * 10; higher = more capability per dollar; null for self-hosted",
            "pricing_note": p.get("note", ""),
            "governance_warning": m.get("governance_warning"),
        })

    # Sort
    def sort_key(x):
        if sort_by == "value_score":
            v = x["value_score"]
            return -(v if v is not None else -999)
        elif sort_by == "input_per_1m":
            v = x["input_per_1m_usd"]
            return v if v is not None else 999
        elif sort_by == "name":
            return x["name"]
        else:  # output_per_1m
            v = x["output_per_1m_usd"]
            return v if v is not None else 999

    results.sort(key=sort_key)
    return results


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=True,
))
def check_availability(model_ids: Optional[list[str]] = None) -> list[dict]:
    """
    Check provider status pages for current API availability.

    model_ids: optional list of model IDs to check. If omitted, checks all providers
               in the roster (filtered by user_config if set).

    Returns a list of {model_id, provider, status, detail} dicts.
    Status values: "operational", "degraded", "major_outage", "unknown", "no_status_page".

    Note: status is fetched from public status pages on a best-effort basis.
    Rate limits, quota exhaustion, and subscription expiry cannot be detected
    here — the user must verify those manually.
    """
    import httpx

    data = _load_scores()
    meta = data["_meta"]
    by_id = {m["id"]: m for m in data["models"]}

    if model_ids:
        targets = {mid: by_id[mid] for mid in model_ids if mid in by_id}
    else:
        cfg = _load_config()
        models = _apply_user_filter(data["models"], cfg) if cfg else data["models"]
        targets = {m["id"]: m for m in models}

    # Group by provider to avoid duplicate fetches
    provider_status: dict[str, dict] = {}
    for provider, url in meta["status_pages"].items():
        if url is None:
            provider_status[provider] = {"status": "no_status_page", "detail": "No public status page; check provider docs manually."}
            continue
        try:
            r = httpx.get(url + "/api/v2/status.json", timeout=5.0)
            if r.status_code == 200:
                body = r.json()
                indicator = body.get("status", {}).get("indicator", "unknown")
                description = body.get("status", {}).get("description", "")
                status_map = {"none": "operational", "minor": "degraded", "major": "major_outage", "critical": "major_outage"}
                provider_status[provider] = {"status": status_map.get(indicator, "unknown"), "detail": description}
            else:
                provider_status[provider] = {"status": "unknown", "detail": f"HTTP {r.status_code} from status page"}
        except Exception as e:
            provider_status[provider] = {"status": "unknown", "detail": f"Could not reach status page: {e}"}

    results = []
    for mid, m in targets.items():
        ps = provider_status.get(m["provider"], {"status": "unknown", "detail": "Provider not in status map"})
        results.append({
            "model_id": mid,
            "model_name": m["name"],
            "provider": m["provider"],
            "status": ps["status"],
            "detail": ps["detail"],
            "note": "Rate limits, quota, and subscription status must be checked manually via your API dashboard.",
        })

    return results


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_config() -> dict:
    """
    Return the current user configuration (accessible models, blocked models,
    governance floor, budget tier, preferred providers).
    """
    cfg = _load_config()
    data = _load_scores()
    by_id = {m["id"]: m["name"] for m in data["models"]}

    return {
        "available_models": [{"id": mid, "name": by_id.get(mid, mid)} for mid in cfg.get("available_models", [])],
        "blocked_models": [{"id": mid, "name": by_id.get(mid, mid)} for mid in cfg.get("blocked_models", [])],
        "require_governance_min": cfg.get("require_governance_min", 0),
        "budget_tier": cfg.get("budget_tier", "any"),
        "preferred_providers": cfg.get("preferred_providers", []),
        "model_classes": cfg.get("model_classes", {}),
        "note": "Empty available_models means all models are included (unfiltered). Add model IDs to restrict.",
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_model_for_class(
    model_class: str,
    apply_user_filter: bool = True,
    explain: bool = False,
) -> dict:
    """Resolve a provider-neutral model class to a concrete model.

    ``model_class`` must be one of ``fast``, ``standard``, or ``powerful``.
    User config may pin class mappings with ``model_classes``. Without a
    pin, the resolver scores currently visible models using class-specific
    priorities: speed/cost for fast, balanced engineering/reliability for
    standard, and reasoning/engineering for powerful.
    """
    cls = model_class.strip().lower()
    if cls not in MODEL_CLASSES:
        return {
            "error": (
                "model_class must be one of: "
                + ", ".join(MODEL_CLASSES)
            )
        }
    data = _load_scores()
    cfg = _load_config()
    selected, basis, alternatives = _select_model_for_class(
        cls,
        data,
        cfg,
        apply_user_filter=apply_user_filter,
    )
    if selected is None:
        return {"model_class": cls, "error": "no accessible models"}
    result = {
        "model_class": cls,
        "basis": basis,
        "model": _profile_block(selected, "summary"),
    }
    if explain:
        result["configured"] = _configured_model_ids_for_class(cfg, cls)
        result["alternatives"] = alternatives
    return result


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=False,
    openWorldHint=False,
))
def set_config(
    available_models: Optional[list[str]] = None,
    blocked_models: Optional[list[str]] = None,
    require_governance_min: Optional[int] = None,
    budget_tier: Optional[str] = None,
    preferred_providers: Optional[list[str]] = None,
    model_classes: Optional[dict] = None,
) -> dict:
    """
    Update user configuration. Only fields provided are changed; omitted fields keep current values.

    available_models: list of model IDs the user has API access to. Empty list = all models.
      Example: ["claude-sonnet-4.6", "claude-haiku-4.5", "gpt-4o"]
    blocked_models: model IDs to never recommend regardless of scores.
      Example: ["deepseek-v3", "deepseek-r1"]
    require_governance_min: minimum G score for all recommendations (0–5).
      Set to 5 to enforce full data sovereignty across all tasks.
    budget_tier: "any" | "low" (prefer cost_efficiency >= 3) | "medium" | "high"
    preferred_providers: list of providers to prioritize in rankings.
      Example: ["Anthropic", "Mistral"]
    model_classes: optional map from fast/standard/powerful to a model ID
      or ordered list of model IDs. Example:
      {"fast": "claude-haiku-4.5", "standard": "claude-sonnet-4.6"}

    Returns the updated configuration.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    data = _load_scores()
    valid_ids = {m["id"] for m in data["models"]}
    valid_providers = {m["provider"] for m in data["models"]}

    cfg = _load_config()

    if available_models is not None:
        bad = [mid for mid in available_models if mid not in valid_ids]
        if bad:
            return {"error": f"Unknown model IDs: {bad}. Call list_models(apply_user_filter=False) to see all valid IDs."}
        cfg["available_models"] = available_models

    if blocked_models is not None:
        bad = [mid for mid in blocked_models if mid not in valid_ids]
        if bad:
            return {"error": f"Unknown model IDs: {bad}."}
        cfg["blocked_models"] = blocked_models

    if require_governance_min is not None:
        if not (0 <= require_governance_min <= 5):
            return {"error": "require_governance_min must be 0–5."}
        cfg["require_governance_min"] = require_governance_min

    if budget_tier is not None:
        if budget_tier not in ("any", "low", "medium", "high"):
            return {"error": "budget_tier must be 'any', 'low', 'medium', or 'high'."}
        cfg["budget_tier"] = budget_tier

    if preferred_providers is not None:
        bad = [p for p in preferred_providers if p not in valid_providers]
        if bad:
            return {"error": f"Unknown providers: {bad}. Valid: {sorted(valid_providers)}"}
        cfg["preferred_providers"] = preferred_providers

    if model_classes is not None:
        bad_classes = [k for k in model_classes if k not in MODEL_CLASSES]
        if bad_classes:
            return {
                "error": (
                    "Unknown model classes: "
                    f"{bad_classes}. Valid: {list(MODEL_CLASSES)}"
                )
            }
        bad_models: list[str] = []
        normalized: dict[str, str | list[str]] = {}
        for cls, value in model_classes.items():
            ids = value if isinstance(value, list) else [value]
            ids = [str(mid) for mid in ids]
            bad_models.extend(mid for mid in ids if mid not in valid_ids)
            normalized[cls] = ids if isinstance(value, list) else ids[0]
        if bad_models:
            return {"error": f"Unknown model IDs: {bad_models}."}
        cfg["model_classes"] = normalized

    _save_config(cfg)
    return {"ok": True, "config": cfg}


# ── resolver (8-layer model-assignment binding resolution) ────────────────

def _resolver_module():
    """Import resolver lazily so the server doesn't fail when invoked outside
    of a processkit-equipped checkout (e.g. the unit tests for the legacy
    scoring tools). Cached on the function object.
    """
    if hasattr(_resolver_module, "_mod"):
        return _resolver_module._mod
    import resolver as _r  # type: ignore
    _resolver_module._mod = _r
    return _r


def _candidate_to_dict(c) -> dict:
    return {
        "model_id": c.model_id,
        "version_id": c.version_id,
        "effort": c.effort,
        "rank": c.rank,
        "source_layer": c.source_layer,
        "rationale": c.rationale,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def resolve_model(
    role: str,
    seniority: Optional[str] = None,
    team_member: Optional[str] = None,
    scope: Optional[str] = None,
    task_hints: Optional[dict] = None,
    explain: bool = False,
) -> dict:
    """Resolve the best-fit model(s) for a (role, seniority, …) call site.

    Runs the 8-layer precedence ladder from DEC-20260422_0234-LoyalComet:
    task-pin → team-member pref → project veto → capability filter →
    role+seniority → role default → project bias → shim fallback.

    Parameters
    ----------
    role:         ``ROLE-<slug>`` identifier (required)
    seniority:    junior | specialist | expert | senior | principal
    team_member:  ``TEAMMEMBER-<slug>`` override
    scope:        ``SCOPE-<slug>`` (or project slug) for project-level bindings
    task_hints:   optional dict: ``{requires_vision, requires_tool_use,
                  requires_computer_use, max_cost_usd, max_latency_ms,
                  model_pin, version_pin, effort, expected_tokens}``
    explain:      include a layer-by-layer trace in the response
    """
    r = _resolver_module()
    try:
        out = r.resolve(
            role=role, seniority=seniority, team_member=team_member,
            scope=scope, task_hints=task_hints, explain=explain,
        )
    except r.NoViableModelError as e:
        return {"candidates": [], "error": str(e)}

    if explain:
        cands, trace = out
        return {
            "candidates": [_candidate_to_dict(c) for c in cands],
            "trace": trace,
        }
    return {"candidates": [_candidate_to_dict(c) for c in out]}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def explain_routing(
    role: str,
    seniority: Optional[str] = None,
    team_member: Optional[str] = None,
    scope: Optional[str] = None,
    task_hints: Optional[dict] = None,
) -> dict:
    """Return the full 8-layer resolution trace for a (role, …) call site.

    Thin wrapper over ``resolve_model(..., explain=True)`` for the
    ``pk explain-routing`` CLI. Always includes the ``trace`` list.
    """
    return resolve_model(
        role=role, seniority=seniority, team_member=team_member,
        scope=scope, task_hints=task_hints, explain=True,
    )


# ── main ───────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    server.run(transport="stdio")
