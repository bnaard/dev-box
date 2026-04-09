# Primitive Mapping Exercise: 16 Primitives to Storage

**Status:** Draft — in discussion (DISC-001)
**Date:** 2026-03-27
**Relates to:** DISC-001, process-ontology-primitives-2026-03.md, file-per-entity-scaling-2026-03.md
**Purpose:** Map each of the 16 universal primitives to concrete file locations, YAML schemas, state machines, and storage tier classification.

---

## Design Decisions Applied

These mappings follow the tentative decisions from DISC-001:
- **Source of truth:** Markdown + YAML frontmatter (file-per-entity)
- **IDs:** Short UUID (8 hex chars), format `TYPE-xxxxxxxx`
- **Directory sharding:** `items/<type>/<YYYY>/<MM>/` for high-volume primitives
- **Storage tiers:** Hot (filesystem) / Warm (SQLite index, gitignored) / Cold (archive)
- **Narrative vs structured:** Frontmatter = structured fields; body = narrative content

## Notation

- **Required fields** are marked with `*`
- **`custom:`** is always available as a free-form YAML map for user-defined fields
- State machines use `-->` for transitions, `|` for branching
- File paths use `{id}` as placeholder for the entity's full ID (e.g., `BACK-a7f3b2c1`)

---

## 1. Work Item

**Current:** Rows in `context/BACKLOG.md` table (BACK-NNN)
**New location:** `context/items/work/{id}.md`
**Sharding:** `context/items/work/2026/03/{id}.md` (when >1K items)
**Hot/cold:** Hot while open; cold (archive) when done >90 days

### YAML Frontmatter Schema

```yaml
id: BACK-a7f3b2c1        # * short UUID
type: work-item            # * primitive type (fixed)
subtype: task              # * task | bug | feature | epic | story | chore
title: "Implement X"      # * human-readable title
state: in-progress         # * see state machine below
priority: medium           # * critical | high | medium | low
owner: "@alice"            # role reference
parent: BACK-f290e4b3     # optional parent work item
scope: PROJ-c3d4e5f6      # scope/container reference
created_at: 2026-03-27    # * ISO date
updated_at: 2026-03-27    # * ISO date
due_date: 2026-04-15      # optional deadline
estimate: M                # optional size (XS/S/M/L/XL)
tags: [backend, auth]      # category references (non-exclusive)
refs:                      # typed cross-references
  - type: blocks
    target: BACK-b2c3d4e5
  - type: implements
    target: DEC-e4f5a6b7
custom: {}                 # user-defined fields
```

### State Machine

```
draft --> ready --> in-progress --> in-review --> done
                       |                          |
                       +--> blocked               |
                       +--> cancelled -------------+
```

States: `draft`, `ready`, `in-progress`, `in-review`, `blocked`, `done`, `cancelled`
Terminal: `done`, `cancelled`
Initial: `draft`

### Markdown Body

Free-form description, acceptance criteria, notes, discussion.

---

## 2. Log Entry / Event

**Current:** No formal representation (implicit in git history)
**New location:** `context/events/` as append-only JSONL
**File:** `context/events/{YYYY}-{MM}.jsonl` (one file per month)
**Hot/cold:** Current month = hot; older months = cold (compress to .jsonl.gz)

### Schema (JSONL, not YAML frontmatter)

Events are high-volume, append-only, and primarily machine-consumed. JSONL is more appropriate than individual .md files.

```jsonl
{"id":"EVT-a1b2c3d4","ts":"2026-03-27T14:30:00Z","type":"state-change","actor":"@alice","subject":"BACK-a7f3b2c1","data":{"field":"state","from":"ready","to":"in-progress"},"source":"cli"}
{"id":"EVT-b2c3d4e5","ts":"2026-03-27T15:00:00Z","type":"comment","actor":"@alice","subject":"BACK-a7f3b2c1","data":{"body":"Started implementation"},"source":"cli"}
```

### Event Types

