---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260424_0058-ToughGrove-feature-global-mcp-permissions
  created: '2026-04-24T00:58:47+00:00'
  updated: '2026-04-24T11:26:26+00:00'
spec:
  title: 'Feature: Global MCP permissions allow/deny list in aibox.toml'
  state: done
  type: story
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-04-24T11:26:19+00:00'
  completed_at: '2026-04-24T11:26:26+00:00'
---

## Transition note (2026-04-24T11:26:23+00:00)

All 4 phases complete: Phase 1 core infrastructure (McpConfig, pattern matching), Phase 2 (8 harness generators), Phase 3 (integration into seed.rs), Phase 4 (documentation). Feature delivers global MCP permission configuration across 8 supported harnesses with provider-independent [mcp.permissions] section in aibox.toml.
