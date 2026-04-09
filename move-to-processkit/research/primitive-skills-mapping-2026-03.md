# Primitive-to-Skills Mapping: Agent API for the Context System

**Status:** Draft — in discussion (DISC-001)
**Date:** 2026-03-27
**Relates to:** DISC-001, process-ontology-primitives-2026-03.md, primitive-mapping-exercise-2026-03.md
**Purpose:** Map each of the 17 primitives to the skills agents need to work with them. Skills are the agent's API to the context system.

---

## Why Skills per Primitive?

In the probabilistic agent model (DISC-001 §2.24-2.32), agents interact with primitives
by reading and editing markdown files. Without skills, every agent must independently
figure out: JSONL event format, frontmatter schema, file naming conventions, sharding
paths, three-level rule structure, ID generation, RBAC interpretation, and more.

Skills encode "how to interact with this primitive correctly." They are the agent's API —
not enforced deterministically, but followed probabilistically because the instructions
are loaded into the agent's context.

**Principle:** Every primitive that an agent creates, reads, updates, or deletes should
have a corresponding skill. The skill handles the mechanical correctness so the agent
can focus on judgment and decisions.

---

## Inventory: Existing Skills vs Needed Skills

### Already Exists (needs updating for new system)

| Primitive | Existing Skill | Status | Migration Needed |
|---|---|---|---|
| Work Item | `backlog-context` | Implemented | Heavy — currently manages BACKLOG.md table, needs rewrite for file-per-entity |
| Decision Record | `decisions-adr` | Implemented | Heavy — currently manages DECISIONS.md table, needs rewrite for file-per-entity |
| Context / Environment | `context-archiving` | Implemented | Medium — archiving logic stays, file paths change |
| Role (partial) | `agent-management` | Implemented | Medium — already handles agent roles, needs Actor primitive support |
| Checkpoint / Gate (partial) | `code-review` | Implemented | Light — already a quality gate process, add gate entity awareness |
| Schedule / Cadence (partial) | `standup-context` | Implemented | Medium — standups are a cadence-triggered process |
| Log Entry / Event | `event-log` | Registered but NOT implemented | New — this is the critical missing skill |
| Actor (partial) | `owner-profile` | Registered but NOT implemented | New — needs to create/update Actor entities |

### Does Not Exist Yet (needs creation)

| Primitive | Needed Skill | Priority |
|---|---|---|
| Process / Workflow | `process-management` | High |
| Scope / Container | `scope-management` | High |
| Metric / Measure | `metrics` | Medium |
| Constraint | `constraint-management` | Medium |
| Category / Taxonomy | `taxonomy-management` | Low |
| State Machine | `state-machine-management` | Low |
| Cross-Reference / Relation | (embedded in other skills) | N/A |
| Discussion | `discussion-management` | Medium |
| Artifact | `artifact-tracking` | Low |

---

## Skill Definitions per Primitive

### 1. Work Item Skill — `workitem` (rewrite of `backlog-context`)

**What the agent does with work items:** Create, update state, update fields, assign,
prioritize, link to other items, query status, close, archive.

**Skill instructions (summary):**

- **Create:** Generate word-ID via `aibox id generate --kind WorkItem`. Create file at
  correct sharding path. Use apiVersion/kind/metadata/spec frontmatter. Write Level 1
  intro. Log creation event via `event-log` skill.
- **Update state:** Read current state. Check state machine definition (plain English
  guards). Edit `spec.state` field. Log state-change event.
- **Update fields:** Edit frontmatter fields. Log field-change event.
- **Assign:** Set `spec.owner` to actor reference. Check RBAC (can this actor take this
  role on this item?). Log assignment event.
- **Link:** Add entry to `spec.refs` array with typed relationship. Log reference event.
- **Query:** Read files directly, or suggest user run `aibox sync` then query index.
- **Close:** Transition to terminal state. Log completion event. (Archiving is separate,
  handled by `context-archiving`.)
- **RBAC awareness:** Check requesting user's Role permissions before modifying. If
  unauthorized, explain and suggest escalation per Role's `escalation` field.

**File naming:** `BACK-<word-id>.md`
**Location:** `context/items/work/` (with sharding per config)

**Dependencies:** `event-log` (for logging), `taxonomy-management` (for valid categories)

---

### 2. Event Log Skill — `event-log` (new, critical)