- `state-change` — field changed on any entity
- `comment` — narrative note on any entity
- `creation` — entity created
- `deletion` — entity removed/archived
- `gate-result` — checkpoint evaluated (pass/fail/waive)
- `metric-sample` — measurement data point
- `system` — automated events (sync, rebuild, etc.)

### State Machine

None — events are immutable. They exist or they don't.

**Note:** This is the one primitive that does NOT use file-per-entity markdown. The rationale: events are high-volume (potentially thousands per day in kaits), append-only, and primarily machine-consumed. JSONL gives efficient append + line-by-line parsing. Monthly sharding keeps files manageable. The SQLite warm index provides queryability.

---

## 3. Decision Record

**Current:** Rows in `context/DECISIONS.md` table (DEC-NNN)
**New location:** `context/items/decision/{id}.md`
**Sharding:** Unlikely to need it (decisions are low-volume)
**Hot/cold:** Hot while proposed/accepted; cold when deprecated/superseded >180 days

### YAML Frontmatter Schema

```yaml
id: DEC-f290e4b3          # * short UUID
type: decision-record      # * primitive type
title: "Use SQLite as derived index"  # * what was decided
state: accepted            # * proposed | accepted | deprecated | superseded
date: 2026-03-27           # * when decided
decision_makers:           # * who decided
  - "@owner"
  - "@claude"
superseded_by: null        # DEC-xxx if replaced
scope: PROJ-c3d4e5f6       # optional scope
tags: [architecture, storage]
refs:
  - type: affects
    target: BACK-a7f3b2c1
  - type: justified-by
    target: DISC-001
custom: {}
```

### State Machine

```
proposed --> accepted --> deprecated
                |
                +--> superseded (by DEC-xxx)
```

States: `proposed`, `accepted`, `deprecated`, `superseded`
Terminal: `deprecated`, `superseded`
Initial: `proposed`

### Markdown Body

Structured narrative sections:
- **Context** — situation requiring a decision
- **Decision** — what was decided and why
- **Alternatives** — options considered and why rejected
- **Consequences** — implications (positive, negative, risks)

---

## 4. Artifact

**Current:** Implicit — files in `context/` are artifacts but not tracked as such
**New location:** `context/items/artifact/{id}.md` (metadata file pointing to actual artifact)
**Hot/cold:** Follows the artifact it describes

### YAML Frontmatter Schema

```yaml
id: ART-c3d4e5f6          # * short UUID
type: artifact             # * primitive type
title: "API specification v2"  # * human-readable name
subtype: document          # * document | code | build | report | design | dataset
state: published           # * draft | in-review | approved | published | archived | superseded
version: "2.1.0"          # semantic or sequential
author: "@alice"           # role reference
location: "docs/api-spec.yaml"  # * path or URI to actual artifact
format: "application/yaml" # media type
scope: PROJ-c3d4e5f6
created_at: 2026-03-27
updated_at: 2026-03-27
tags: [api, documentation]
refs:
  - type: produced-by
    target: BACK-a7f3b2c1
  - type: approved-at
    target: GATE-d4e5f6a7
custom: {}
```

### State Machine

```
draft --> in-review --> approved --> published --> archived
              |                                     |
              +--> rejected (back to draft)          +--> superseded
```

States: `draft`, `in-review`, `approved`, `rejected`, `published`, `archived`, `superseded`
Terminal: `archived`, `superseded`
Initial: `draft`

### Markdown Body

Description of the artifact, changelog, review notes.

**Design note:** Most context files (research reports, work instructions) are BOTH artifacts and their own content. For these, the frontmatter IS the artifact metadata — no separate metadata file needed. The separate `items/artifact/` entry is for tracking external artifacts (builds, deployments, files outside context/).

---

## 5. Role

**Current:** `context/OWNER.md`, `context/work-instructions/TEAM.md`
**New location:** `context/items/role/{id}.md`
**Hot/cold:** Always hot (roles are reference data)

