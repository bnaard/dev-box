---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_0425-SilentCrane-implement-amended-two-line-tmux-status
  created: '2026-05-08T04:25:49+00:00'
  labels:
    area: tmux
    component: status
    release: next-patch
    decision: DEC-20260508_0425-BoldTiger-adopt-amended-two-line-tmux-status
  updated: '2026-05-08T04:41:26+00:00'
spec:
  title: Implement amended two-line tmux status bar and diagnostics metrics
  state: review
  type: task
  priority: high
  description: 'Implement the accepted two-line tmux-powerkit status layout. Line
    1: AIBOX identity, tmux screens/windows, project/runtime context. Line 2: mode/prefix
    state and compact runtime health/agent state. Replace ambiguous symbols with short
    labels. DISK must show used/total. Extend sidecar/aibox-status collection for
    log severity counts, CPU usage deltas, disk used/total, process/AI/MCP state,
    migrations, memory pressure, and degraded state as needed.'
  parent: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  started_at: '2026-05-08T04:27:23+00:00'
---

## Transition note (2026-05-08T04:27:23+00:00)

Starting implementation for next patch release.


## Transition note (2026-05-08T04:41:26+00:00)

Ready for review: two-line tmux/powerkit status config and expanded aibox-status data path implemented; normal cargo test, clippy, shell syntax checks, and status helper tests passed.