**What the agent does with events:** Append process events — state changes, decisions,
comments, gate evaluations, metric observations, assignment changes, anything noteworthy
that happened.

**Skill instructions (summary):**

- **When to log:** ALWAYS. Every state change, every decision, every gate evaluation,
  every notable action. This is the single most important habit. The event log is the
  source of truth for history.
- **Format:** Append one JSONL line to `context/events/YYYY-MM.jsonl`:
  ```jsonl
  {"id":"EVT-<word-id>","ts":"<ISO8601>","type":"<event-type>","actor":"<who>","subject":"<entity-id>","description":"<what happened>","data":{<structured payload>}}
  ```
- **Event types:** `state-change`, `creation`, `comment`, `assignment`, `gate-result`,
  `metric-sample`, `decision-made`, `reference-added`, `field-change`, `escalation`,
  `process-started`, `process-completed`
- **Generate event ID:** Use `aibox id generate --kind Event`
- **Actor field:** Always record who initiated the action (human user or agent identity)
- **Subject field:** Always reference the entity ID the event is about
- **Data field:** Structured payload — for state changes: `{"from":"ready","to":"in-progress"}`,
  for comments: `{"body":"..."}`, for gate results: `{"gate":"GATE-xxx","result":"passed"}`
- **Never edit past events.** Corrections are new events that reference the original.

**This skill's instructions must be prominent in all scaffolded process documentation.**
The agent should internalize: "if I did something, I log it."

**File naming:** Events don't get their own files — they're lines in monthly JSONL.
**Location:** `context/events/YYYY-MM.jsonl` (sharding per config)

**Dependencies:** None (this is the foundational skill other skills depend on)

---

### 3. Decision Record Skill — `decision` (rewrite of `decisions-adr`)

**What the agent does with decisions:** Record choices with context, alternatives,
rationale, and consequences. Track decision lifecycle (proposed → accepted → deprecated /
superseded).

**Skill instructions (summary):**

- **Create:** Generate word-ID. Create file with apiVersion/kind/metadata/spec frontmatter.
  Body follows three-level rule: Level 1 = one-line "what was decided", Level 2 = context
  and decision summary, Level 3 = alternatives, consequences, discussion.
  Log creation event.
- **Accept:** Transition state proposed → accepted. Log event.
- **Supersede:** Create new decision referencing the old. Set old decision's
  `spec.superseded_by`. Log event on both entities.
- **Link:** Decisions typically reference work items they affect (`spec.refs`).

**File naming:** `DEC-<word-id>.md`
**Location:** `context/items/decision/`

**Dependencies:** `event-log`

---

### 4. Actor Profile Skill — `actor-profile` (rewrite of `owner-profile`)

**What the agent does with actors:** Create and maintain profiles for humans and AI agents
that the project's agents work with. This is what OWNER.md and TEAM.md become.

**Skill instructions (summary):**

- **Create:** When a new human or agent joins the project. Generate word-ID. Include:
  name, type (human/ai-agent), expertise, communication preferences, working hours,
  roles filled. For AI agents: model, context window size, tool access.
- **Update:** When the agent learns new information about an actor (new expertise,
  changed preferences, new role assignment).
- **Read:** Before interacting with a human, check their actor profile for communication
  preferences. Before assigning work, check expertise.
- **RBAC context:** The actor profile links to roles. When checking permissions, the agent
  reads actor → roles → permissions/restrictions.

**File naming:** `ACTOR-<word-id>.md`
**Location:** `context/items/actor/`

**Dependencies:** `event-log`, `role-management`

---

### 5. Role Management Skill — `role-management` (new)

**What the agent does with roles:** Define roles with permissions and restrictions. Assign
actors to roles. Check permissions before actions.

**Skill instructions (summary):**

- **Create:** Define a new role with name, permissions (plain English), restrictions
  (plain English), escalation path. Log event.
- **Assign:** Link an actor to a role by adding to `spec.filled_by` on the Role and
  updating the Actor's profile. Log assignment event.
- **Check permissions:** Before modifying any entity, read the requesting user's roles
  and check permissions/restrictions. If restricted, explain and suggest escalation.
  This check should be habitual — part of every modification action.
- **Escalation:** When a request exceeds permissions, follow the role's `spec.escalation`
  field (typically: suggest asking an admin role holder).

**File naming:** `ROLE-<word-id>.md`
**Location:** `context/items/role/`

**Dependencies:** `event-log`, `actor-profile`

