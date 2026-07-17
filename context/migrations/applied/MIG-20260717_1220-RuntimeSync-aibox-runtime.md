---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260717_1220-RuntimeSync-aibox-runtime
  created: 2026-07-17 12:20:45+00:00
  updated: '2026-07-17T12:32:08+00:00'
spec:
  source: aibox-runtime-home
  source_url: aibox://runtime-home
  from_version: 0.27.6
  to_version: 0.27.5
  state: rejected
  generated_by: aibox apply
  generated_at: 2026-07-17 12:20:45+00:00
  summary: 0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)
  affected_groups: []
  affected_files: []
  rejected_reason: 'Malformed no-op runtime sync: reports zero changed/conflicted/new/removed
    files and a backwards version transition from 0.27.6 to 0.27.5; no migration work
    can be applied.'
  rejected_at: '2026-07-17T12:32:08+00:00'
---

# Migration MIG-20260717_1220-RuntimeSync-aibox-runtime

Managed `.aibox-home/` runtime changes from `0.27.6` to `0.27.5`.

0 changed upstream, 0 conflicts, 0 new, 0 removed (0 groups affected)

## Counts

- unchanged: 43
- changed-locally-only: 0
- changed-upstream-only: 0
- conflict: 0
- new-upstream: 0
- removed-upstream: 0

- removed-upstream-stale: 0

## Per-intermediate review

Every released version between `from_version` and `to_version` whose              template snapshot is present on disk is listed below, with the number              of files that changed between consecutive snapshots. Useful for              catching scaffolding changes that were introduced and later reverted              across the span of a multi-version upgrade.

- `0.27.6` → `0.27.5`: 3 file(s) changed of 43

_No user-relevant changes._
