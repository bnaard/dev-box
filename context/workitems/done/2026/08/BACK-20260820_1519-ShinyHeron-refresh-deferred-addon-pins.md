---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260820_1519-ShinyHeron-refresh-deferred-addon-pins
  created: '2026-08-20T15:19:28+00:00'
  updated: '2026-08-21T20:10:36+00:00'
spec:
  title: Refresh deferred addon tool pins after v0.34.1
  state: cancelled
  type: chore
  priority: medium
  description: 'Review and update non-security dependency drift reported by the v0.34.1
    release-state preflight: Zensical 0.0.56, Go 1.27.0, Bun 1.4.0, PDM 2.28.2, OpenTofu
    1.12.6, kubectl 1.36.4, and Tau 0.3.12. Inspect upstream release notes, update
    curated pins and checksums, rebuild affected images, and run the relevant addon/runtime
    E2E gates. This is intentionally deferred from the theme-focused v0.34.1 patch
    release.'
  completed_at: '2026-08-21T20:10:36+00:00'
---

## Transition note (2026-08-21T20:10:36+00:00)

Superseded by consolidated v0.x dependency refresh BACK-20260821_1557-AgileEmber and archived during its implementation.
