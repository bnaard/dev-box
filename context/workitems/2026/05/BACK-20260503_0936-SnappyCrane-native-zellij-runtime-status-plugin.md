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
  updated: '2026-05-03T13:30:24+00:00'
spec:
  title: Build native Zellij plugin for aibox runtime status
  state: review
  type: story
  priority: medium
  description: Replace the shell-pane aibox-status workaround with a small native
    Zellij plugin. The plugin should render grouped runtime status in a visually native
    status-bar style, support first-class leader-key show/hide without layout churn,
    and expose compact groups for memory pressure, CPU throttling, container uptime,
    processkit gateway/granular mode, AI agent process count, disk free space, and
    on-demand project state such as pending migrations or dirty git state.
  started_at: '2026-05-03T13:12:59+00:00'
---

## Transition note (2026-05-03T13:12:59+00:00)

Implementation started after owner released the hold. Using the approved three-lane plan with a maximum of three parallel agents.


## Transition note (2026-05-03T13:30:24+00:00)

Native Zellij plugin implementation is code-complete for review: added image-scoped Rust/WASM plugin crate, image build packaging to /usr/local/share/aibox/zellij/aibox-status.wasm, generated layout/keybinding integration for two rows, JSON status backend, docs, and validation commands. Runtime .aibox-home apply reported existing local conflicts, so active mounted runtime files were not overwritten.
