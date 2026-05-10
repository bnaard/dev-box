---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260411_0000-TallQuail-track-aider-native-mcp
  created: '2026-04-10T22:36:51+00:00'
  labels:
    old_id: BACK-121
    area: cli
  updated: '2026-04-25T09:53:48+00:00'
spec:
  title: Track Aider native MCP client support upstream
  state: cancelled
  type: task
  priority: low
  description: 'Aider has no native MCP client today (third-party mcpm-aider bridge
    is explicitly experimental). When Aider adds native MCP client support, add a
    writer to `cli/src/mcp_registration.rs` and remove the v0.16.5 sync-time warning.
    Watch `paul-gauthier/aider` releases for MCP support. Old ID: BACK-121.'
  completed_at: '2026-04-25T09:53:48+00:00'
---

## Transition note (2026-04-25T09:53:48+00:00)

Aider native MCP client support is an upstream tracking item. Decision made to monitor via GitHub watching rather than maintain as backlog workitem. If Aider announces MCP support, reopen or create new item.
