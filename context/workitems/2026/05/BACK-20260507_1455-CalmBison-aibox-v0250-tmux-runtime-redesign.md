---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  created: '2026-05-07T14:55:49+00:00'
  updated: '2026-05-08T10:47:49+00:00'
spec:
  title: Implement aibox v0.25.0 tmux runtime redesign
  state: review
  type: epic
  priority: high
  description: 'Replace Zellij completely with tmux for aibox v0.25.0. Remove Zellij binaries, KDL layouts/themes, WASM plugin status rows, permission caches, terminal profiles, docs, and tests. Introduce pinned preinstalled tmux plugins and tmux-native generated runtime config while preserving the sidecar container and visual testing paradigm. Related decision: DEC-20260507_1447-VastLeaf-remove-zellij-and-rebuild-aibox-runtime.'
  scope: runtime-architecture
  started_at: '2026-05-07T14:56:15+00:00'
---

## Transition note (2026-05-07T14:56:15+00:00)

Owner approved implementation for target v0.25.0.


## Transition note (2026-05-08T10:47:49+00:00)

Resolved from active implementation into review: core tmux runtime migration, config generation, runtime seed, status, docs/scripts/smoke, keybinding popup, provider endpoint variables, Yazi/Vim handoff fixes, and host-smoke guardrails are implemented. Remaining follow-ups are explicit separate backlog/review items: live visual companion execution after companion rebuild, persistence enablement policy, and deferred dependency update passes.
