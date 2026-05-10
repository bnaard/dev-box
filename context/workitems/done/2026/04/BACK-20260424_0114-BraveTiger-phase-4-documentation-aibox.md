---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260424_0114-BraveTiger-phase-4-documentation-aibox
  created: '2026-04-24T01:14:40+00:00'
  updated: '2026-04-24T11:25:50+00:00'
spec:
  title: 'Phase 4: Documentation (aibox.toml ref, AGENTS.md, troubleshooting)'
  state: done
  type: task
  priority: medium
  description: |
    **Scope:** Document new `[mcp]` configuration and per-harness behavior.

    **Deliverables:**
    1. Update `docs-site/docs/reference/aibox-toml.md` with `[mcp]` section schema
       - Example: `allow_patterns = ["mcp__processkit-*", "bash"]`
       - Per-harness override syntax
    2. Update `AGENTS.md` with new MCP permission behavior (5-10 lines)
    3. Create troubleshooting section: "Why am I still seeing permission prompts?"
       - Document per-harness behavior differences
       - Pattern matching limitations per harness
    4. Add inline code comments explaining harness-specific semantic differences

    **Blocker:** Requires Phase 3 completion (must document actual behavior, not planned)

    **Estimated Tokens:** ~2K (reference docs, examples, troubleshooting guide)
  parent: BACK-20260424_0058-ToughGrove-feature-global-mcp-permissions
  blocked_by:
  - BACK-20260424_0114-JollyDew-phase-3-integration-into
  started_at: '2026-04-24T11:21:25+00:00'
  completed_at: '2026-04-24T11:25:50+00:00'
---

## Transition note (2026-04-24T11:25:47+00:00)

All four deliverables complete: configuration.md updated with [mcp.permissions] schema and per-harness examples, AGENTS.md brief added, comprehensive troubleshooting section created with harness-specific behavior for all 8 harnesses.
