---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1456-AmberBrook-cli-config-generation-tmux-migration
  created: '2026-05-07T14:56:07+00:00'
  updated: '2026-05-08T10:47:36+00:00'
spec:
  title: Refactor CLI config, generation, sync, and migration from Zellij to tmux
  state: done
  type: task
  priority: high
  description: 'Replace customization.zellij_status and Zellij layout generation with
    tmux-oriented configuration for v0.25.0. Preserve aibox.toml as declarative source
    of truth, generate tmux config/session templates, and provide migration warnings
    or migration docs for old Zellij fields. Related decision: DEC-20260507_1447-VastLeaf-remove-zellij-and-rebuild-aibox-runtime.'
  parent: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  scope: cli-runtime-generation
  started_at: '2026-05-07T15:35:59+00:00'
  completed_at: '2026-05-08T10:47:36+00:00'
---

## Transition note (2026-05-07T15:35:59+00:00)

Implementation completed in v0.25.0 tmux migration branch; transitioning through in-progress before review.


## Transition note (2026-05-07T15:36:06+00:00)

CLI config/generation/runtime sync tmux migration implemented and validated with cargo test, clippy, and e2e compile gate.


## Transition note (2026-05-08T10:47:36+00:00)

Resolved after review: tmux config generation, legacy status alias migration, named-window layout targeting, and runtime config comments are implemented with focused Rust/E2E coverage.
