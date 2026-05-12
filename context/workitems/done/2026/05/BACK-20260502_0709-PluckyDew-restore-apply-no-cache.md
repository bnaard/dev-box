---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260502_0709-PluckyDew-restore-apply-no-cache
  created: '2026-05-02T07:09:10+00:00'
  updated: '2026-05-02T07:25:46+00:00'
spec:
  title: Restore aibox apply --no-cache and keep rebuild alias
  state: done
  type: task
  priority: high
  description: Reintroduce aibox apply --no-cache as the public no-cache build flag while keeping --rebuild as a compatibility alias. Cover CLI parsing/help and build command behavior.
  started_at: '2026-05-02T07:09:23+00:00'
  completed_at: '2026-05-02T07:25:46+00:00'
---

## Transition note (2026-05-02T07:09:23+00:00)

Implementation started in this session.


## Transition note (2026-05-02T07:25:35+00:00)

Implementation and focused tests complete; ready to close after verification.


## Transition note (2026-05-02T07:25:46+00:00)

Verified with focused CLI tests, cargo check, and clippy.
