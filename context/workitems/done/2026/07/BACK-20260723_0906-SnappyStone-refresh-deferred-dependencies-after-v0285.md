---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260723_0906-SnappyStone-refresh-deferred-dependencies-after-v0285
  created: '2026-07-23T09:06:26+00:00'
  labels:
    source: v0.28.5-release-state
    version_lines:
    - v0.x
    - v1.x
  updated: '2026-08-21T20:10:38+00:00'
spec:
  title: Refresh deferred dependencies and tool pins after v0.28.5
  state: cancelled
  type: task
  priority: medium
  description: Review and update the dependency and tool-version drift recorded in
    dist/RELEASE-STATE.md for v0.28.5 as a dedicated maintenance pass. Cover Rust
    lockfile updates, base image utilities, uv, harness/addon pins, documentation
    toolchains, Go, infrastructure tools, and Kubernetes tools. Inspect upstream release
    notes, apply compatible updates to both maintained version lines, rebuild affected
    images, and rerun the appropriate release and runtime validation gates.
  completed_at: '2026-08-21T20:10:38+00:00'
---

## Transition note (2026-08-21T20:10:38+00:00)

Superseded by consolidated v0.x dependency refresh BACK-20260821_1557-AgileEmber and archived during its implementation.
