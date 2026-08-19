---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260819_1340-CordialShell-initialize-codex-title-session-start
  created: '2026-08-19T13:40:27+00:00'
  updated: '2026-08-19T14:04:41+00:00'
spec:
  title: Initialize Codex tmux title on SessionStart
  state: done
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: Codex startup rendered ' - @codex' because the generated SessionStart
    hook emitted only the compliance contract. Add the idle attention signal at SessionStart,
    reconcile the live runtime, and verify a fresh Codex process shows repository
    and model before the first prompt.
  started_at: '2026-08-19T13:40:34+00:00'
  completed_at: '2026-08-19T14:04:41+00:00'
---

## Transition note (2026-08-19T13:40:34+00:00)

Added Codex SessionStart idle signal, regenerated live runtime, and passed focused hook/real-tmux tests plus clippy/fmt/diff checks. Awaiting fresh Codex process startup verification before review.


## Transition note (2026-08-19T14:04:32+00:00)

Codex hook payload propagation implemented; live managed runtime reconciled with normal aibox apply; isolated fresh tmux/Codex startup verified as idle|aibox|gpt-5.6-sol|codex before first prompt; focused, real-tmux, full Rust, serial timeout reruns, clippy, fmt, and diff checks passed.


## Transition note (2026-08-19T14:04:41+00:00)

Review complete: verified the generated launcher initializes repository and model before Codex hook trust or the first prompt, and generated Codex lifecycle hooks preserve JSON payloads for model/effort extraction.