### YAML Frontmatter Schema

```yaml
id: ROLE-d4e5f6a7         # * short UUID
type: role                 # * primitive type
name: "Product Owner"      # * role name
state: active              # * active | suspended | revoked
permissions:               # what this role can do
  - "approve-decisions"
  - "prioritize-backlog"
responsibilities:          # what this role must do
  - "Maintain product backlog"
  - "Define acceptance criteria"
filled_by:                 # current holders
  - "@alice"
reports_to: ROLE-e5f6a7b8 # hierarchical superior
scope: PROJ-c3d4e5f6
tags: [leadership]
custom: {}
```

### State Machine

Role assignments (not roles themselves):
```
proposed --> active --> suspended --> revoked
```

States: `active`, `suspended`, `revoked`
Terminal: `revoked`
Initial: `active`

### Markdown Body

Detailed role description, expectations, authority boundaries.

---

## 6. Process / Workflow

**Current:** `templates/processes/*.md` (bug-fix, code-review, feature-development, release)
**New location:** `context/items/process/{id}.md` (instance-specific) + `templates/processes/*.md` (definitions)
**Hot/cold:** Definitions always hot; instances follow work item lifecycle

### YAML Frontmatter Schema (Process Definition)

```yaml
id: PROC-e5f6a7b8         # * short UUID
type: process              # * primitive type
name: "Code Review"        # * process name
version: "1.0.0"           # semantic version
state: active              # * draft | active | deprecated
trigger: "pull-request-created"  # what starts this process
inputs:                    # required artifacts
  - "source-code"
  - "test-results"
outputs:                   # produced artifacts
  - "review-comments"
  - "approval-decision"
roles:                     # participating roles
  - ROLE-d4e5f6a7          # author
  - ROLE-f6a7b8c9          # reviewer
gates:                     # quality checkpoints
  - GATE-a7b8c9d0
metrics:                   # how success is measured
  - MET-b8c9d0e1
owner: ROLE-d4e5f6a7
tags: [quality, development]
custom: {}
```

### State Machine (Process Instance)

```
not-started --> in-progress --> completed
                    |
                    +--> suspended
                    +--> aborted
                    +--> failed
```

States: `not-started`, `in-progress`, `completed`, `suspended`, `aborted`, `failed`
Terminal: `completed`, `aborted`, `failed`
Initial: `not-started`

### Markdown Body

Step-by-step procedure, decision points, guidance notes.

---

## 7. State Machine

**Current:** Implicit — hardcoded in BACKLOG.md status column
**New location:** `context/schemas/state-machines/{name}.yaml`
**Hot/cold:** Always hot (configuration data)

### Schema (Pure YAML, not markdown)

State machines are definitions, not narrative documents. Pure YAML is appropriate.

```yaml
name: work-item-default        # * machine name
description: "Default work item lifecycle"
governs: work-item             # which primitive type(s)
initial: draft
terminal: [done, cancelled]
states:
  draft:
    description: "Created but not ready for work"
  ready:
    description: "Accepted and ready to be picked up"
  in-progress:
    description: "Actively being worked on"
  in-review:
    description: "Work complete, awaiting review"
  blocked:
    description: "Cannot proceed due to dependency"
  done:
    description: "Completed and accepted"
  cancelled:
    description: "Abandoned, will not be completed"
transitions:
  - from: draft
    to: ready
    trigger: accept
  - from: ready
    to: in-progress
    trigger: start
  - from: in-progress
    to: in-review
    trigger: submit
  - from: in-progress
    to: blocked
    trigger: block
  - from: blocked
    to: in-progress
    trigger: unblock
  - from: in-review
    to: done
    trigger: approve
  - from: in-review
    to: in-progress
    trigger: reject
  - from: [draft, ready, in-progress, blocked]
    to: cancelled
    trigger: cancel
```

### State Machine

Meta: state machines don't have their own state machine. They are static definitions — either active or not.

