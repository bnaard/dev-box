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
  rationale: |
    The PreToolUse compliance gate (`check_route_task_called.py`) blocks every
    Write/Edit on `context/` until a session marker is present. The marker is
    only writable via the `acknowledge_contract()` MCP tool exposed by
    skill-gate's MCP server. If skill-gate's MCP server is not registered in
    the harness's `.mcp.json`, the gate is unsatisfiable for the entire
    session, and every entity write has to fall back to bash+python heredoc
    workarounds (as the last 3 sessions had to do).

    Until aibox#53 (this same release), the merged `.mcp.json` was never
    written at all because of a flat one-level walker bug. With #53 fixed,
    the question becomes: is skill-gate opt-in (KERNEL, requires explicit
    inclusion in the user's enabled skills) or always-on (MANDATORY)?

    Always-on is the right default: the gate is the project's primary
    enforcement mechanism for the processkit compliance contract that ships
    in every project's AGENTS.md. A project that opts out of skill-gate but
    keeps the AGENTS.md contract has a contradictory configuration that we
    should not silently support — the contract literally says "call
    route_task before any create_*/transition_*/etc.", and route_task is
    the very tool the gate enforces.
  alternatives_considered: |
    1. Leave skill-gate in KERNEL only (status quo before today).
       Rejected: same broken outcome — every project that follows the
       canonical setup expects the gate to work, but kernel-only means
       it works only if the user happens to enable skill-gate explicitly.
       Surprising failure mode.

    2. Make skill-gate enable-on-detection (auto-add to enabled set when
       AGENTS.md contains the compliance contract markers).
       Rejected: implicit magic, harder to reason about, fails for
       projects with custom AGENTS.md variants.

    3. Promote to MANDATORY (this decision).
       Selected: explicit, predictable, matches the contract semantics.
       Users who genuinely don't want the gate can fork
       `processkit_vocab.rs` or add an opt-out config later (BACKLOG).
  consequences: |
    + Compliance gate works out of the box on every fresh `aibox sync`.
    + Future agent sessions don't need bash workarounds for context/ writes.
    + Owners of derived projects get the contract enforcement they signed
      up for via AGENTS.md.

    - Slightly higher floor of always-running MCP servers (skill-gate spawns
      a uv-managed Python subprocess on every harness session). Cost is
      negligible — the server idles waiting for tool calls.
    - Projects that explicitly want to disable the gate (rare) need to
      either fork the vocab module or wait for a planned opt-out config.
  related:
  - aibox#53
  - BACK-20260411_1554-cleverAsh-gitignore-mcp-json-and
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
