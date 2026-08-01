---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260731_1724-RuntimeSync-aibox-runtime
  created: 2026-07-31 17:24:20+00:00
  updated: '2026-08-01T07:47:48+00:00'
spec:
  source: aibox-runtime-home
  source_url: aibox://runtime-home
  from_version: 1.0.0-alpha.1
  to_version: 0.28.18
  state: rejected
  generated_by: aibox apply
  generated_at: 2026-07-31 17:24:20+00:00
  summary: 0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)
  affected_groups: []
  affected_files: []
  rejected_reason: Malformed zero-change runtime migration caused by applying v0.28.18
    host context to the independent v1.0.0-alpha.1 line. It affects no files or groups
    and must not record a cross-line runtime downgrade.
  rejected_at: '2026-08-01T07:47:48+00:00'
---

# Migration MIG-20260731_1724-RuntimeSync-aibox-runtime

Managed `.aibox-home/` runtime changes from `1.0.0-alpha.1` to `0.28.18`.

0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)

## Counts

- unchanged: 44
- changed-locally-only: 0
- changed-upstream-only: 0
- conflict: 0
- new-upstream: 0
- removed-upstream: 0

- removed-upstream-stale: 0

## Per-intermediate review

Every released version between `from_version` and `to_version` whose              template snapshot is present on disk is listed below, with the number              of files that changed between consecutive snapshots. Useful for              catching scaffolding changes that were introduced and later reverted              across the span of a multi-version upgrade.

- `1.0.0-alpha.1` → `0.28.18`: 43 file(s) changed of 44

_No user-relevant changes._
