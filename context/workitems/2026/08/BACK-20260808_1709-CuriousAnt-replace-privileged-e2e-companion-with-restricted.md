---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260808_1709-CuriousAnt-replace-privileged-e2e-companion-with-restricted
  created: '2026-08-08T17:09:46+00:00'
  labels:
    github_issue: '372'
    release_line: v0.x
  updated: '2026-08-08T17:09:54+00:00'
spec:
  title: Replace privileged E2E companion with restricted host release testing
  state: in-progress
  type: task
  priority: critical
  description: 'Implement GitHub issue #372 in staged slices. First slice: define
    E2E execution classification and migrate companion-independent lifecycle contracts
    to a local temporary-workspace harness without weakening asserted behavior.'
  started_at: '2026-08-08T17:09:54+00:00'
---

## Transition note (2026-08-08T17:09:54+00:00)

Beginning first implementation slice: local E2E execution classification and migration of companion-independent lifecycle contracts.
