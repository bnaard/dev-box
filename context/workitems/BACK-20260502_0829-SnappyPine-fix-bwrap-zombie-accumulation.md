---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260502_0829-SnappyPine-fix-bwrap-zombie-accumulation
  created: '2026-05-02T08:29:36+00:00'
  labels:
    area: runtime
    component: devcontainer
    provider: codex
  updated: '2026-05-02T08:35:30+00:00'
spec:
  title: Fix bwrap zombie accumulation in aibox devcontainers
  state: done
  type: bug
  priority: high
  description: 'Implement the accepted bwrap zombie solution: verify Codex bubblewrap
    sandbox prerequisites and add a proper init/reaper to generated devcontainer compose
    so orphaned sandbox helpers are reaped.'
  started_at: '2026-05-02T08:29:39+00:00'
  completed_at: '2026-05-02T08:35:30+00:00'
---

## Transition note (2026-05-02T08:29:39+00:00)

Starting implementation of accepted plan with max two parallel agents: compose init/reaper plus Codex bubblewrap verification guardrails.


## Transition note (2026-05-02T08:35:27+00:00)

Implemented compose init reaper, generated compose refresh, Codex sandbox doctor guardrails, docs, and tests. Validation: full cargo test, clippy, diff check, apply --no-container.


## Transition note (2026-05-02T08:35:30+00:00)

Completed. Existing zombie bwrap processes still require container recreation because zombies cannot be killed directly once reparented to the old sleep PID 1.
