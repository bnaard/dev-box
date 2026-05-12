---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_1316-SnappyWolf-tmux-multi-harness-layouts
  created: '2026-05-09T13:16:16+00:00'
  updated: '2026-05-09T22:19:13+00:00'
spec:
  title: 'tmux layouts: support multiple enabled harnesses with primary/secondary slot model'
  state: done
  type: task
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-09T22:18:30+00:00'
  completed_at: '2026-05-09T22:19:13+00:00'
---

## Transition note (2026-05-09T22:19:13+00:00)

4c+4b scope shipped. Commit 917f160 + merge e427e62. AIBOX_LAYOUT_AGENT_SPLIT/RATIO env knobs + drop-in support; 16/16 cargo tests pass incl. 3 new.
