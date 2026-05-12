---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1517-TrueBrook-zellij-scorched-earth-complete-excision
  created: '2026-05-08T15:17:19+00:00'
  labels:
    track: zellij-excise
    release: v0.25.6
  updated: '2026-05-08T20:44:43+00:00'
spec:
  title: 'v0.25.6: Zellij scorched-earth excision'
  state: done
  type: task
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-08T19:56:11+00:00'
  completed_at: '2026-05-08T20:44:43+00:00'
---

## Transition note (2026-05-08T19:56:11+00:00)

Implementation in progress this session via subagent.


## Transition note (2026-05-08T20:44:34+00:00)

Implementation complete. Tests: 825 unit + 73 e2e + 28 integration = 926 green. Acceptance grep: only compat.rs (historical notes) and seed.rs (cleanup_legacy_zellij_files function name) remain.


## Transition note (2026-05-08T20:44:43+00:00)

Accepted as done.
