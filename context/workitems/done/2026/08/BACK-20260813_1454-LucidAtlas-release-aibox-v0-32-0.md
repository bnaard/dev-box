---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260813_1454-LucidAtlas-release-aibox-v0-32-0
  created: '2026-08-13T14:54:44+00:00'
  updated: '2026-08-13T16:21:55+00:00'
spec:
  title: Release aibox v0.32.0
  state: done
  type: task
  priority: high
  description: Prepare, validate, publish, and verify the v0.32.0 minor release from
    v0.x-release. Complete repository-side Phase 1 and provide the owner with the
    exact macOS release-host Phase 2 command.
  started_at: '2026-08-13T14:54:48+00:00'
  completed_at: '2026-08-13T16:21:55+00:00'
---

## Transition note (2026-08-13T14:54:48+00:00)

Owner requested the next v0.x minor release; target resolved to v0.32.0 and release preflight started.


## Transition note (2026-08-13T16:21:50+00:00)

v0.32.0 Phase 1 published, but host completion exposed a full-Chromium/headless-shell probe mismatch. The immutable correction was released as v0.32.1.


## Transition note (2026-08-13T16:21:55+00:00)

Superseded for host completion by v0.32.1; v0.32.0 remains published and immutable, while the corrected patch release is the supported completion target.
