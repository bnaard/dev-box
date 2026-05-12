---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_1331-WildAsh-yazi-brief-terminal-response-timeout-error
  created: '2026-05-09T13:31:09+00:00'
  labels:
    version: v0.25.7
    area: yazi
    surface: terminal-stack
  updated: '2026-05-09T22:19:18+00:00'
spec:
  title: 'yazi: brief ''Terminal response timeout'' error flashes in panel on launch'
  state: done
  type: bug
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-09T22:18:31+00:00'
  completed_at: '2026-05-09T22:19:18+00:00'
---

## Transition note (2026-05-09T22:19:18+00:00)

Diagnosed and fixed in commit 6abd17c + merge e427e62. Terminal-emulator-agnostic env passthrough across docker-compose + tmux update-environment. Host-flash verification post `aibox up` still pending.
