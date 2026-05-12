---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260504_0924-NeatThorn-codex-mcp-addon-fallback
  created: '2026-05-04T09:24:41+00:00'
  labels:
    area: aibox
    reported_by: owner
    surface: codex-mcp add-ons migration
  updated: '2026-05-04T09:34:42+00:00'
spec:
  title: Fix Codex subagent MCP startup crash and addon dependency migration fallback
  state: done
  type: bug
  priority: high
  description: 'Reported bugs: processkit MCP server starts still crash for Codex subagents; older derived-project aibox.toml fails during `aibox apply --no-cache` with `Addon ''preview-enhanced'' requires ''preview-archive'' addon` and leaves no actionable fallback/migration guidance. Analyze and fix both with tests.'
  started_at: '2026-05-04T09:24:44+00:00'
  completed_at: '2026-05-04T09:34:42+00:00'
---

## Transition note (2026-05-04T09:24:44+00:00)

Started immediate fix after owner reported Codex subagent MCP crash and older aibox.toml addon dependency failure.


## Transition note (2026-05-04T09:34:38+00:00)

Implementation and validation are complete; moving through review per the workitem state machine.


## Transition note (2026-05-04T09:34:42+00:00)

Focused validation passed: Codex MCP path unit test, addon fallback no-container E2E, lazygit disablement no-container E2E, and cargo clippy --all-targets -- -D warnings.
