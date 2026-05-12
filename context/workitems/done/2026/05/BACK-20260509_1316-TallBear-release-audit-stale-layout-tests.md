---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_1316-TallBear-release-audit-stale-layout-tests
  created: '2026-05-09T13:16:56+00:00'
  updated: '2026-05-09T22:19:05+00:00'
spec:
  title: 'release-audit: add stale-test grep sweep for hardcoded layout/window-set assumptions'
  state: done
  type: task
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-09T22:18:28+00:00'
  completed_at: '2026-05-09T22:19:05+00:00'
---

## Transition note (2026-05-09T22:19:05+00:00)

Implemented and merged in commit 62889df + merge e427e62. Ships project-local prototype scripts/release-audit-stale-tests.py with 32-hit baseline plus propose-only upstream patch.