---

## 8. Category / Taxonomy

**Current:** Ad-hoc tags in backlog, informal labels
**New location:** `context/schemas/categories/{name}.yaml`
**Hot/cold:** Always hot (configuration data)

### Schema (Pure YAML)

```yaml
name: priority                 # * category dimension name
description: "How urgent/important a work item is"
exclusive: true                # only one value per entity
hierarchical: false            # flat list, not a tree
default: medium
values:
  - id: critical
    label: "Critical"
    description: "Drop everything"
    color: "#dc2626"
  - id: high
    label: "High"
    description: "Address this cycle"
    color: "#ea580c"
  - id: medium
    label: "Medium"
    description: "Normal priority"
    color: "#ca8a04"
  - id: low
    label: "Low"
    description: "When time permits"
    color: "#16a34a"
```

Additional default categories: `subtype` (task/bug/feature/epic/story/chore), `size` (XS/S/M/L/XL), `area` (user-defined).

### State Machine

None — categories are static definitions.

---

## 9. Cross-Reference / Relation

**Current:** Informal text mentions (BACK-NNN, DEC-NNN in prose)
**New location:** Stored inline in entity frontmatter `refs:` field + materialized in SQLite index
**No dedicated file** — cross-references live on the entities they connect

### Schema (Inline in entity frontmatter)

```yaml
refs:
  - type: blocks           # * relationship type
    target: BACK-b2c3d4e5  # * target entity ID
  - type: implements
    target: DEC-e4f5a6b7
  - type: relates-to
    target: PROJ-c3d4e5f6
  - type: parent-child
    target: BACK-f290e4b3
```

### Allowed Relationship Types

- `parent-child` — hierarchical containment
- `blocks` / `blocked-by` — dependency (stored on blocker, inverse computed)
- `relates-to` — informational association
- `duplicates` — equivalence
- `supersedes` — replacement
- `implements` — realization
- `caused-by` — causal
- `references` — citation
- `produced-by` — authorship/origin
- `approved-at` — gate linkage
- `justified-by` — decision evidence

### State Machine

None — references are structural, not stateful. Created or deleted.

### Warm Index

The SQLite index maintains a bidirectional reference table for fast lookups:
`(source_id, relation_type, target_id)` with inverse computation.

---

## 10. Checkpoint / Gate

**Current:** `triggers/quality-gate.yaml` (partial)
**New location:** `context/items/gate/{id}.md`
**Hot/cold:** Definitions always hot; evaluation results are events

### YAML Frontmatter Schema

```yaml
id: GATE-a7b8c9d0         # * short UUID
type: checkpoint           # * primitive type
name: "PR Merge Readiness" # * gate name
state: active              # * active | inactive
severity: blocking         # * blocking | advisory
automated: true            # machine-checkable?
authority: ROLE-d4e5f6a7   # who can approve/waive
criteria:                  # * what must be true
  - id: tests-pass
    description: "All tests pass"
    automated: true
  - id: review-approved
    description: "At least one reviewer approved"
    automated: false
  - id: no-lint-warnings
    description: "Zero clippy warnings"
    automated: true
scope: PROJ-c3d4e5f6
tags: [quality, ci]
refs:
  - type: enforces
    target: CON-c9d0e1f2   # constraint reference
custom: {}
```

### State Machine (Gate Evaluation)

Gate evaluations are recorded as events, not as states on the gate definition. Each evaluation produces an event:

```jsonl
{"id":"EVT-xxx","type":"gate-result","subject":"GATE-a7b8c9d0","data":{"target":"BACK-a7f3b2c1","result":"passed","criteria":{"tests-pass":true,"review-approved":true,"no-lint-warnings":true}}}
```

The gate definition itself is either `active` or `inactive`.

### Markdown Body

Detailed criteria descriptions, examples, waiver policy.

---

## 11. Metric / Measure

**Current:** No formal representation
**New location:** `context/items/metric/{id}.md`
**Hot/cold:** Definitions always hot; values are events in the event log

