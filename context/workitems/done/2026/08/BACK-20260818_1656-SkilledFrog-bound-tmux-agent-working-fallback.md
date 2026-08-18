---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260818_1656-SkilledFrog-bound-tmux-agent-working-fallback
  created: '2026-08-18T16:56:04+00:00'
  updated: '2026-08-18T17:00:56+00:00'
spec:
  title: Bound tmux agent working fallback when lifecycle hooks are unavailable
  state: done
  type: bug
  priority: high
  description: Fix long-lived interactive harness panes remaining permanently in the
    working title state when native lifecycle hooks do not fire. Change the source/generator
    behavior, add isolated real-tmux regression coverage, reconcile the managed runtime,
    and verify title transitions locally.
  started_at: '2026-08-18T16:56:09+00:00'
  completed_at: '2026-08-18T17:00:56+00:00'
---

## Transition note (2026-08-18T16:56:09+00:00)

Started source-level implementation and isolated tmux validation.


## Transition note (2026-08-18T17:00:56+00:00)

Implemented idle launcher fallback; focused generator tests and isolated tmux runtime test pass.


## Transition note (2026-08-18T17:00:56+00:00)

Validated locally: long-running fake agent leaves tmux attention state idle; native hooks remain responsible for working/question/done.
