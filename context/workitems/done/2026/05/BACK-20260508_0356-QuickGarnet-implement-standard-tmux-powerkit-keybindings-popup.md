---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_0356-QuickGarnet-implement-standard-tmux-powerkit-keybindings-popup
  created: '2026-05-08T03:56:24+00:00'
  labels:
    area: tmux
    component: runtime
    release: next-patch
  updated: '2026-05-08T10:47:32+00:00'
spec:
  title: Implement standard tmux-powerkit keybindings popup defaults in aibox runtime
  state: done
  type: task
  priority: high
  description: 'Ensure generated tmux runtime config explicitly follows tmux-powerkit standard key-help pattern: on-demand keybindings popup as the canonical mechanism, with documented key/size overrides and release checks to avoid regressions.'
  parent: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  started_at: '2026-05-08T04:41:20+00:00'
  completed_at: '2026-05-08T10:47:32+00:00'
---

## Transition note (2026-05-08T04:41:20+00:00)

Implementation was folded into the tmux runtime patch set.


## Transition note (2026-05-08T04:41:23+00:00)

Ready for review: tmux runtime patch set includes standard keybinding popup direction plus passing normal cargo test, clippy, shell syntax checks, and status helper tests.


## Transition note (2026-05-08T10:47:32+00:00)

Resolved after review: generated tmux config binds the standard key-help popup to prefix + ? via display-popup/list-keys; regression assertion covers it.