---

### 6. Process Management Skill — `process-management` (new)

**What the agent does with processes:** Read process definitions, follow process steps,
start and complete process instances, track which step is current.

**Skill instructions (summary):**

- **Follow:** When a process is triggered (e.g., PR created → code review process), read
  the process definition. Create a work item with `subtype: process-instance` and
  `process_def: PROC-xxx`. Follow steps in order, logging events at each step.
- **Create/Edit definition:** When the project owner asks to change a process (check RBAC
  first — typically requires admin role). Edit the process definition file directly.
- **Suggest:** When the agent recognizes a situation that matches a process trigger, suggest
  starting the process. ("This looks like a bug report — shall I follow the bug-fix process?")

**File naming:** `PROC-<word-id>.md`
**Location:** `context/items/process/`

**Dependencies:** `event-log`, `workitem` (for process instances), `role-management` (RBAC)

---

### 7. Scope Management Skill — `scope-management` (new)

**What the agent does with scopes:** Create and manage project scopes, iterations, team
boundaries. Group entities within scopes. Track scope lifecycle.

**Skill instructions (summary):**

- **Create:** New project, iteration, or team scope. Generate word-ID. Set parent scope
  for nesting. Log event.
- **Assign entities:** Set `spec.scope` on work items, artifacts, etc. to group them.
- **Status tracking:** Monitor scope state (planned → active → completed → archived).
- **Aggregate:** When asked about scope status, read all entities within the scope and
  summarize (how many items open/in-progress/done, any blockers, etc.).

**File naming:** `PROJ-<word-id>.md` (for project scopes), `SCOPE-<word-id>.md` (for
iterations, teams, areas)
**Location:** `context/items/scope/`

**Dependencies:** `event-log`, `workitem`

---

### 8. Gate / Checkpoint Skill — `gate-management` (new, extends `code-review`)

**What the agent does with gates:** Define quality checkpoints, evaluate gate criteria,
record pass/fail/waive results.

**Skill instructions (summary):**

- **Define:** Create gate with criteria list (plain English), severity (blocking/advisory),
  authority (which role can waive). Log event.
- **Evaluate:** When a work item or artifact reaches a gate, check each criterion. Record
  the result as an event via `event-log` (type: `gate-result`). If failed: explain which
  criteria failed and what needs fixing.
- **Waive:** If authority role requests a waiver, record waiver event with justification.
- **Code review as gate:** The existing `code-review` skill IS a gate evaluation. It should
  log its results as gate events.

**File naming:** `GATE-<word-id>.md`
**Location:** `context/items/gate/`

**Dependencies:** `event-log`, `role-management` (for authority checks)

---

### 9. Metric Skill — `metrics` (new)

**What the agent does with metrics:** Define metrics, compute values from events, report
on metric health, detect threshold breaches.

**Skill instructions (summary):**

- **Define:** Create metric with name, description, unit, formula (plain English), target
  value, thresholds. Log event.
- **Compute:** When asked about a metric, read relevant events from the event log and
  compute the value. For example: lead time = average time from state `ready` to state
  `done` across recent work items. Report as green/yellow/red against thresholds.
- **Sample:** Record a metric observation as an event (type: `metric-sample`).
- **Report:** When asked for a project health summary, compute all defined metrics and
  present a dashboard-style report.

**Note:** Computation is probabilistic — the agent reads events and calculates. For large
event volumes, suggest the user run `aibox sync` to populate the SQLite index and query
from there. The metric definition's formula is plain English guidance for the agent, not
executable code.

**File naming:** `MET-<word-id>.md`
**Location:** `context/items/metric/`

**Dependencies:** `event-log`

---

### 10. Constraint Skill — `constraint-management` (new)

**What the agent does with constraints:** Define rules and limits, check compliance,
flag violations.

**Skill instructions (summary):**

- **Define:** Create constraint with description, severity (mandatory/advisory), scope,
  enforcement guidance. Log event.
- **Check compliance:** Before or after actions, review applicable constraints. For example:
  WIP limit constraint → count in-progress items before starting new work. If violated,
  warn and suggest corrective action.
- **Flag violations:** When `aibox lint` or the agent detects a constraint violation, log
  an event and notify the relevant role.

**File naming:** `CON-<word-id>.md`
**Location:** `context/items/constraint/`

**Dependencies:** `event-log`, `role-management`

---

