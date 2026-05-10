---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260502_0709-SteadyMoss-runtime-home-layout-provenance
  created: '2026-05-02T07:09:10+00:00'
  updated: '2026-05-02T07:25:40+00:00'
spec:
  title: Fix runtime-home layout drift with provenance-safe updates
  state: done
  type: task
  priority: high
  description: Add provenance-safe managed runtime file updates so aibox apply updates
    stale generated .aibox-home zellij layouts when AI harness selection changes,
    while preserving genuine user edits. Include regression tests for claude+codex
    to codex-only layout changes.
  started_at: '2026-05-02T07:09:23+00:00'
  completed_at: '2026-05-02T07:25:40+00:00'
---

## Transition note (2026-05-02T07:09:23+00:00)

Implementation started in this session.


## Transition note (2026-05-02T07:25:35+00:00)

Implementation and focused tests complete; ready to close after verification.


## Transition note (2026-05-02T07:25:40+00:00)

Verified with focused tests, cargo check, clippy, aibox apply, and live runtime layout inspection.
