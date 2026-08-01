---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260801_0911-RuntimeSync-aibox-runtime
  created: 2026-08-01 09:11:50+00:00
  updated: '2026-08-01T09:22:02+00:00'
spec:
  source: aibox-runtime-home
  source_url: aibox://runtime-home
  from_version: 0.28.18
  to_version: 0.28.19
  state: rejected
  generated_by: aibox apply
  generated_at: 2026-08-01 09:11:50+00:00
  summary: 0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)
  affected_groups: []
  affected_files: []
  rejected_reason: 'No-op runtime sync: 0 upstream changes, 0 conflicts, 0 new files,
    and 0 removed files.'
  rejected_at: '2026-08-01T09:22:02+00:00'
---

# Migration MIG-20260801_0911-RuntimeSync-aibox-runtime

Managed `.aibox-home/` runtime changes from `0.28.18` to `0.28.19`.

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

- `0.28.18` → `0.28.19`: 43 file(s) changed of 44

_No user-relevant changes._
