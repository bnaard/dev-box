---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_1316-SilentFjord-tmux-statusline-l1-left-oom-double
  created: '2026-05-09T13:16:40+00:00'
  updated: '2026-05-09T22:19:16+00:00'
spec:
  title: 'tmux status-line: add window list to line1-left, fix OOM/LOG/PROC/AI/MCP/MIG label doubling'
  state: done
  type: bug
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-09T22:18:30+00:00'
  completed_at: '2026-05-09T22:19:16+00:00'
---

## Transition note (2026-05-09T22:19:16+00:00)

Implemented and merged in commit e6e1e9a + merge e427e62. Six powerkit plugins de-doubled; status-left extended with window list. Paired DEC-20260509_2125-CoolFrog records contract. Visual verification post container restart still pending.
