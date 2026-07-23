---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260723_0829-ValiantCliff-ship-v0285-port-cross-line-fixes
  created: '2026-07-23T08:29:51+00:00'
  updated: '2026-07-23T09:28:37+00:00'
spec:
  title: Ship v0.28.5 and port fixes across v0.x and v1.x
  state: done
  type: task
  priority: high
  description: Repair Hermes installation, integrate current v0.x changes, add enforceable
    bidirectional version-line port tracking, release v0.28.5, and port applicable
    changes to v1.x.
  started_at: '2026-07-23T08:30:01+00:00'
  completed_at: '2026-07-23T09:28:37+00:00'
---

## Transition note (2026-07-23T08:30:01+00:00)

Implementation started with three parallel read-only audits.


## Transition note (2026-07-23T09:28:27+00:00)

Implementation, cross-line promotion, validation, and v0.28.5 publication are complete and ready for closure.


## Transition note (2026-07-23T09:28:37+00:00)

Verified completion: v0.28.5 release is public with both Linux archives and checksums, documentation is deployed, v0.x and v1.x maintained branches carry the applicable fixes, and the v0 worktree is clean after closure promotion.
