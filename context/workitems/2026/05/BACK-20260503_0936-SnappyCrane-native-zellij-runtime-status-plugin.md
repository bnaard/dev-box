---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin
  created: '2026-05-03T09:36:27+00:00'
  labels:
    area: runtime-ui
    component: zellij
    source: owner-request
spec:
  title: Build native Zellij plugin for aibox runtime status
  state: backlog
  type: story
  priority: medium
  description: Replace the shell-pane aibox-status workaround with a small native
    Zellij plugin. The plugin should render grouped runtime status in a visually native
    status-bar style, support first-class leader-key show/hide without layout churn,
    and expose compact groups for memory pressure, CPU throttling, container uptime,
    processkit gateway/granular mode, AI agent process count, disk free space, and
    on-demand project state such as pending migrations or dirty git state.
---
