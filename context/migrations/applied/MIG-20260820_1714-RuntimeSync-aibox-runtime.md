---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260820_1714-RuntimeSync-aibox-runtime
  created: 2026-08-20 17:14:43+00:00
  updated: '2026-08-21T09:34:58+00:00'
spec:
  source: aibox-runtime-home
  source_url: aibox://runtime-home
  from_version: 0.34.0
  to_version: 0.34.1
  state: applied
  generated_by: aibox apply
  generated_at: 2026-08-20 17:14:43+00:00
  summary: 0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)
  affected_groups: []
  affected_files: []
  started_at: '2026-08-21T09:34:58+00:00'
  applied_at: '2026-08-21T09:34:58+00:00'
  progress_notes:
  - timestamp: '2026-08-21T09:34:58+00:00'
    actor: mcp
    note: Applied during v0.34.2 release reconciliation; runtime sync reports no changed,
      conflicting, new, or removed files.
---

# Migration MIG-20260820_1714-RuntimeSync-aibox-runtime

Managed `.aibox-home/` runtime changes from `0.34.0` to `0.34.1`.

0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)

## Counts

- unchanged: 48
- changed-locally-only: 0
- changed-upstream-only: 0
- conflict: 0
- new-upstream: 0
- removed-upstream: 0

- removed-upstream-stale: 0

## Per-intermediate review

Every released version between `from_version` and `to_version` whose              template snapshot is present on disk is listed below, with the number              of files that changed between consecutive snapshots. Useful for              catching scaffolding changes that were introduced and later reverted              across the span of a multi-version upgrade.

- `0.34.0` → `0.34.1`: 46 file(s) changed of 48

_No user-relevant changes._
