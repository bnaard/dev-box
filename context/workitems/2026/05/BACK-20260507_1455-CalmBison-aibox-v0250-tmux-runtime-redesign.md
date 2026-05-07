---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  created: '2026-05-07T14:55:49+00:00'
  updated: '2026-05-07T14:56:15+00:00'
spec:
  title: Implement aibox v0.25.0 tmux runtime redesign
  state: in-progress
  type: epic
  priority: high
  description: 'Replace Zellij completely with tmux for aibox v0.25.0. Remove Zellij
    binaries, KDL layouts/themes, WASM plugin status rows, permission caches, terminal
    profiles, docs, and tests. Introduce pinned preinstalled tmux plugins and tmux-native
    generated runtime config while preserving the sidecar container and visual testing
    paradigm. Related decision: DEC-20260507_1447-VastLeaf-remove-zellij-and-rebuild-aibox-runtime.'
  scope: runtime-architecture
  started_at: '2026-05-07T14:56:15+00:00'
---

## Transition note (2026-05-07T14:56:15+00:00)

Owner approved implementation for target v0.25.0.
