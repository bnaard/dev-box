<!-- pk-compliance v2 -->

## processkit Compliance Contract

<!-- BEGIN HOOK -->

### processkit per-turn checklist

- On session start, call `acknowledge_contract(version="v2")` once
  before any write-side processkit tool call.
- Before any sub-agent / `Task` dispatch or write-side MCP call
  (`create_*`, `transition_*`, `link_*`, `record_*`, `open_*`),
  call `route_task(task_description)` and use its recommendations.
- On any domain-relevant request, consult `skill-finder` (or call
  `find_skill(task_description)`) before acting.
- Do not hand-edit files under `context/` — use MCP tools.
- Do not browse `context/` with `ls` or `grep` — use `index-management`.
- Do not edit `context/templates/` (read-only upstream mirror).
- Full positive actions and prohibitions: see
  `context/skills/processkit/skill-gate/assets/compliance-contract.md`.

<!-- END HOOK -->

## On session start

- Call `acknowledge_contract(version="v2")` once before any write-side
  processkit tool call. This unblocks the skill-gate for the session.
- Treat each new domain-relevant request as a routing checkpoint (see
  *Tool routing*).

## Sub-agent dispatch

- Call `route_task(task_description)` before any sub-agent / `Task` /
  `Agent` dispatch; read `recommended_team_member_slug` and
  `recommended_model_class` from the response.
- Pass the recommended TeamMember slug as the sub-agent's identity where
  the harness supports it, and pick the cheapest model in the recommended
  class (Haiku < Sonnet < Opus).
- Bare-model sub-agent dispatch without a prior `route_task` call is a
  compliance miss.

## Tool routing

- Consult `skill-finder` (or call `find_skill(task_description)`) before
  acting whenever there is even a 1% chance a processkit skill applies.
- Call `route_task(task_description)` before any `create_*`,
  `transition_*`, `link_*`, `record_*`, or `open_*` tool call.
- Read entities through `index-management` (`query_entities`,
  `get_entity`, `search_entities`) when looking up entity content.

## Entity writes

- Write entities through MCP tools so schema validation, state-machine
  enforcement, and event-log auto-entry all run.
- Create the WorkItem, DecisionRecord, Note, or Artifact in the same
  turn you decide on it — deferred entity creation is lost.
- Log an event after any state change that an MCP write did not already
  produce automatically.

## Decisions

- Call `record_decision` in the same turn a cross-cutting recommendation
  is accepted.
- When the last five user messages contain explicit decision language
  (approved / decided / ship it / let's go / ok / yes / confirmed),
  either call `record_decision` in the same turn or call
  `skip_decision_record(reason=...)` to acknowledge the skip.

## Prohibitions

- Do not hand-edit files under `context/` to create or mutate entities
  (use MCP tools).
- Do not browse `context/` with `ls`, `grep`, or raw filesystem walks
  (use `index-management`).
- Do not edit any file under `context/templates/` (read-only upstream
  mirror used as a diff baseline).
- Do not hand-edit the generated harness MCP config — edit the
  per-skill `mcp-config.json` and let the installer re-merge.

## Preferred MCP entry points by task type

| Task type | Preferred MCP entry point |
|-----------|--------------------------|
| Read a single entity by ID | `get_entity(id=...)` or kind-specific `get_workitem` / `get_decision` / `get_team_member` |
| Read an entity by filesystem path | `get_entity_by_path(path=...)` |
| List entities across kinds | `list_entities(kind?, state?, limit?)` |
| Search entities by text | `search_entities(text)` or `hybrid_search_entities(text)` |
| Create / mutate an entity | `create_*` / `transition_*` / `record_*` / `open_*` tools (always route_task first) |
| Run the aggregator health check | `run_pk_doctor(check?, fix?)` |
| Run the pre-release validation sweep | `run_pk_release_audit(tree?)` |
| Dispatch a sub-agent | `route_task(task_description=...)` first → then `Agent` or `Task` with recommended model |

## Read is OK for non-entity files

The Read tool is **blocked** on canonical entity files (paths matching
`context/{workitems,decisions,artifacts,team-members,scopes,gates,actors,
roles,bindings}/*.md`). A PreToolUse hook enforces this at runtime.

Read is **allowed** (no hook block) for:
- Skill source code under `context/skills/<skill>/` — scripts, SKILL.md,
  configs, assets are all readable directly.
- Schema definitions under `context/schemas/` (reading is fine; writes
  require a Migration + DEC).
- Log entries under `context/logs/` (append-only, safe to scan).
- Applied migrations under `context/migrations/applied/`.
- TeamMember sub-files: `persona.md`, `card.json`, and everything under
  `knowledge/`, `journal/`, `skills/`, `relations/`, `lessons/`,
  `private/`, `working/`.
- Any file outside `context/` entirely (docs/, src/, README.md, etc.).
