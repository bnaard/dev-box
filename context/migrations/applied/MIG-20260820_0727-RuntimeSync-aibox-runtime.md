---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260820_0727-RuntimeSync-aibox-runtime
  created: 2026-08-20 07:27:54+00:00
  updated: '2026-08-20T07:39:24+00:00'
spec:
  source: aibox-runtime-home
  source_url: aibox://runtime-home
  from_version: 0.33.2
  to_version: 0.34.0
  state: applied
  generated_by: aibox apply
  generated_at: 2026-08-20 07:27:54+00:00
  summary: 0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)
  affected_groups: []
  affected_files: []
  started_at: '2026-08-20T07:39:24+00:00'
  applied_at: '2026-08-20T07:39:24+00:00'
  progress_notes:
  - timestamp: '2026-08-20T07:39:24+00:00'
    actor: mcp
    note: 'Reconciled as an unambiguous runtime version transition: all 45 managed
      files are unchanged locally/upstream and there are no conflicts or affected
      files.'
---

# Migration MIG-20260820_0727-RuntimeSync-aibox-runtime

Managed `.aibox-home/` runtime changes from `0.33.2` to `0.34.0`.

0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)

## Counts

- unchanged: 45
- changed-locally-only: 0
- changed-upstream-only: 0
- conflict: 0
- new-upstream: 0
- removed-upstream: 0

- removed-upstream-stale: 0

## Per-intermediate review

Every released version between `from_version` and `to_version` whose              template snapshot is present on disk is listed below, with the number              of files that changed between consecutive snapshots. Useful for              catching scaffolding changes that were introduced and later reverted              across the span of a multi-version upgrade.

- `0.33.2` → `0.34.0`: 44 file(s) changed of 45

_No user-relevant changes._
