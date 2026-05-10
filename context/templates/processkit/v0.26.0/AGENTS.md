# AGENTS.md

<!-- pk-compliance-contract v2 BEGIN -->
<!-- pk-compliance v2 -->

## processkit Compliance Contract

Call `route_task(task_description)` before any `create_*`,
`transition_*`, `link_*`, `record_*`, or `open_*` tool call.

If there is even a 1% chance a processkit skill applies to the current
task, consult `skill-finder` (or call `find_skill`) before acting.

When you decide to create a WorkItem, DecisionRecord, Note, or Artifact,
call the tool in the same turn — deferred entity creation is lost.

Write entities through MCP tools, not by hand-editing files under
`context/` — hand edits bypass schema validation, state-machine
enforcement, and the event-log auto-entry.

Read entities through `index-management` (`query_entities`,
`get_entity`, `search_entities`) — do not use `ls`, `grep`, or raw
filesystem walks under `context/`.

Log an event after any state change that an MCP write did not already
produce automatically.

After a cross-cutting recommendation is accepted, call `record_decision`
in the same turn.

When the last five user messages contain explicit decision language
(approved / decided / ship it / let's go / ok / yes / confirmed),
either call `record_decision` in the same turn or call
`skip_decision_record(reason=...)` to acknowledge the skip.

Do not edit any file under `context/templates/` — it is a read-only
upstream mirror used as a diff baseline.

Do not hand-edit the generated harness MCP config — edit the per-skill
`mcp-config.json` and let the installer re-merge.
<!-- pk-compliance-contract v2 END -->

## About & session start

processkit is a versioned, provider-neutral library of process
primitives, skills, and MCP servers consumed by aibox and dogfooded
here. Run `pk-resume` before acting. Provider-specific files
(`CLAUDE.md`, `CODEX.md`, `.cursor/rules`) are thin pointers — edit
**this** file.

## Skill guards (if/then)

- Editing/creating a file under `context/` → call `find_skill`; never
  hand-edit entity YAML.
- Creating or transitioning any entity (WorkItem, Decision, Note,
  Artifact, Discussion, Scope, Gate, Binding) → use the relevant
  `*-management` MCP tool.
- Pending migration under `context/migrations/pending/` → use
  `migration-management` MCP; don't move files by hand.
- Cross-cutting recommendation accepted → `record_decision` or
  `skip_decision_record(reason=...)` same turn.
- Authoring/reviewing a skill → `skill-builder` / `skill-reviewer`.
- Status briefing / standup / wrap-up → `status-briefing` /
  `standup-context` / `session-handover`.
- Retro / retrospective / post-release review / post-mortem →
  `retrospective` skill (`/pk-retro`).
- Any domain-specific task (PRD, audit, research ingest, discussion,
  backlog add) → `find_skill` first; see the six mandatory skill-check
  classes in `skill-gate/SKILL.md`.
- Otherwise → browse `context/skills/INDEX.md` before falling back to
  general knowledge.
- Schema-invalid LogEntry repair → prefer
  `pk-doctor --fix=schema_filename --yes` (narrow, known-safe patches
  like inserting `actor: system` for pre-TeamMember logs). Direct
  hand-edit of a schema-invalid entry is permitted as an escape
  hatch; commit with a clear reference. LogEntries remain
  append-only for the normal write path.

## Sub-agent delegation

Harness `Agent`-tool sub-agents inherit the main session's permission
policy but run ephemerally — any unallowed tool call errors
immediately rather than prompting. Delegate **read-only** work to
sub-agents (Read, search Bash, MCP `query_*` / `get_*` / `search_*` /
`list_*`); keep **mutating** work on the main session (Write, Edit,
new `mkdir`, MCP `create_*` / `transition_*` / `record_*` / `link_*`
/ `open_*` / `log_event`, `git` mutations). A sub-agent permission
block is not a cue to broaden the allowlist — move the write back to
main.

## Setup

```sh
npm --prefix docs-site install && npm --prefix docs-site run build
uv run scripts/smoke-test-servers.py
```

<!-- pk-commands BEGIN -->
<!--
build: "npm --prefix docs-site run build"
test: "uv run scripts/smoke-test-servers.py"
lint: ""
fmt: ""
typecheck: ""
-->
<!-- pk-commands END -->

## Code style & PRs

Hard-wrap Markdown/Python/YAML at 80 cols (exempt: tables, URLs,
frontmatter, code fences). Conventional Commits; never `--no-verify`.
`src/` ships to consumers, `context/` is local — never mix. Preferences
live in per-skill `context/skills/<name>/config/settings.toml`. PRs:
link WorkItem ID, squash-merge, green tests before merge.

## Team

51-role catalog under `context/roles/`, with **pure-ordinal seniority**
(`junior → specialist → expert → senior → principal`). Persistent
identities live as **TeamMembers** under `context/team-members/<slug>/`
(directory tree: persona + A2A card + tiered memory). Ad-hoc
invocations are ephemeral `(role, seniority)` dispatches resolved via
`model-recommender.resolve_model` against `model-assignment` bindings.
Role and TeamMember defaults bind to provider-neutral
`Artifact(kind=model-profile)` entities; concrete
`Artifact(kind=model-spec)` candidates are selected after runtime access
gates. Model-spec filenames may encode provider/model names; model
profiles, actors, roles, and team-member identities must not.
If `team-manager.get_active_interlocutor` returns a configured
TeamMember, show that identity at session start; otherwise state that
the current speaker is an ephemeral harness agent.

Charters: `DEC-20260422_0233-SpryTulip` (team-member model + memory),
`DEC-20260422_0234-BraveFalcon` (role catalog + seniority),
`DEC-20260422_0234-LoyalComet` (model artifacts + binding routing),
`DEC-20260503_1829-LoyalComet` (provider-neutral model profiles).
See [`context/team/roster.md`](context/team/roster.md) and
[`context/skills/processkit/team-manager/SKILL.md`](context/skills/processkit/team-manager/SKILL.md).

## Project-specific notes

- Required MCP servers: `index-management`, `id-management`,
  `workitem-management`, `discussion-management`, `decision-record`,
  `event-log`, `skill-finder`, `task-router`.
- Never edit `.devcontainer/Dockerfile`; use `Dockerfile.local`.
- `apiVersion` locked through v1.x; `v2` requires a full migration.
- `_find_lib()` uses cwd; smoke tests `os.chdir()` before invoking servers.

## MCP config manifest

`context/.processkit-mcp-manifest.json` records a sha256 per
`context/skills/*/*/mcp/mcp-config.json` plus an `aggregate_sha256` over
all of them. It is regenerated at release time by
`scripts/generate-mcp-manifest.py` and mirrored into
`src/context/` so consumers receive it in the release tarball.
Downstream installers (notably `aibox sync`) are expected to compare the
aggregate hash against their last-merged state and re-merge `.mcp.json`
when they differ — independently of whether the processkit version
changed. Without this signal, per-skill MCP-config edits made within a
release cycle never reach derived projects until the next version bump.
Tracking issue: [projectious-work/aibox#54](https://github.com/projectious-work/aibox/issues/54).
The `mcp_config_drift` pk-doctor check validates the manifest locally.

---

<sub>Scaffolded by processkit `v0.18.1` on `2026-04-17`. Re-rendered on each installer sync.</sub>