### 11. Schedule / Cadence Skill — `schedule-management` (new, extends `standup-context`)

**What the agent does with schedules:** Define cadences and deadlines, remind about
upcoming events, trigger scheduled processes.

**Skill instructions (summary):**

- **Define:** Create schedule with pattern (weekly, biweekly, etc.), trigger description,
  associated process. Log event.
- **Remind:** When a session starts, check active schedules. If a cadence is due or
  overdue, remind the user. ("Weekly review is due — last one was 8 days ago.")
- **Trigger:** When a scheduled event fires, suggest starting the associated process.
  ("Retrospective cadence reached — shall I facilitate a retro?")
- **Deadline tracking:** Monitor work items with `spec.due_date`. Warn when deadlines
  approach. Flag overdue items.

**Note:** There is no cron daemon. The agent checks schedules at the start of each session.
Cadences are advisory — the agent suggests, the human decides.

**File naming:** `SCHED-<word-id>.md`
**Location:** `context/items/schedule/`

**Dependencies:** `event-log`, `process-management`

---

### 12. Discussion Skill — `discussion-management` (new)

**What the agent does with discussions:** Facilitate structured discussions, record
arguments, track open questions, produce decisions.

**Skill instructions (summary):**

- **Create:** When a topic requires exploration with multiple perspectives, create a
  discussion entity. Link related work items and research. Log event.
- **Continue:** Append new sections to the discussion body as the conversation progresses.
  Number sections sequentially (§2.1, §2.2, etc.). Record arguments, counterarguments,
  and resolutions.
- **Produce decisions:** When a discussion point is resolved, create a Decision Record
  via the `decision` skill and link it. Update the discussion's decisions section.
- **Conclude:** When all open questions are resolved (or explicitly deferred), transition
  state to concluded. Log event.

**File naming:** `DISC-<word-id>.md`
**Location:** `context/items/discussion/`

**Dependencies:** `event-log`, `decision`

---

### 13. Taxonomy Skill — `taxonomy-management` (new)

**What the agent does with categories:** Define classification dimensions, assign
categories to entities, ensure consistency.

**Skill instructions (summary):**

- **Define:** Create category dimension (priority, type, area, size, etc.) as a YAML
  schema file. Include allowed values, whether exclusive, whether hierarchical.
- **Assign:** When creating or updating entities, use values from defined taxonomies.
  Validate against allowed values.
- **Suggest:** When the agent sees an entity without categories, suggest appropriate
  classifications based on content.
- **Evolve:** When new category values are needed, add them to the taxonomy definition
  (check RBAC — typically requires admin role).

**File naming:** `context/schemas/categories/<name>.yaml`
**Location:** Schema files, not entity files

**Dependencies:** None (foundational configuration)

---

### 14. State Machine Skill — `state-machine-management` (new)

**What the agent does with state machines:** Define allowed states and transitions,
check validity of transitions, suggest next valid states.

**Skill instructions (summary):**

- **Define:** Create state machine YAML with states, transitions, plain English guards,
  and suggestions. Check RBAC.
- **Consult:** Before transitioning an entity, read the applicable state machine. Check
  the guard (in English). If the guard suggests conditions aren't met, warn and suggest
  what to do first.
