---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260424_0114-NobleSage-phase-2b-continue-cursor
  created: '2026-04-24T01:14:29+00:00'
  updated: '2026-04-24T07:12:31+00:00'
spec:
  title: 'Phase 2b: Continue + Cursor + Aider generators'
  state: done
  type: task
  priority: high
  description: |
    **Scope:** Implement per-tool permission generators for Continue, Cursor, and Aider.

    **Deliverables:**
    1. `generate_continue_permissions(config: &McpConfig) -> Result<()>`
       - Set tool mode to "Ask" (default) or "Automatic" in `continue/config.json`
    2. `generate_cursor_permissions(config: &McpConfig) -> Result<()>`
       - Update `allowedMcpServers[]` in `.cursor/settings.json`
    3. `generate_aider_permissions(config: &McpConfig) -> Result<()>`
       - Generate per-tool agent config entries

    **Key Difference:** Continue/Cursor use per-tool modes; Aider uses agent-scoped permissions.

    **Estimated Tokens:** ~2K per harness (format-specific config generation)
    **Can run in parallel:** with Phase 2a, 2c, 2d
  parent: BACK-20260424_0058-ToughGrove-feature-global-mcp-permissions
  blocked_by:
  - BACK-20260424_0114-JollyStream-phase-1-core-mcp
  started_at: '2026-04-24T07:12:30+00:00'
  completed_at: '2026-04-24T07:12:31+00:00'
---

## Transition note (2026-04-24T07:12:30+00:00)

Continue, Cursor, and Aider generators implemented with comprehensive test coverage.


## Transition note (2026-04-24T07:12:31+00:00)

Phase 2b complete. 591 tests passing. Phase 2c and 2d ready to proceed.
