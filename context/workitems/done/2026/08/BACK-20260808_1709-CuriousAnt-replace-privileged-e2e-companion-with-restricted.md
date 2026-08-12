---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260808_1709-CuriousAnt-replace-privileged-e2e-companion-with-restricted
  created: '2026-08-08T17:09:46+00:00'
  labels:
    github_issue: '372'
    release_line: v0.x
  updated: '2026-08-12T16:28:31+00:00'
spec:
  title: Replace privileged E2E companion with restricted host release testing
  state: done
  type: task
  priority: critical
  description: 'Implement GitHub issue #372 in staged slices. First slice: define
    E2E execution classification and migrate companion-independent lifecycle contracts
    to a local temporary-workspace harness without weakening asserted behavior.'
  started_at: '2026-08-08T17:09:54+00:00'
  completed_at: '2026-08-12T16:28:31+00:00'
---

## Transition note (2026-08-08T17:09:54+00:00)

Beginning first implementation slice: local E2E execution classification and migration of companion-independent lifecycle contracts.


## Transition note (2026-08-12T16:28:31+00:00)

PR #373 merged; v0.31.2 and v0.31.3 completed the restricted host validation and publication cycle. Serialized local E2E, Textual headless tests, doctor, release audit, Darwin/Linux assets, GHCR pushes, and manifest-constrained publication verified.


## Transition note (2026-08-12T16:28:31+00:00)

Owner-run v0.31.3 host gate and retry-safe publisher completed. GitHub release contains Linux and Darwin assets with checksums; foundation, versioned runtime, and runtime-latest images were pushed and inspected. Issue #372 acceptance is complete.
