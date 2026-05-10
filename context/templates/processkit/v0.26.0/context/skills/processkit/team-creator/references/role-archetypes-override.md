# role-archetypes.yaml Override — Reference

## Purpose

A project may supply a `context/team/role-archetypes.yaml` file to
remap tier pins for some or all of the 8 processkit role archetypes.
`pk-team-create` loads this file in Step 4 (Layer B) and validates it
eagerly — before model scoring — so violations fail fast.

> **Capacity is no longer overridden here.** v1's `clone_cap_override`
> field is rejected by the validator. Capacity is the number of
> RoleSlots opened in step 6 (`--parallelism-cap`, with PM pinned to
> 1). The `(role, seniority)` pair an archetype maps to is overridden
> separately via `context/team/archetype-catalog-mapping.yaml` (see
> SKILL.md §"Catalog-driven archetype mapping (v2)").

## File location

```
context/team/role-archetypes.yaml
```

The file must be committed to the project repository. If absent,
kit defaults from `references/role-archetypes.md` apply unchanged.

## Full schema

```yaml
version: "2"                     # schema version; v2 dropped clone_cap_override
override_semantics: delta         # "delta" | "replace"  (default: delta)

roles:
  <archetype-name>:               # must match one of the 8 kit archetype names
    tier_pin: heavy|medium|light  # REQUIRED per entry
    rationale: |                  # REQUIRED — non-empty; must be project-specific
      <why this project overrides the kit default for this role>
    override_when: []             # optional; inherits kit rules if omitted
```

> Files declaring `version: "1"` and/or `clone_cap_override` are
> tolerated but the `clone_cap_override` field is ignored with a
> warning logged to the chartering DEC. Capacity moved to RoleSlot
> count at v0.19.0; bump the version field on next edit.

Valid `<archetype-name>` values (exactly these 8):
`project-manager`, `senior-architect`, `senior-researcher`,
`junior-architect`, `developer`, `junior-researcher`,
`junior-developer`, `assistant`

## Override semantics

**`delta` (default):** Only the roles listed are overridden. All
other archetypes inherit kit defaults unchanged. Use this when a
project needs to change one or two roles — no need to specify the
full set.

**`replace`:** The file is the complete archetype table. All 8 role
archetypes must appear; missing roles are a hard validation error.
Required only when a project needs to restructure the full set. The
kit's `references/role-archetypes.md` is ignored entirely in
`replace` mode.

## Validation invariants (eager — checked at pk-team-create startup)

The following are hard errors, checked before model scoring begins:

| Invariant | Error message |
|---|---|
| `project-manager.tier_pin != "heavy"` | "PM tier_pin must remain heavy — immutable per DEC-20260414_0900" |
| Any `rationale` is empty or absent | "Role <name>: rationale is required in role-archetypes.yaml override" |
| `replace` mode with fewer than 8 archetypes | "replace mode requires all 8 archetypes; missing: <list>" |

The following is a warning (logged to DecisionRecord, not an error):

| Condition | Warning |
|---|---|
| Two or more roles share `tier_pin: heavy` but the accessible heavy-tier set has fewer models than heavy-pinned roles | "Heavy tier has <N> models but <M> roles are pinned heavy — some roles will share a model" |

## Examples

### Delta override (one role remapped)

```yaml
version: "2"
override_semantics: delta
roles:
  senior-architect:
    tier_pin: medium
    rationale: |
      This edge-deployment project has no accessible heavy-tier model
      with G-score ≥ 4 under the target subscription. Owner approved
      downgrade to medium per project charter 2026-Q2.
```

### Replace override (full archetype table)

```yaml
version: "2"
override_semantics: replace
roles:
  project-manager:
    tier_pin: heavy       # immutable — must remain heavy
    rationale: "Kit default retained — PM routing quality requirement unchanged."
  senior-architect:
    tier_pin: medium
    rationale: "Research lab does not need frontier models for architecture review."
  senior-researcher:
    tier_pin: heavy
    rationale: "Multi-source synthesis is the lab's primary output — max capability."
  junior-architect:
    tier_pin: medium
    rationale: "Module-scoped design; medium capability is sufficient."
  developer:
    tier_pin: medium
    rationale: "Implementation against detailed specs; medium capability is sufficient."
  junior-researcher:
    tier_pin: medium
    rationale: "Bounded research scope; medium capability matches task complexity."
  junior-developer:
    tier_pin: light
    rationale: "Kit default retained — single-file edits do not require upgrade."
  assistant:
    tier_pin: light
    rationale: "Kit default retained — admin tasks do not require capability upgrade."
```

## Audit trail

When `context/team/role-archetypes.yaml` is present, `pk-team-create`
records in the chartering DecisionRecord:

```yaml
inputs_snapshot:
  archetype_override_file: "present"      # or "absent"
  archetype_override_semantics: delta     # or "replace"
  archetype_overrides:
    - role: senior-architect
      kit_default_pin: heavy
      override_pin: medium
      rationale: "<first sentence of rationale>"
    # ... one entry per overridden role
```

This ensures every run's configuration is fully reconstructable from
its chartering DecisionRecord alone.
