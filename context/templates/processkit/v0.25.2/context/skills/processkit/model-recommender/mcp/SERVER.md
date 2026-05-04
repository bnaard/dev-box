# model-recommender MCP server

Structured query engine for the model roster. Provides capability scoring,
sub-dimension drill-down, comparison, user access configuration, and live
provider status checks.

This server documents its own tool contract only. In gateway deployments,
the gateway must expose only the processkit servers present in the
installed/merged MCP configuration; this file is not an aggregate tool
manifest and does not imply every processkit tool is available.

## Tools

| Tool | Purpose |
|---|---|
| `list_models(apply_user_filter?)` | List all models with top-level scores; respects user config by default |
| `get_profile(model_id, scope?)` | Full or scoped profile; scope: "summary", "full", "R", "E", "S", "B", "L", "G" |
| `compare_models(model_ids, scope?)` | Side-by-side comparison; scope "top_level" or dimension letter for sub-dim drill-down |
| `query_models(R?, E?, S?, B?, L?, G?, sub_R?, sub_E?, sub_B?, task_class?, lifecycle?, apply_user_filter?, limit?)` | Find models meeting minimum scores, ranked by fit; can rank/filter by lifecycle and task suitability |
| `list_task_classes()` | List fine-grained task classes used for task-suitability routing |
| `get_pricing(model_ids?, sort_by?)` | Per-token pricing + value score (capability per dollar); sort by output cost or value |
| `check_availability(model_ids?)` | Fetch provider status pages; returns operational / degraded / major_outage / unknown |
| `get_config()` | Show current user configuration (access list, governance floor, budget, preferences) |
| `set_config(available_models?, blocked_models?, require_governance_min?, budget_tier?, preferred_providers?)` | Update user configuration |

## Storage

| File | Purpose |
|---|---|
| `context/artifacts/ART-YYYYMMDD_HHMM-ModelSpec-*.md` | Source of truth: `Artifact(kind=model-spec)` roster records with versions, scores, pricing, and metadata |
| `model_scores.json` | Packaged projection/cache and compatibility fallback; also carries shared `_meta` pages |
| `user_config.json` | User's access list, blocked models, governance floor, budget tier, preferred providers |

Older installs may still carry `context/models/MODEL-*.md` entity cards.
The server can read those as a compatibility fallback, but v2 does not
treat Model as a first-class processkit primitive.

## Running

```bash
uv run context/skills/model-recommender/mcp/server.py
```

## Key design decisions

**Governance ceiling is enforced by `query_models` and `_apply_user_filter`.**
A model that fails `require_governance_min` is excluded from all query results
even if it scores perfectly on other dimensions. This is intentional.

**`check_availability` is best-effort.** It fetches Atlassian statuspage.io
`/api/v2/status.json` endpoints. Rate limits, quota exhaustion, and subscription
expiry cannot be detected from status pages — those must be checked manually
via the provider's API dashboard.

**`apply_user_filter=False`** bypasses user_config filtering. Use this when you
want to see the full roster regardless of what the user has configured.

**Lifecycle is separate from ranking.** Models remain in the roster while they
are still offered. `active` models route normally, `legacy` models remain
available for price/compatibility niches, `deprecated` models surface
replacement metadata, `retired` models are excluded unless
`include_retired=True`, and `unverified` models follow the same conservative
path as `_estimated` records.

**Task suitability is optional.** `task_class` is a fine-grained routing hint
layered on top of the stable R/E/S/B/L/G scores. Missing suitability means
unknown, not unsuitable, unless callers set `require_task_suitability=True`.

**Sub-dimension requirements** in `query_models` are supported for R, E, and B
(the most commonly filtered). S, L, G sub-dims can be filtered post-query using
`get_profile` if needed.

## Adding new models

1. Add or update the relevant timestamped `context/artifacts/ART-YYYYMMDD_HHMM-ModelSpec-*.md`
   model-spec artifact.
2. Score all 6 dimensions and all sub-dimensions.
3. Add `pricing` fields to the version entry.
4. Set `lifecycle` (`active`, `legacy`, `deprecated`, `retired`,
   `unverified`) and source URLs.
5. Add `task_suitability` scores for the classes in
   `_meta.task_suitability_classes` where evidence is strong enough.
6. Refresh any packaged `model_scores.json` projection and bump
   `_meta.validated` to the current quarter.
7. Update `references/model-profiles.md` with the narrative profile.
8. Update the summary table in `SKILL.md` Overview.
