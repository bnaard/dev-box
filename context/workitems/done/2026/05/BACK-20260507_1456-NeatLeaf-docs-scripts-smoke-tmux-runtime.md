---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1456-NeatLeaf-docs-scripts-smoke-tmux-runtime
  created: '2026-05-07T14:56:07+00:00'
  updated: '2026-05-08T10:47:39+00:00'
spec:
  title: Update docs, scripts, release smoke, and diagnostics for tmux runtime
  state: done
  type: task
  priority: high
  description: 'Replace Zellij references in docs, scripts, release smoke, stale-runtime
    diagnostics, cheatsheet, terminal profile labels, screencast tooling, and release-check
    surfaces. Keep host/sidecar runtime evidence workflows but make them tmux-native.
    Related decision: DEC-20260507_1447-VastLeaf-remove-zellij-and-rebuild-aibox-runtime.'
  parent: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  scope: docs-release-ops
  started_at: '2026-05-07T15:36:00+00:00'
  completed_at: '2026-05-08T10:47:39+00:00'
---

## Transition note (2026-05-07T15:36:00+00:00)

Docs, scripts, release smoke, and diagnostics were updated for tmux; transitioning through in-progress before review.


## Transition note (2026-05-07T15:36:06+00:00)

Docs/scripts/release smoke were updated for tmux and shell syntax checks passed.


## Transition note (2026-05-08T10:47:39+00:00)

Resolved after review: docs, maintenance scripts, release runtime smoke, SSH-first companion guidance, host-smoke addon default, and persistence guardrails are updated; shell syntax checks pass.
