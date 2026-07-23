---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260723_0911-RuntimeSync-aibox-runtime
  created: 2026-07-23 09:11:17+00:00
  updated: '2026-07-23T09:12:24+00:00'
spec:
  source: aibox-runtime-home
  source_url: aibox://runtime-home
  from_version: 0.28.3
  to_version: 0.28.4
  state: applied
  generated_by: aibox apply
  generated_at: 2026-07-23 09:11:17+00:00
  summary: 0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)
  affected_groups: []
  affected_files: []
  started_at: '2026-07-23T09:12:23+00:00'
  applied_at: '2026-07-23T09:12:24+00:00'
  progress_notes:
  - timestamp: '2026-07-23T09:12:24+00:00'
    actor: mcp
    note: Applied the branch-specific v0.28.4 runtime template baseline generated
      by the v1 line CLI while reconciling required skill selection.
---

# Migration MIG-20260723_0911-RuntimeSync-aibox-runtime

Managed `.aibox-home/` runtime changes from `0.28.3` to `0.28.4`.

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

- `0.28.3` → `0.28.4`: 43 file(s) changed of 44

_No user-relevant changes._
