---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260424_0110-CuriousFjord-mcp-permissions-3-tier
  created: '2026-04-24T01:10:34+00:00'
  updated: '2026-04-24T01:19:23+00:00'
spec:
  title: 'MCP permissions: 3-tier architecture with per-harness generators'
  state: accepted
  decision: 'Adopt a 3-tier MCP permission architecture for aibox.toml: global defaults
    → pattern-based allow/deny → per-harness overrides. Implement per-harness generator
    functions (Claude Code, OpenCode, Continue, Cursor, Gemini, GitHub Copilot, Aider,
    Codex) to eliminate permission prompts across all 8 supported harnesses.'
  context: 'Claude Code and other harnesses continuously prompt for MCP server permissions
    (~26 prompts per harness per session) despite aibox-shipped content being trusted.
    Current workaround requires manual entry in each harness''s config. Investigation
    revealed 8 harnesses with different permission models: Claude Code (simple allowlist),
    OpenCode (3-state + wildcards), Continue (ask-first), Cursor (per-tool), Gemini
    (dual allow/deny), GitHub Copilot (CLI flags), Aider (per-tool), Codex (trust_level).'
  rationale: 'Provider-independent configuration eliminates redundant manual work
    and reduces user friction. Glob pattern matching and first-match-wins semantics
    allow intuitive configuration (e.g. allow_patterns = [\"mcp__processkit-*\"]).
    Per-harness generators let each tool use its native permission format. Pattern-based
    design future-proofs the solution: new tools can be added without code changes.
    Backward compatible with existing user configs via merge strategy (never overwrite).'
  alternatives:
  - option: Hard-code all 26 processkit MCP tools into each harness config
    summary: 'Require users to manually discover and enter each tool name. Rejected:
      poor UX, not maintainable across processkit updates, doesn''t address repeated
      manual entries across 8 harnesses.'
  - option: Per-harness config files only (no central aibox.toml point)
    summary: 'Each harness gets its own MCP config outside aibox.toml. Rejected: violates
      provider-independence principle, harder to edit, no single source of truth.'
  - option: One-size-fits-all permission model (no per-harness override)
    summary: 'Force all harnesses to use same model via synthetic translation. Rejected:
      some harnesses like Cursor don''t support per-tool granularity; creates unnecessary
      limitations.'
  - option: Continue asking users to edit harness configs manually
    summary: 'Document patterns, let users copy-paste per harness. Rejected: current
      status quo, poor UX, error-prone, violates DRY principle.'
  related_workitems:
  - BACK-20260424_0058-ToughGrove-feature-global-mcp-permissions
  decided_at: '2026-04-24T01:19:23+00:00'
---
