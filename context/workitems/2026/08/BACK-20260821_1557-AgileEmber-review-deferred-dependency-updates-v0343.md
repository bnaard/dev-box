---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260821_1557-AgileEmber-review-deferred-dependency-updates-v0343
  created: '2026-08-21T15:57:55+00:00'
spec:
  title: Review and validate deferred dependency updates after v0.34.3
  state: backlog
  type: chore
  priority: medium
  description: Phase 0 for v0.34.3 found routine non-security updates for Zensical
    0.0.56, Go 1.27.0, Rust 1.98.0, Bun 1.4.0, PDM 2.28.2, OpenTofu 1.12.6, kubectl
    1.36.4, Tau 0.3.13, and cc 1.4.4. Review upstream release notes, apply compatible
    pins and lockfile updates in a dedicated change, rebuild affected images, and
    run full addon, companion, visual, audit, and cross-line validation before release.
  scope: v0.x
---
