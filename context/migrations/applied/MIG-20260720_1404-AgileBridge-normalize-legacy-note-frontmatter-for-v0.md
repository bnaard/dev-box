---
apiVersion: processkit.projectious.work/v2
kind: Migration
metadata:
  id: MIG-20260720_1404-AgileBridge-normalize-legacy-note-frontmatter-for-v0
  created: '2026-07-20T14:04:25+00:00'
  updated: '2026-07-20T14:07:00+00:00'
spec:
  source: local-project
  kind: data-fix
  state: rejected
  generated_by: migration-management.draft_migration
  generated_at: '2026-07-20T14:04:25+00:00'
  summary: Normalize legacy Note frontmatter for v0.27.5 schemas
  affected_files:
  - path: context/notes/2026/06/NOTE-20260609_1604-ZestfulQuail-aibox-multitarget-devcontainer-snapshot.md
    classification: changed-locally-only
  affected_groups: []
  plan: ''
  progress_notes: []
  source_api_version: processkit.projectious.work/v2
  source_processkit_version: 2.0.0-alpha.1
  target_api_version: processkit.projectious.work/v2
  target_processkit_version: 2.0.0-alpha.1
  apply_mode: one-shot
  rejected_reason: 'Rejected after diagnosis: the Note is schema-valid; pk-doctor''s
    naive frontmatter delimiter split truncates YAML block scalars containing Markdown
    horizontal rules.'
  rejected_at: '2026-07-20T14:07:00+00:00'
---

# Migration briefing

## Summary

Promote the existing Note's nested type and state fields to canonical root frontmatter so the v0.27.5 generated schema validates it.

## Plan

1. Preserve the Note body and all existing metadata.
2. Add root type=insight and state=captured using the processkit entity serializer.
3. Re-run pk-doctor and apply this migration after validation.
