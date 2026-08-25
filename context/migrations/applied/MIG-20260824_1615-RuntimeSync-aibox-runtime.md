---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260824_1615-RuntimeSync-aibox-runtime
  created: 2026-08-24 16:15:30+00:00
  updated: '2026-08-25T09:38:19+00:00'
spec:
  source: aibox-runtime-home
  source_url: aibox://runtime-home
  from_version: 0.34.4
  to_version: 0.34.7
  state: applied
  generated_by: aibox apply
  generated_at: 2026-08-24 16:15:30+00:00
  summary: 0 changed upstream, 0 conflicts, 2 new, 0 removed (2 groups affected)
  affected_groups:
  - runtime-misc
  - runtime-yazi
  affected_files:
  - path: .config/yazi/plugins/preview-options.yazi/main.lua
    classification: new-upstream
  - path: .local/bin/aibox-size-tree
    classification: new-upstream
  started_at: '2026-08-25T09:38:19+00:00'
  applied_at: '2026-08-25T09:38:19+00:00'
  progress_notes:
  - timestamp: '2026-08-25T09:38:19+00:00'
    actor: mcp
    note: Applied during explicit full migration and pk-doctor reconciliation; migration
      contains two new upstream runtime files and no conflicts, removals, or local-only
      changes.
---

# Migration MIG-20260824_1615-RuntimeSync-aibox-runtime

Managed `.aibox-home/` runtime changes from `0.34.4` to `0.34.7`.

0 changed upstream, 0 conflicts, 2 new, 0 removed (2 groups affected)

## Counts

- unchanged: 49
- changed-locally-only: 0
- changed-upstream-only: 0
- conflict: 0
- new-upstream: 2
- removed-upstream: 0

- removed-upstream-stale: 0

## Changes by group

### runtime-misc

**new-upstream**

- `.aibox-home/.local/bin/aibox-size-tree` -> `.aibox-home/.local/bin/aibox-size-tree`

### runtime-yazi

**new-upstream**

- `.aibox-home/.config/yazi/plugins/preview-options.yazi/main.lua` -> `.aibox-home/.config/yazi/plugins/preview-options.yazi/main.lua`