- **Suggest:** When an entity is in a given state, suggest valid next transitions.
  ("This item is in-review — valid transitions: done (approve), in-progress (reject),
  cancelled.")

**File naming:** `context/schemas/state-machines/<name>.yaml`
**Location:** Schema files, not entity files

**Dependencies:** None (foundational configuration)

---

### 15. Artifact Tracking Skill — `artifact-tracking` (new)

**What the agent does with artifacts:** Track external artifacts (builds, releases,
deployments) that don't live in the context directory. For content-primary artifacts
(research, docs), maintain frontmatter consistency.

**Skill instructions (summary):**

- **Track:** When the project produces a notable output (release, deployment, published
  doc), create an artifact entity pointing to its location. Log event.
- **Frontmatter maintenance:** When creating research reports or work instructions, ensure
  proper apiVersion/kind/metadata/spec frontmatter and three-level body structure.
- **Version:** When an artifact is updated, increment version in frontmatter. Log event.

**File naming:** `ART-<word-id>.md`
**Location:** `context/items/artifact/` (for metadata-primary), or in-place (for
content-primary with frontmatter)

**Dependencies:** `event-log`

---

### 16. Context Archiving Skill — `context-archiving` (existing, needs update)

**What the agent does:** Move completed/old entities to archive. Maintain archive
structure. Update index.

**Skill updates needed:**
- File-per-entity archiving (move individual files, not table rows)
- Archive path mirrors hot path: `context/archive/items/work/BACK-swift-oak.md`
- Log archive events via `event-log`
- Compression of old event log files (JSONL → .jsonl.gz)

**Dependencies:** `event-log`

---

### 17. Session Handover Skill — `session-handover` (existing, needs update)

**What the agent does:** At session end, produce a handover document for the next agent.

**Skill updates needed:**
- Reference entities by word-IDs
- Include summary of events logged this session
- Reference active discussions and their open questions

**Dependencies:** `event-log`

---

## Cross-Cutting Concerns

### RBAC Integration

RBAC is NOT a separate skill — it's a **cross-cutting instruction embedded in every
entity-modifying skill.** Every skill that creates, updates, or deletes an entity must
include the instruction:

> Before modifying, check the requesting user's Actor profile → Roles → permissions and
> restrictions. If the action is restricted for this role, explain why and suggest
> escalation per the Role's escalation field.

This is repeated in each skill rather than abstracted into a separate skill, because
the agent needs to encounter it at the point of action, not as a separate lookup.

### Event Logging Integration

Similarly, event logging is cross-cutting. Every entity-modifying skill includes:

> After completing this action, log it via the event-log skill. Always.

The `event-log` skill itself defines the format. Other skills reference it.

### INDEX.md Maintenance

When any skill creates or modifies files in a directory, it should update the directory's
INDEX.md if one exists. This is a cross-cutting instruction:

> After creating or modifying a file, check if the directory has an INDEX.md. If so,
> update it to reflect the change. If the directory has many files and no INDEX.md,
> suggest creating one.

---

## Skill Package Mapping (revised)

The existing 13 packages need updating. Proposed revision:

### Core packages (always scaffolded)

| Package | Skills | Primitives Covered |
|---|---|---|
| **core** | `actor-profile-management`, `role-management`, `event-log-management` | Actor, Role, Event |
| **tracking** | `workitem-management`, `decision-record-management`, `context-archiving` | Work Item, Decision, Lifecycle |

### Elective packages

| Package | Skills | Primitives Covered |
|---|---|---|
| **processes** | `process-management`, `state-machine-management`, `gate-management` | Process, State Machine, Checkpoint |
| **planning** | `scope-management`, `schedule-management`, `estimation-planning` | Scope, Schedule, Estimation |
| **governance** | `constraint-management`, `metrics-management`, `taxonomy-management` | Constraint, Metric, Category |
| **collaboration** | `discussion-management`, `standup-context`, `session-handover`, `retrospective` | Discussion, Standup, Handover, Retro |
| **artifacts** | `artifact-tracking`, `documentation` | Artifact |

### Domain packages (unchanged, still elective)

`code`, `research`, `design`, `architecture`, `security`, `data`, `operations`

### Presets (revised)

| Preset | Packages |
|---|---|
| **minimal** | core |
| **managed** | core + tracking + collaboration |
| **software** | managed + processes + code + architecture |
| **research-project** | managed + research + documentation + artifacts |
| **full-product** | managed + processes + planning + governance + collaboration + code + architecture + design + security + operations + artifacts |

---

## Skill Dependency Graph

```
               event-log-management  (Layer 0 — foundation)
                        |
          +-------------+-------------+
          |             |             |
   actor-profile-  workitem-    decision-record-
   management      management   management
          |             |             |
   role-management      |             |
          |             |             |
   +------+------+------+------+------+----------+
   |      |      |      |      |      |          |
 process- scope- gate- constraint- schedule- discussion-
 mgmt     mgmt  mgmt  management  mgmt      management
   |                    |
 state-machine-    metrics-
 management        management
```

---

## Implementation Priority

**Phase 1 — Foundation (must exist before anything else works):**
1. `event-log-management` — everything depends on this
2. `workitem-management` — rewrite of backlog-context for file-per-entity
3. `decision-record-management` — rewrite of decisions-adr for file-per-entity

**Phase 2 — Identity and access:**
4. `actor-profile-management` — rewrite of owner-profile
5. `role-management` — RBAC in plain English

**Phase 3 — Process layer:**
6. `process-management` — following/defining processes
7. `state-machine-management` — state machine awareness
8. `gate-management` — extends code-review with gate entity tracking

**Phase 4 — Planning and governance:**
9. `scope-management` — project/iteration/team scoping
10. `schedule-management` — extends standup-context with general cadences
11. `constraint-management` — rules and limits
12. `metrics-management` — metric definitions and computation
13. `taxonomy-management` — category definitions

**Phase 5 — Collaboration and lifecycle:**
14. `discussion-management` — structured discussions
15. `artifact-tracking` — external artifact metadata
16. Update `context-archiving` for file-per-entity
17. Update `session-handover` for new entity references

---

## Design Refinements (resolved in discussion)

### Skill Naming — Long descriptive names

Use `<noun>-management` for CRUD skills, `<noun>-<verb>` for action-specific skills.
Full revised naming:

| Skill | Primitive |
|---|---|
| `workitem-management` | Work Item |
| `event-log-management` | Log Entry / Event |
| `decision-record-management` | Decision Record |
| `actor-profile-management` | Actor |
| `role-management` | Role |
| `process-management` | Process / Workflow |
| `state-machine-management` | State Machine |
| `taxonomy-management` | Category / Taxonomy |
| `gate-management` | Checkpoint / Gate |
| `metrics-management` | Metric / Measure |
| `schedule-management` | Schedule / Cadence |
| `scope-management` | Scope / Container |
| `constraint-management` | Constraint |
| `discussion-management` | Discussion |
| `artifact-tracking` | Artifact |
| `context-archiving` | Context / Environment (lifecycle) |
| `session-handover` | (cross-cutting) |

### Skill Hierarchy — Layered instruction references

Skills reference lower-layer skills by name in their instructions. The `uses:` field in
frontmatter documents dependencies. Dependency is strictly downward.

```
Layer 0: event-log-management (foundation — everything depends on this)
Layer 1: role-management, actor-profile-management
Layer 2: workitem-management, decision-record-management, scope-management
Layer 3: process-management, gate-management, schedule-management
Layer 4: discussion-management, metrics-management
```

Example in `workitem-management`:
> "After creating or updating a work item, use the **event-log-management** skill to
> record what you did. Always."
> "Before modifying a work item, use the **role-management** skill to check whether
> the requesting user has permission."

Frontmatter documents this:
```yaml
---
name: workitem-management
uses:
  - event-log-management
  - role-management
  - taxonomy-management
---
```

### Skill Size — One per primitive, ~100-200 lines

Existing skills range 17-244 lines. Target ~140 lines per primitive skill. The three-level
rule means agents read only what they need (Level 1 = when to use, Level 2 = operation
overview, Level 3 = detailed schema/examples). Split only if a skill exceeds ~250 lines.

### Human Vocabulary Mapping

Each skill's "When to Use" must list all human terms that map to the primitive:

```markdown
## When to Use

Use this skill when the user asks to:
- Add/create a **backlog item**, **task**, **bug**, **feature**, **story**, **ticket**, **issue**
- Update, prioritize, assign, or close any of the above
- "What's on the backlog?" / "What am I working on?" / "Add this to the list"
```

This ensures "add a backlog item" triggers `workitem-management`.

### Cross-Cutting — Reference, don't repeat

RBAC and event-log instructions reference the respective skills by name rather than
repeating full instructions. Each entity-modifying skill includes a brief instruction
like "Before modifying, check permissions per **role-management** skill" and "After
modifying, log via **event-log-management** skill." The referenced skills contain the
detailed how-to.

### Derived Project Customization

Direct editing after scaffolding. Same principle as process files — the derived project
owns its skill files after scaffolding.

`aibox init` stores original templates in `context/.aibox/templates/<version>/`.
Each `aibox update` adds new version copies. `aibox migrate` generates diffs + migration
instructions. The derived project's agent can compare originals vs current customized
files when analysis is needed.

## Remaining Open Questions

1. **Cross-reference skill:** Cross-references are embedded in entity frontmatter (`refs:`
   field). Should there be a lightweight skill specifically for managing references
   (adding, validating, detecting broken links)? Or is this adequately covered as part
   of each entity-modifying skill?
2. **Skill testing:** How do we validate that a skill's instructions actually produce
   correct output? Manual review? Automated test cases that simulate agent interactions?
3. **Skill versioning:** Skills evolve. Should skills carry their own version in
   frontmatter? How does this interact with the template originals mechanism?