### YAML Frontmatter Schema

```yaml
id: MET-b8c9d0e1          # * short UUID
type: metric               # * primitive type
name: "Lead Time"          # * metric name
description: "Time from ready to done"
unit: hours                # * unit of measurement
formula: "done_at - ready_at"  # how to compute
target: 48                 # desired value
thresholds:
  green: [0, 48]
  yellow: [48, 96]
  red: [96, null]
frequency: weekly          # how often computed
source_events:             # what events feed this
  - "state-change(to=done)"
  - "state-change(to=ready)"
scope: PROJ-c3d4e5f6
tags: [flow, performance]
custom: {}
```

### State Machine

None for the definition. Metric health is a computed property:
- `within-target` (green) / `approaching-threshold` (yellow) / `breached` (red)

Computed by the materializer from event data; not stored as a field.

### Markdown Body

What this metric measures, why it matters, how to improve it.

---

## 12. Schedule / Cadence

**Current:** Implicit (heartbeat concept in BACK-046)
**New location:** `context/items/schedule/{id}.md`
**Hot/cold:** Always hot (active schedules)

### YAML Frontmatter Schema

```yaml
id: SCHED-c9d0e1f2        # * short UUID
type: schedule             # * primitive type
name: "Weekly Review"      # * schedule name
subtype: cadence           # * cadence | deadline | window
state: active              # * active | paused | completed
pattern: "weekly(friday)"  # recurrence (or cron: "0 9 * * 5")
start_date: 2026-03-01
end_date: null             # null = indefinite
duration: 1h               # how long each occurrence
timezone: "Europe/Amsterdam"
triggers_process: PROC-e5f6a7b8  # what happens when it fires
owner: ROLE-d4e5f6a7
scope: PROJ-c3d4e5f6
tags: [ceremony, review]
custom: {}
```

### State Machine

```
planned --> active --> completed
               |
               +--> paused
```

States: `planned`, `active`, `paused`, `completed`
Terminal: `completed`
Initial: `planned`

### Markdown Body

Purpose of the cadence, agenda template, expected outcomes.

---

## 13. Scope / Container

**Current:** Rows in `context/PROJECTS.md` table (PROJ-NNN) + directory structure
**New location:** `context/items/scope/{id}.md`
**Hot/cold:** Hot while active; cold when archived

### YAML Frontmatter Schema

```yaml
id: PROJ-d0e1f2a3         # * short UUID (keep PROJ- prefix for project scopes)
type: scope                # * primitive type
subtype: project           # * portfolio | project | iteration | team | area
name: "aibox v1.0"        # * scope name
state: active              # * planned | active | completed | archived | cancelled
parent: null               # enclosing scope (for nesting)
owner: ROLE-d4e5f6a7
start_date: 2026-01-01
end_date: 2026-06-30       # null for ongoing
constraints:               # active constraints in this scope
  - CON-c9d0e1f2
tags: [product, release]
custom: {}
```

### State Machine

```
planned --> active --> completed --> archived
               |
               +--> suspended
               +--> cancelled
```

States: `planned`, `active`, `completed`, `suspended`, `cancelled`, `archived`
Terminal: `archived`, `cancelled`
Initial: `planned`

### Markdown Body

Scope description, goals, boundaries, success criteria.

---

## 14. Constraint

**Current:** No formal representation
**New location:** `context/items/constraint/{id}.md`
**Hot/cold:** Always hot while active

### YAML Frontmatter Schema

```yaml
id: CON-e1f2a3b4          # * short UUID
type: constraint           # * primitive type
name: "WIP Limit"          # * constraint name
subtype: policy            # * regulatory | policy | resource | time | quality | budget
state: active              # * draft | active | suspended | superseded
description: "Max 5 items in-progress per person"
severity: mandatory        # * mandatory | advisory
enforcement: automated     # * automated | manual | audit
scope: PROJ-d0e1f2a3       # where this applies
source: "team-agreement"   # where constraint comes from
violation_process: null     # process triggered on violation
tags: [flow, kanban]
refs:
  - type: enforced-at
    target: GATE-a7b8c9d0
custom: {}
```

