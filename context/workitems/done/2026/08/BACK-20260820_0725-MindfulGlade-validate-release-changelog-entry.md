---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260820_0725-MindfulGlade-validate-release-changelog-entry
  created: '2026-08-20T07:25:24+00:00'
  updated: '2026-08-21T20:36:05+00:00'
spec:
  title: Validate public changelog entry during release
  state: done
  type: task
  priority: medium
  description: Add a release/docs validation gate that requires docs-site/content/changelog
    to contain the exact release version and verifies it renders as the newest Change
    Log entry. This prevents a fully published release from missing its public documentation
    timeline entry.
  started_at: '2026-08-21T19:59:21+00:00'
  completed_at: '2026-08-21T20:36:05+00:00'
---

## Transition note (2026-08-21T19:59:21+00:00)

Implementation started by owner direction on 2026-08-21.


## Transition note (2026-08-21T20:36:04+00:00)

Implementation and regression verification complete; owner-requested local validation passed.


## Transition note (2026-08-21T20:36:05+00:00)

Closed after full Rust 1.98 tests, mandatory visual matrix, all-theme cast sweep, release contract tests, and documentation validation passed.
