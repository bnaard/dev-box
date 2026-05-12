---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260419_0807-LuckyDaisy-mcp-skill-name-collision
  created: '2026-04-19T08:07:13+00:00'
spec:
  title: "MCP skill-name collision guard \u2014 warn-and-continue, keep bare-name keys"
  state: accepted
  decision: The MCP/commands walkers (cli/src/mcp_registration.rs::collect_processkit_mcp_specs and cli/src/claude_commands.rs::collect_live_commands) continue to key on the bare skill directory name (e.g. `skill-gate`). A collision guard emits `output::warn(...)` with both category paths when a duplicate basename is encountered; first-wins is deterministic. No hard fail, no rekeying to `<category>/<skill>`.
  context: During aibox#53 (per-skill MCP config merge), both walkers were identified as vulnerable to a silent last-wins overwrite if a future processkit catalog ever shipped two skills with the same basename in different categories. The current catalog has no collisions, so the risk is prospective. Q2 of the aibox#53 architect's plan flagged this; on 2026-04-18 the project owner approved the guard scope.
  rationale: "Existing installs must not break on sync, so hard-fail is too aggressive. Rekeying to the fully-qualified `<category>/<skill>` form would be a breaking change to downstream MCP config consumers. A warn-and-continue guard with deterministic first-wins semantics catches the failure mode without cost \u2014 operators get enough information (both colliding paths) to disambiguate upstream."
  alternatives:
  - option: Hard-fail on collision
    why_not: Breaks existing installs on any upstream regression; sync becomes brittle.
  - option: Rekey to <category>/<skill>
    why_not: Breaking change for every MCP config consumer; unnecessary for a prospective problem.
  - option: Silent last-wins (status quo pre-guard)
    why_not: No operator signal when it happens; debugging a missing tool surface would be painful.
  consequences: If upstream processkit ever ships two same-basename skills in different categories, sync emits a warning and deterministically keeps the first-seen entry. Operators see both paths and can rename upstream or override locally. No silent clobber. If in the future the catalog grows large enough that collisions are likely, this decision should be revisited and fully-qualified keys considered.
  related_workitems:
  - BACK-20260418_1145-CarefulFalcon-mcp-skill-name-collision-guard
  decided_at: '2026-04-19T08:07:13+00:00'
---
