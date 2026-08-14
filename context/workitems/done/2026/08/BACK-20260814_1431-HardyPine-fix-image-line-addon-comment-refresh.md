---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260814_1431-HardyPine-fix-image-line-addon-comment-refresh
  created: '2026-08-14T14:31:22+00:00'
  updated: '2026-08-14T14:38:50+00:00'
spec:
  title: Keep v0 latest image resolution and addon comment catalogs line-correct
  state: done
  type: bug
  priority: high
  description: Fix v0.x CLI image latest resolution so it cannot select v1 prerelease
    images. Make generated aibox.toml comment refresh detect addon catalog additions
    such as browser-testing, with regression tests for derived-project upgrades.
  started_at: '2026-08-14T14:31:30+00:00'
  completed_at: '2026-08-14T14:38:50+00:00'
---

## Transition note (2026-08-14T14:31:30+00:00)

Implementation and regression testing started after reproducing both causes.


## Transition note (2026-08-14T14:38:49+00:00)

Implementation complete and full verification evidence collected.


## Transition note (2026-08-14T14:38:50+00:00)

Accepted: major-line latest selection, catalog-change comment refresh, and regression coverage all pass.
