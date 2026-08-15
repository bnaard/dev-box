---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260815_1553-PluckyFinch-embed-canonical-addon-catalog-in-cli
  created: '2026-08-15T15:53:43+00:00'
  updated: '2026-08-15T16:02:03+00:00'
spec:
  title: Embed canonical addon catalog in CLI
  state: done
  type: bug
  priority: high
  description: 'Make the shipped v0.x CLI self-contained for addon discovery: load
    every canonical addon definition embedded in the binary, preserve supported filesystem
    overrides, cover stale/incomplete installed catalogs (including supply-chain),
    and publish the fix in v0.32.5.'
  started_at: '2026-08-15T15:53:47+00:00'
  completed_at: '2026-08-15T16:02:03+00:00'
---

## Transition note (2026-08-15T15:53:47+00:00)

Implementing embedded canonical addon catalog and release coverage.


## Transition note (2026-08-15T16:01:59+00:00)

Embedded catalog implementation and stale-catalog regression verified; full suite contention failures passed sequential reruns; clippy, audit, and aarch64 release build pass.


## Transition note (2026-08-15T16:02:03+00:00)

Accepted for v0.32.5 integration.
