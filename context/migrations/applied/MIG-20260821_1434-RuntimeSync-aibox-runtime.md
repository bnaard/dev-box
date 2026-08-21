---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260821_1434-RuntimeSync-aibox-runtime
  created: 2026-08-21 14:34:04+00:00
  updated: '2026-08-21T14:59:40+00:00'
spec:
  source: aibox-runtime-home
  source_url: aibox://runtime-home
  from_version: 0.34.1
  to_version: 0.34.2
  state: applied
  generated_by: aibox apply
  generated_at: 2026-08-21 14:34:04+00:00
  summary: 0 changed upstream, 0 conflicts, 1 new, 0 removed (1 groups affected)
  affected_groups:
  - runtime-misc
  affected_files:
  - path: .local/bin/aibox-codex-notify
    classification: new-upstream
  started_at: '2026-08-21T14:59:40+00:00'
  applied_at: '2026-08-21T14:59:40+00:00'
  progress_notes:
  - timestamp: '2026-08-21T14:59:40+00:00'
    actor: mcp
    note: Applied during user-authorized full reconciliation; one conflict-free new
      managed runtime helper from v0.34.2.
---

# Migration MIG-20260821_1434-RuntimeSync-aibox-runtime

Managed `.aibox-home/` runtime changes from `0.34.1` to `0.34.2`.

0 changed upstream, 0 conflicts, 1 new, 0 removed (1 groups affected)

## Counts

- unchanged: 48
- changed-locally-only: 0
- changed-upstream-only: 0
- conflict: 0
- new-upstream: 1
- removed-upstream: 0

- removed-upstream-stale: 0

## Changes by group

### runtime-misc

**new-upstream**

- `.aibox-home/.local/bin/aibox-codex-notify` -> `.aibox-home/.local/bin/aibox-codex-notify`
