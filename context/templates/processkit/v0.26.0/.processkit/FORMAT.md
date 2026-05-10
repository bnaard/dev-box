---
apiVersion: processkit.projectious.work/v2
kind: Documentation
metadata:
  id: FORMAT
  title: "Entity File Format Specification"
  version: "1.0.0"
spec:
  level: 3
---

# Entity File Format

## Level 1 — What and why

Every primitive entity in processkit (WorkItem, LogEntry, DecisionRecord, Actor,
Binding, ...) is stored as a **Markdown file with a YAML frontmatter block**.
The frontmatter carries structured fields (`apiVersion`, `kind`, `metadata`,
`spec`); the Markdown body carries human-readable content. This format is
inspired by Kubernetes objects — the shape is stable, versioned, and easy for
both humans and tools to parse.

## Level 2 — The contract

### Canonical shape

```yaml
---
apiVersion: processkit.projectious.work/v2        # required — schema version
kind: WorkItem                   # required — primitive type
metadata:                        # required — identity and bookkeeping
  id: BACK-calm-fox              # required — unique within repo
  created: 2026-04-06T10:30:00Z  # required — ISO 8601 UTC
  updated: 2026-04-06T11:15:00Z  # optional — set on edit
  labels:                        # optional — arbitrary tags
    priority: high
    area: cli
spec:                            # required — primitive-specific fields
  title: "Add aibox lint command"
  state: in-progress
  assignee: ACTOR-alice
---

# Body — freeform Markdown

Human-readable description, acceptance criteria, notes, history.
```

### The four top-level keys

| Key          | Required | Purpose                                                    |
|--------------|----------|------------------------------------------------------------|
| `apiVersion` | yes      | Schema version. Enables migrations. Shipped v2 release content uses `processkit.projectious.work/v2`. |
| `kind`       | yes      | Primitive type. Determines which schema validates `spec`.  |
| `metadata`   | yes      | Identity, timestamps, labels. Cross-cutting fields.        |
| `spec`       | yes      | Entity-specific fields. Schema lives in `primitives/schemas/<kind>.yaml`. |

### metadata fields

| Field     | Required | Type              | Notes                                                   |
|-----------|----------|-------------------|---------------------------------------------------------|
| `id`      | yes      | string            | Unique identifier. Format configurable (see below).     |
| `created` | yes      | ISO 8601 datetime | UTC. Never edited after creation.                       |
| `updated` | no       | ISO 8601 datetime | Set on modification. Omitted means "same as created."   |
| `labels`  | no       | map[string]string | Arbitrary key-value tags. Used by queries and filters.  |

### ID formats

ID format is configurable in the consuming project's `processkit.toml`:

| Format                   | Example                     | `processkit.toml`                                    |
|--------------------------|-----------------------------|------------------------------------------------------|
| Word-based (default)     | `BACK-calm-fox`             | `id_format = "word"`, `id_slug = false`              |
| Word-based + slug        | `BACK-calm-fox-add-lint`    | `id_format = "word"`, `id_slug = true`               |
| UUID                     | `BACK-550e8400-e29b-41d4`   | `id_format = "uuid"`, `id_slug = false`              |
| UUID + slug              | `BACK-550e8400-add-lint`    | `id_format = "uuid"`, `id_slug = true`               |

The prefix (`BACK-`, `LOG-`, `DEC-`, ...) is determined by the primitive type
and is not configurable.

### File naming

The filename matches `metadata.id` plus the `.md` extension:

```
context/workitems/BACK-calm-fox.md
context/logs/LOG-2026-04-06-bright-owl.md
context/decisions/DEC-steady-river.md
```

Directory layout within `context/` is configurable per primitive type (sharding
by date, by state, flat, ...). Default: one directory per primitive, flat.

## Level 3 — Full reference

### apiVersion evolution

`apiVersion` follows the Kubernetes convention: `<group>/<version>`, where the
group is a reverse-DNS name anchored on the owning organization. For processkit
the group is `processkit.projectious.work`, making `processkit` a subcomponent
of the `projectious.work` organization. This prevents name collisions if other
organizations fork or publish compatible primitives under their own domains.

| apiVersion                              | Status            | Notes                                                 |
|-----------------------------------------|-------------------|-------------------------------------------------------|
| `processkit.projectious.work/v1`        | legacy            | Initial stable version and migration-source format.   |
| `processkit.projectious.work/v1beta1`   | not used          | Reserved. Use `v1` directly for now.                  |
| `processkit.projectious.work/v2`        | current           | Current shipped release contract.                     |

Migrations are handled by `aibox sync`, which generates a diff between
the old upstream reference templates (stored verbatim in
`context/templates/processkit/<old-version>/`) and the new ones,
producing prompts for the agent to apply with human approval.

### kind registry

The shipped v2 primitive kinds are:

| kind             | schema file                               | state machine? |
|------------------|-------------------------------------------|----------------|
| WorkItem         | `schemas/workitem.yaml`                   | yes            |
| LogEntry         | `schemas/logentry.yaml`                   | no (immutable) |
| DecisionRecord   | `schemas/decisionrecord.yaml`             | yes            |
| Migration        | `schemas/migration.yaml`                  | yes            |
| Artifact         | `schemas/artifact.yaml` (Phase 2)         | no             |
| Role             | `schemas/role.yaml` (Phase 2)             | no             |
| Category         | `schemas/category.yaml` (Phase 2)         | no             |
| Gate             | `schemas/gate.yaml` (Phase 2)             | no             |
| Scope            | `schemas/scope.yaml` (Phase 2)            | yes (active/archived) |
| Constraint       | `schemas/constraint.yaml` (Phase 2)       | no             |
| Context          | `schemas/context.yaml` (Phase 2)          | no             |
| Discussion       | `schemas/discussion.yaml` (Phase 2)       | yes            |
| Actor            | `schemas/actor.yaml` (Phase 2)            | no             |
| Binding          | `schemas/binding.yaml` (Phase 2)          | no             |
| CrossReference   | not a file — frontmatter-only (see below) | n/a            |

**Note on Migration:** Migration is the 19th primitive (added v0.4.0).
Each Migration entity represents a pending, in-progress, or applied
transition between two versions of an upstream source (processkit, aibox,
or community packages). The state mirrors the directory the file lives in:
`context/migrations/{pending,in-progress,applied}/`. See
[`src/skills/migration-management/`](../skills/migration-management/) for
the workflow.

`CrossReference` is the one exception: it is not stored as its own file. Simple
relationships are expressed as references in frontmatter (`blocks: [BACK-...]`,
`related: [DEC-...]`). Use `Binding` instead when the relationship has its own
scope, temporality, or attributes.

`Metric`, `Model`, `Process`, `Schedule`, and `StateMachine` are legacy
v1 migration-source kinds. v2 represents model data through the
model-recommender roster/configuration surface, process definitions as
Artifacts plus process-instance WorkItems, schedule concepts as time-window
Bindings, and metric/policy definitions as artifact-backed configuration.
Runtime state-machine YAML files remain implementation contracts; they are
not user-authored StateMachine entities.

### spec validation

`spec` is validated against `src/primitives/schemas/<kind-lowercase>.yaml`. The
schema is authoritative. `aibox lint` performs structural validation only
(`apiVersion` present, `kind` in registry, `metadata.id` present). Full
schema-aware validation is the job of the `index-management` MCP server
(Phase 3) which parses all entity files, validates against schemas, and
serves queries via a SQLite index.

### Privacy (optional `privacy:` field)

Some entities — owner profile files, personal-context files, sensitive
notes — should be visible to a user's local agent but never checked into
git or shared with collaborators. processkit recognizes three privacy
tiers via an **optional** `privacy:` field in `metadata`:

```yaml
metadata:
  id: OWNER-identity
  privacy: public            # readable by anyone with repo access (e.g. README, contributors)
  # or:
  privacy: project-private   # repo collaborators only — DEFAULT, can be omitted
  # or:
  privacy: user-private      # only the file's owner agent — must live under a `private/` directory
```

**Default:** `project-private`. Most entities omit the field.

**Filesystem rule:** any entity with `privacy: user-private` must live
under a directory named `private/` somewhere within `context/`. The aibox-
generated `.gitignore` includes the rule `context/**/private/` which
matches `private/` directories at any depth under `context/`. Validated
by `aibox lint` (Phase 4.4): a `user-private` entity outside a `private/`
directory is a lint error.

| privacy        | git-tracked? | typical use                                          |
|----------------|--------------|------------------------------------------------------|
| `public`       | yes          | identity.md, project README, public roadmap          |
| `project-private` | yes       | workitems, decisions, logs, working-style.md (default)|
| `user-private` | no           | team-and-relationships.md, personal scratch notes    |

### Cross-references vs Bindings

**Rule:** if a relationship has scope, time, or its own attributes → use a
Binding entity. If it is just "A relates to B" → use a cross-reference field
in frontmatter.

```yaml
# Cross-reference (lightweight, frontmatter-only)
spec:
  blocks: [BACK-swift-oak]           # this work item blocks that one
  related_decisions: [DEC-calm-river] # this work item references this decision
```

```yaml
# Binding (first-class entity, own file)
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-bright-falcon
spec:
  type: role-assignment              # freeform — conventions emerge per process
  subject: ACTOR-alice
  target: ROLE-developer
  scope: SCOPE-project-x             # this is what makes it a Binding, not a ref
  valid_from: 2026-01-01
  valid_until: 2026-12-31
```

See [the Binding analysis in DISC-002 §11](https://github.com/projectious-work/aibox/blob/main/context/discussions/DISC-002-aibox-refocus.md#11-investigation-binding-as-a-generalized-primitive)
for the full rationale.

### Labels and queries

`metadata.labels` is a free-form map, but some labels are conventional and
understood by the index MCP server:

| Label        | Meaning                                                   |
|--------------|-----------------------------------------------------------|
| `priority`   | `critical` / `high` / `medium` / `low`                    |
| `area`       | Which part of the project (cli, docs, ...)                |
| `type`       | Subtype of the kind (e.g. `bug` / `feature` for WorkItem) |
| `scope`      | Scope entity ID (alternative to a Binding for simple cases) |

These are conventions, not enforced. Projects may add any labels they like.

### Level 0 — where does a file live?

Every directory containing entity files should carry an `INDEX.md` that
describes what lives there and why. INDEX.md files are Level 0 content in the
three-level principle: enough for an agent to decide whether to open the
directory at all.

### Examples

See the example files referenced by each schema in `primitives/schemas/`.
