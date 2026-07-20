---
apiVersion: processkit.projectious.work/v2
kind: Migration
metadata:
  id: MIG-20260720_1408-SnappyComet-canonicalize-note-spec-key-order-for
  created: '2026-07-20T14:08:59+00:00'
  updated: '2026-07-20T14:09:21+00:00'
spec:
  source: local-project
  kind: data-fix
  state: applied
  generated_by: migration-management.draft_migration
  generated_at: '2026-07-20T14:08:59+00:00'
  summary: Canonicalize Note spec key order for v0.27.5 doctor
  affected_files:
  - path: context/notes/2026/06/NOTE-20260609_1604-ZestfulQuail-aibox-multitarget-devcontainer-snapshot.md
    classification: changed-locally-only
  affected_groups: []
  plan: ''
  progress_notes:
  - timestamp: '2026-07-20T14:09:21+00:00'
    actor: mcp
    note: Canonicalized spec key order using the processkit entity serializer; content
      and metadata preserved. Focused schema_filename doctor passes with zero errors.
      Upstream parser regression tracked in processkit#88.
  source_api_version: processkit.projectious.work/v2
  source_processkit_version: 2.0.0-alpha.1
  target_api_version: processkit.projectious.work/v2
  target_processkit_version: 2.0.0-alpha.1
  apply_mode: one-shot
  started_at: '2026-07-20T14:09:21+00:00'
  applied_at: '2026-07-20T14:09:21+00:00'
---

# Migration briefing

## Summary

Canonicalize the Note spec key order so required type and state metadata precede the body block scalar. This preserves content while avoiding the v0.27.5 pk-doctor parser defect tracked as projectious-work/processkit#88.

## Plan

1. Preserve the Note content and all metadata.
2. Reorder spec keys to title, type, state, body, then the remaining fields using the processkit entity serializer.
3. Re-run pk-doctor and apply this migration after validation.
