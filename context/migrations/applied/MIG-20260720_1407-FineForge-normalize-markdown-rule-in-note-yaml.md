---
apiVersion: processkit.projectious.work/v2
kind: Migration
metadata:
  id: MIG-20260720_1407-FineForge-normalize-markdown-rule-in-note-yaml
  created: '2026-07-20T14:07:58+00:00'
  updated: '2026-07-20T14:08:38+00:00'
spec:
  source: local-project
  kind: data-fix
  state: rejected
  generated_by: migration-management.draft_migration
  generated_at: '2026-07-20T14:07:58+00:00'
  summary: Normalize Markdown rule in Note YAML block scalar
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
  rejected_reason: 'Rejected after refinement: the first triple-hyphen occurs inside
    a Markdown table separator, so changing the content would degrade the Note. The
    safe compatibility repair is metadata key reordering only.'
  rejected_at: '2026-07-20T14:08:38+00:00'
---

# Migration briefing

## Summary

Replace a Markdown horizontal rule inside the Note's YAML block scalar with the equivalent asterisk form so processkit v0.27.5 pk-doctor does not mistake it for the frontmatter delimiter. Upstream parser defect: projectious-work/processkit#88.

## Plan

1. Preserve the Note content and all metadata.
2. Replace line-delimited triple-hyphen Markdown rules in spec.body with triple-asterisk rules using the processkit entity serializer.
3. Re-run pk-doctor and apply this migration after validation.
