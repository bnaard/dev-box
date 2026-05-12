---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1456-SoftGarnet-pinned-tmux-image-runtime-seed
  created: '2026-05-07T14:56:06+00:00'
  updated: '2026-05-08T10:47:44+00:00'
spec:
  title: Replace image and generated runtime seed with pinned tmux stack
  state: done
  type: task
  priority: high
  description: 'Remove Zellij image stages, KDL config, WASM plugin build, and permission cache seed. Add tmux, clipboard dependencies, and preinstalled pinned plugins: tmux-powerkit, tmux-sensible, vim-tmux-navigator, tpm, tmux-yank, with tmux-resurrect and tmux-continuum installed but disabled by default until persistence policy is finalized. Related decision: DEC-20260507_1447-VastLeaf-remove-zellij-and-rebuild-aibox-runtime.'
  parent: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  scope: runtime-image
  started_at: '2026-05-07T14:56:15+00:00'
  completed_at: '2026-05-08T10:47:44+00:00'
---

## Transition note (2026-05-07T14:56:15+00:00)

Starting first implementation slice: image and generated runtime seed.


## Transition note (2026-05-07T15:36:06+00:00)

Pinned tmux image/runtime seed implemented; Zellij image/config/plugin sources removed; managed tmux plugins preinstalled and persistence plugins disabled by default.


## Transition note (2026-05-08T10:47:44+00:00)

Resolved after review: runtime seed and base fallback tmux config now use pinned tmux plugin paths, no Zellij assumptions, safe status defaults, and disabled-by-default persistence settings.
