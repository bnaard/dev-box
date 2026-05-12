---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260418_1200-CleverHarbor-skill-gate-mandatory-mcp-set
  created: 2026-04-18 12:00:00+00:00
spec:
  title: Promote skill-gate from KERNEL_MCP_SKILLS to MANDATORY_MCP_SKILLS in v0.18.6
  state: accepted
  decision: |
    `skill-gate` moves from `KERNEL_MCP_SKILLS` (fallback-only, included only
    when explicit user config doesn't disable it) to `MANDATORY_MCP_SKILLS`
    (always force-included in the merged `.mcp.json` for every harness).

    Concretely in `cli/src/processkit_vocab.rs`:
      MANDATORY_MCP_SKILLS now includes: decision-record, discussion-management,
      event-log, id-management, index-management, skill-gate, workitem-management.
      `skill-gate` remains in KERNEL_MCP_SKILLS for fallback coverage when the
      mandatory entry can't be located.
  rationale: "The PreToolUse compliance gate (`check_route_task_called.py`) blocks every\nWrite/Edit on `context/` until a session marker is present. The marker is\nonly writable via the `acknowledge_contract()` MCP tool exposed by\nskill-gate's MCP server. If skill-gate's MCP server is not registered in\nthe harness's `.mcp.json`, the gate is unsatisfiable for the entire\nsession, and every entity write has to fall back to bash+python heredoc\nworkarounds (as the last 3 sessions had to do).\n\nUntil aibox#53 (this same release), the merged `.mcp.json` was never\nwritten at all because of a flat one-level walker bug. With #53 fixed,\nthe question becomes: is skill-gate opt-in (KERNEL, requires explicit\ninclusion in the user's enabled skills) or always-on (MANDATORY)?\n\nAlways-on is the right default: the gate is the project's primary\nenforcement mechanism for the processkit compliance contract that ships\nin every project's AGENTS.md. A project that opts out of skill-gate but\nkeeps the\
    \ AGENTS.md contract has a contradictory configuration that we\nshould not silently support \u2014 the contract literally says \"call\nroute_task before any create_*/transition_*/etc.\", and route_task is\nthe very tool the gate enforces.\n"
  alternatives_considered: "1. Leave skill-gate in KERNEL only (status quo before today).\n   Rejected: same broken outcome \u2014 every project that follows the\n   canonical setup expects the gate to work, but kernel-only means\n   it works only if the user happens to enable skill-gate explicitly.\n   Surprising failure mode.\n\n2. Make skill-gate enable-on-detection (auto-add to enabled set when\n   AGENTS.md contains the compliance contract markers).\n   Rejected: implicit magic, harder to reason about, fails for\n   projects with custom AGENTS.md variants.\n\n3. Promote to MANDATORY (this decision).\n   Selected: explicit, predictable, matches the contract semantics.\n   Users who genuinely don't want the gate can fork\n   `processkit_vocab.rs` or add an opt-out config later (BACKLOG).\n"
  consequences: "+ Compliance gate works out of the box on every fresh `aibox sync`.\n+ Future agent sessions don't need bash workarounds for context/ writes.\n+ Owners of derived projects get the contract enforcement they signed\n  up for via AGENTS.md.\n\n- Slightly higher floor of always-running MCP servers (skill-gate spawns\n  a uv-managed Python subprocess on every harness session). Cost is\n  negligible \u2014 the server idles waiting for tool calls.\n- Projects that explicitly want to disable the gate (rare) need to\n  either fork the vocab module or wait for a planned opt-out config.\n"
  related:
  - aibox#53
  - BACK-20260411_1554-CleverAsh-gitignore-mcp-json-and
  - BACK-20260418_1145-CarefulFalcon-mcp-skill-name-collision-guard
  - LOG-20260418_0900-ClearHarbor-session-handover (open_threads on the gate)
  recorded_by: claude-opus-4-7
  recorded_at: 2026-04-18 12:00:00+00:00
---

# Decision summary

Promote `skill-gate` to `MANDATORY_MCP_SKILLS` in v0.18.6 so the PreToolUse
compliance gate is always satisfiable on a freshly synced project. Documented
here per AGENTS.md compliance-contract requirement to call `record_decision`
on any cross-cutting recommendation that's accepted.
