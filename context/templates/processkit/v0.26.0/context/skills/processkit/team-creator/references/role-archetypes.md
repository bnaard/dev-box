# Role Archetypes — Full Reference

## The 8 processkit role archetypes

Each archetype maps onto a catalog Role (`context/roles/ROLE-*`) and
seniority tier via `assets/archetype-catalog-mapping.yaml`. Archetypes
are mapping keys, not Role entities; see SKILL.md §"Catalog-driven
archetype mapping (v2)" for the loading sequence.

| Role archetype | Catalog Role | Seniority | Tier pin | Rationale | Override-when | Slots opened (rank 1..N) |
|---|---|---|---|---|---|---|
| **project-manager** | `ROLE-product-manager` | senior | heavy | Routing quality compounds; every routing error propagates to all downstream work | **Never override.** PM is always heavy; always exactly one slot at rank=1 (no parallelism). | 1 |
| **senior-architect** | `ROLE-solutions-architect` | senior | heavy | Cross-subsystem design; blast radius is high; wrong architecture poisons the entire codebase | Medium only if no heavy candidate clears G-floor AND owner explicitly approves in writing. | up to `--parallelism-cap` |
| **senior-researcher** | `ROLE-research-scientist` | senior | heavy | Multi-source synthesis; wrong synthesis poisons downstream decisions at scale | Same as senior-architect. | up to `--parallelism-cap` |
| **junior-architect** | `ROLE-solutions-architect` | specialist | medium | Single-module scope; bounded blast radius | Heavy if median capability gap between medium and heavy tiers exceeds 15pp on SWE-bench Verified for the accessible candidate set. | up to `--parallelism-cap` |
| **developer** | `ROLE-software-engineer` | senior | medium | Implementation against a written plan; bounded scope | Heavy for security-critical or regulated subsystems (owner sets `--security-critical` flag). | up to `--parallelism-cap` |
| **junior-researcher** | `ROLE-research-scientist` | specialist | medium | Bounded single-topic research; output reviewed by senior-researcher | No override. | up to `--parallelism-cap` |
| **junior-developer** | `ROLE-software-engineer` | junior | light | Well-specified single-file edits; no architecture decisions | Medium if no light-tier candidate is accessible (escalation fallback). | up to `--parallelism-cap` |
| **assistant** | `ROLE-assistant` | specialist | light | Admin, formatting, summarisation, file management | No override. If no light model is accessible, the assistant slot's `default_model_profile` shares the junior-developer slot's profile. | up to `--parallelism-cap` |

## Override rules in detail

### Tier upgrade: light → medium (junior-developer)

Applied automatically when the accessible candidate set yields no
light-tier models (TierScore < 0.40). The skill selects the lowest-
scoring medium model for junior-developer to minimise cost.

### Tier upgrade: light → light (assistant shares junior-developer)

If no light-tier model is accessible, the `assistant` archetype's
RoleSlot is opened with the junior-developer slot's
`default_model_profile`. No separate Actor is created (Actors are no
longer written by team-creator v2). The DecisionRecord notes this as
a shared-model-profile arrangement.

### Tier downgrade: heavy → medium (senior-architect, senior-researcher)

Permitted only when:
1. No heavy-tier candidate passes the G-floor gate, AND
2. The owner provides explicit written approval (supply as
   `--override-reason "<text>"` argument to `pk-team-create`).

The downgrade is recorded in the chartering DecisionRecord under
`spec.override_exceptions`.

### Tier upgrade: medium → heavy (junior-architect)

Applied automatically when:
- The gap between the best medium-tier model's SWE-bench Verified
  score and the best heavy-tier model's score exceeds 15 percentage
  points, AND
- At least one heavy-tier model is accessible.

Rationale: at this gap size, assigning junior-architect to a medium
model means accepting materially inferior architectural output for
single-module scope work. The cost increment is justified.

### Tier upgrade: medium → heavy (developer — security flag)

When `--security-critical` is set, developer is treated as heavy-pinned
for that run. This is appropriate for regulated or security-sensitive
subsystems. Record in DecisionRecord `override_exceptions`.

## Parallelism cap

`--parallelism-cap` (default 5) applies to all roles **except**
`project-manager`, which is always capped at 1. The rationale is in
`DEC-20260414_0900`: PM routing quality requires a single authoritative
decision-maker; parallel PMs produce conflicting routing decisions.

Values > 5 require explicit owner approval supplied via
`--override-reason "parallelism > 5 approved for <reason>"` and are
recorded in the DecisionRecord.

## Tier-collapse rules

If the accessible model set produces fewer than 3 distinct tiers:

| Collapse scenario | Resolution |
|---|---|
| Only heavy + medium (no light) | Promote the 2 lowest-scoring medium models to "effective light." |
| Only heavy + light (no medium) | Promote the 2 highest-scoring light models to medium. |
| Only one tier (e.g. all medium) | Treat the top ⅓ as heavy, middle ⅓ as medium, bottom ⅓ as light by rank. |

The skill never fails on tier-collapse. It degrades gracefully and
records the collapse scenario in the DecisionRecord `inputs_snapshot`.

## Role responsibilities reference

Brief one-liners that summarise each archetype's intent. These are
read by the chartering DecisionRecord generator and surfaced in
roster.md narratives. The catalog Roles themselves
(`context/roles/ROLE-*`) carry their own canonical responsibilities
lists; this table is the archetype-side gloss only.

| Role | responsibilities (imperative bullets) |
|---|---|
| project-manager | Route all tasks to appropriate actors; maintain roster.md; flag team drift; resolve role conflicts |
| senior-architect | Own cross-subsystem design decisions; review all architecture PRs; define subsystem interfaces |
| senior-researcher | Synthesise multi-source research; validate research output from junior-researcher; own landscape snapshots |
| junior-architect | Design within assigned module; produce design docs for senior-architect review |
| developer | Implement tasks from design docs; write unit tests; resolve failing CI |
| junior-researcher | Research bounded single topics; produce structured summaries for senior-researcher review |
| junior-developer | Implement well-specified single-file edits; run linters; update changelogs |
| assistant | Format documents; manage file moves; write standup notes; perform admin tasks |

## Capacity in v2: RoleSlots, not Actor templates/clones

In v2 there is no "seed Actor / clone Actor" distinction — that model
was a v0.16.0 expedient and was removed at v0.19.0 along with the
backing schema fields. Capacity is now expressed as the count of
**RoleSlots** opened under the chartering Scope:
`SLOT-<scope-slug>-<role-slug>-<rank>`, with `rank=1` reserved for
the primary fill and `rank=2..N` for parallel reservations
(N = `--parallelism-cap`, except for `project-manager`, which is
hard-coded to 1).

Re-tiering an archetype rebinds the **fill** of the existing slot
(via `team-manager.fill_role_slot`); the slot itself stays open
across rebalances unless `--roles all` triggers a full re-charter,
which closes the prior chartering Scope's slots and opens a new
generation under the new Scope.

## Provider-neutrality invariant

No model name, provider name, or tier label (Opus/Sonnet/Haiku or
equivalents) may appear in this reference file or in any command. The
archetype → tier mapping is abstract. Model identifiers are always
resolved at runtime by `model-recommender` and stored in the
DecisionRecord. This invariant enables the same team composition
logic to operate against any provider mix.
