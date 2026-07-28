---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260728_0705-RefinedHarvest-fix-addon-installer-download-failures
  created: '2026-07-28T07:05:15+00:00'
  updated: '2026-07-28T07:05:24+00:00'
spec:
  title: Fix recurring add-on installer download failures and release v0.28.17
  state: in-progress
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: Diagnose the Go add-on checksum failure reported from a v0.28.16 derived
    project, audit all download-based add-on installers for the recurring failure
    pattern, implement a durable source/generator-level prevention with regression
    coverage, regenerate tracked runtime output, validate the v0.x line, and prepare
    the v0.28.17 patch release.
  started_at: '2026-07-28T07:05:24+00:00'
---

## Transition note (2026-07-28T07:05:24+00:00)

Starting v0.28.17 diagnosis, add-on-wide audit, durable fix, regression coverage, and release preparation.
