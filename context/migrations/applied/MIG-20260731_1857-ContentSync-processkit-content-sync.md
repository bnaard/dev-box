---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260731_1857-ContentSync-processkit-content-sync
  created: 2026-07-31 18:57:01+00:00
  updated: '2026-08-01T07:44:01+00:00'
spec:
  source: processkit
  source_url: https://github.com/projectious-work/processkit.git
  from_version: v0.28.4
  to_version: v0.28.5
  state: applied
  generated_by: aibox apply
  generated_at: 2026-07-31 18:57:01+00:00
  summary: 0 changed upstream, 1 conflicts, 0 new, 0 removed, 0 stale-removed (1 groups
    affected)
  affected_groups:
  - AGENTS
  affected_files:
  - path: AGENTS.md
    classification: conflict
  started_at: '2026-08-01T07:44:01+00:00'
  applied_at: '2026-08-01T07:44:01+00:00'
  progress_notes:
  - timestamp: '2026-08-01T07:44:01+00:00'
    actor: mcp
    note: Resolved AGENTS.md conflict by retaining the aibox project-local AGENTS.md.
      The upstream v0.28.5 change only updates processkit repository build commands;
      adopting it would overwrite aibox-specific Rust, E2E, release, and project policy
      instructions. Compared v0.28.4, v0.28.5, and live AGENTS.md before applying.
---

# Migration MIG-20260731_1857-ContentSync-processkit-content-sync

From `v0.28.4` to `v0.28.5` (source: `https://github.com/projectious-work/processkit.git`).

0 changed upstream, 1 conflicts, 0 new, 0 removed, 0 stale-removed (1 groups affected)

## Counts

- unchanged: 721
- changed-locally-only: 0
- changed-upstream-only: 0
- conflict: 1
- new-upstream: 0
- removed-upstream: 0
- removed-upstream-stale: 0

## Changes by group

### AGENTS

**conflict**

- `AGENTS.md` → `AGENTS.md`