### State Machine

```
draft --> active --> superseded
             |
             +--> suspended (temporary waiver)
```

States: `draft`, `active`, `suspended`, `superseded`
Terminal: `superseded`
Initial: `draft`

### Markdown Body

Full constraint description, rationale, exemption/waiver policy.

---

## 15. Context / Environment

**Current:** `context/` directory as a whole (CLAUDE.md, OWNER.md, PRD.md, work-instructions/, research/)
**New location:** Stays as-is — the `context/` directory IS this primitive
**Hot/cold:** Always hot (it's the active working context)

### Design Note

Context/Environment is the **meta-primitive** — it's the container for all other primitives. The `context/` directory itself, plus `CLAUDE.md` at the repo root, IS the context. Individual context artifacts (PRD, work instructions, research reports) are files that carry their own frontmatter.

Rather than creating a separate entity type, Context is represented by:
1. **`aibox.toml`** — project-level configuration (which process template, which packages enabled)
2. **`CLAUDE.md`** — agent-facing context instructions
3. **`context/`** — the directory structure containing all primitives

### YAML Frontmatter (on context documents like PRD.md, work instructions)

```yaml
type: context              # marks this as ambient context
subtype: requirements      # requirements | instructions | reference | research
title: "Product Requirements"
scope: PROJ-d0e1f2a3
updated_at: 2026-03-27
tags: [product]
```

### State Machine

None — context documents are living documents, versioned by git.

---

## 16. Discussion

**Current:** `context/discussions/DISC-001-*.md` (this is already file-per-entity!)
**New location:** `context/items/discussion/{id}.md`
**Hot/cold:** Hot while active; cold when concluded

### YAML Frontmatter Schema

```yaml
id: DISC-f2a3b4c5         # * short UUID (or sequential for rare human-authored ones)
type: discussion           # * primitive type
title: "Context System Redesign"  # * discussion topic
state: active              # * active | concluded | abandoned
date: 2026-03-27           # * when started
participants:              # * who is involved
  - "@owner"
  - "@claude"
related:                   # related backlog items
  - BACK-a7f3b2c1
  - BACK-b2c3d4e5
research:                  # research documents produced
  - "context/research/process-ontology-primitives-2026-03.md"
scope: PROJ-d0e1f2a3
tags: [architecture, context-system]
custom: {}
```

### State Machine

```
active --> concluded
   |
   +--> abandoned
```

States: `active`, `concluded`, `abandoned`
Terminal: `concluded`, `abandoned`
Initial: `active`

### Markdown Body

Train of thought, arguments, open questions, decisions made — the full discussion narrative (like DISC-001 today).

---

## Summary: Directory Structure

```
context/
  items/
    work/                    # Work Items (BACK-xxxxxxxx.md)
      2026/03/               # sharded by year/month when >1K
    decision/                # Decision Records (DEC-xxxxxxxx.md)
    artifact/                # Artifact metadata (ART-xxxxxxxx.md)
    role/                    # Role definitions (ROLE-xxxxxxxx.md)
    process/                 # Process instances (PROC-xxxxxxxx.md)
    gate/                    # Checkpoint/Gate definitions (GATE-xxxxxxxx.md)
    metric/                  # Metric definitions (MET-xxxxxxxx.md)
    schedule/                # Schedule/Cadence (SCHED-xxxxxxxx.md)
    scope/                   # Scope/Container (PROJ-xxxxxxxx.md)
    constraint/              # Constraints (CON-xxxxxxxx.md)
    discussion/              # Discussions (DISC-xxxxxxxx.md)
  events/                    # Event log (YYYY-MM.jsonl)
  schemas/
    state-machines/          # State machine definitions (YAML)
    categories/              # Category/taxonomy definitions (YAML)
  work-instructions/         # Context documents (unchanged)
  research/                  # Research artifacts (unchanged)
  archive/                   # Cold storage (unchanged structure)
  PRD.md                     # Context document
  OWNER.md                   # Role (legacy, or symlink to items/role/)
  CLAUDE.md                  # Context (agent instructions — stays at repo root)
```

## Summary: Primitive Classification

| # | Primitive | ID Prefix | Storage | Has State Machine | Volume | Shard? |
|---|-----------|-----------|---------|-------------------|--------|--------|
| 1 | Work Item | BACK- | file-per-entity (.md) | Yes (configurable) | High | Yes |
| 2 | Log Entry / Event | EVT- | JSONL (monthly) | No (immutable) | Very high | By month |
| 3 | Decision Record | DEC- | file-per-entity (.md) | Yes | Low | No |
| 4 | Artifact | ART- | file-per-entity (.md) | Yes | Medium | No |
| 5 | Role | ROLE- | file-per-entity (.md) | Yes (assignments) | Very low | No |
| 6 | Process | PROC- | file-per-entity (.md) | Yes (instances) | Low-medium | No |
| 7 | State Machine | — | YAML config | No (definition) | Very low | No |
| 8 | Category | — | YAML config | No (definition) | Very low | No |
| 9 | Cross-Reference | — | Inline in refs: | No (structural) | High | N/A |
| 10 | Checkpoint / Gate | GATE- | file-per-entity (.md) | No (eval = events) | Low | No |
| 11 | Metric | MET- | file-per-entity (.md) | No (health = computed) | Low | No |
| 12 | Schedule | SCHED- | file-per-entity (.md) | Yes | Low | No |
| 13 | Scope / Container | PROJ- | file-per-entity (.md) | Yes | Low | No |
| 14 | Constraint | CON- | file-per-entity (.md) | Yes | Low | No |
| 15 | Context / Environment | — | Directory structure | No (living docs) | N/A | N/A |
| 16 | Discussion | DISC- | file-per-entity (.md) | Yes | Very low | No |

## Summary: Three Storage Patterns

The 16 primitives map to exactly **three storage patterns**:

1. **File-per-entity markdown** (11 primitives): Work Item, Decision, Artifact, Role, Process, Gate, Metric, Schedule, Scope, Constraint, Discussion
   - YAML frontmatter + markdown body
   - Git-native diffs, blame, merge
   - SQLite warm index for queries

2. **JSONL append-only** (1 primitive): Log Entry / Event
   - High-volume, machine-consumed
   - Monthly sharding
   - Immutable records

3. **YAML configuration** (2 primitives): State Machine, Category/Taxonomy
   - Static definitions, rarely changed
   - Referenced by entities via name

Plus **2 structural primitives** with no dedicated storage:
- Cross-Reference — inline `refs:` in entity frontmatter
- Context/Environment — the directory structure itself

## Open Questions for DISC-001

1. **Migration of existing IDs:** Current BACK-001 through BACK-099 — keep as-is and only use UUIDs for new items? Or migrate all at once?
2. **OWNER.md / TEAM.md:** Keep as legacy convenience files, or fully migrate to `items/role/`?
3. **Event log format:** JSONL monthly files vs. SQLite-only (events are the one primitive where git-native may not matter since they're append-only)?
4. **Artifact self-description:** Research reports and work instructions are both artifacts AND their content. Do they get frontmatter added in-place, or do they get a separate metadata pointer in `items/artifact/`?
5. **Process definitions vs instances:** Process templates in `templates/processes/` are definitions. Running instances (e.g., "code review for PR #42") — do these get their own `items/process/` file, or are they tracked as work items with a `process:` reference?
6. **ID prefix consistency:** The table above uses BACK- for work items (legacy) but PROJ- for scopes. Should we standardize to WORK- or keep BACK- for familiarity?
