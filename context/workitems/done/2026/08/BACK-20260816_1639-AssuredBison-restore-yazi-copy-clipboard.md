---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260816_1639-AssuredBison-restore-yazi-copy-clipboard
  created: '2026-08-16T16:39:54+00:00'
  updated: '2026-08-16T16:53:39+00:00'
spec:
  title: Restore Yazi copy actions to tmux and host clipboard
  state: done
  type: bug
  priority: high
  description: Fix Yazi c-prefixed copy actions, including copying a selected file
    path, so copied values reach tmux buffers and the host clipboard; add regression
    coverage.
  started_at: '2026-08-16T16:39:58+00:00'
  completed_at: '2026-08-16T16:53:39+00:00'
---

## Transition note (2026-08-16T16:39:58+00:00)

Started Yazi clipboard action diagnosis and regression coverage.


## Transition note (2026-08-16T16:53:33+00:00)

Routed all Yazi c-copy variants through aibox-copy and added a real Yazi-to-tmux clipboard regression test; validation passed.


## Transition note (2026-08-16T16:53:39+00:00)

Accepted implementation evidence: c p/c d/c f/c n bridge to tmux/OSC52 and exact c p Yazi workflow passes end to end.
