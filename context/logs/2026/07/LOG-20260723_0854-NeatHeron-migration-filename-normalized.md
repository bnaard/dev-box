---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260723_0854-NeatHeron-migration-filename-normalized
  created: '2026-07-23T08:54:05+00:00'
spec:
  event_type: migration.filename-normalized
  timestamp: '2026-07-23T08:54:05+00:00'
  summary: 'Migration ID normalized: ''MIG-20260717T143144'' → ''MIG-20260717_1431-ContentSync'''
  subject: MIG-20260717_1431-ContentSync
  subject_kind: Migration
  actor: processkit-migration-management
  details:
    old_id: MIG-20260717T143144
    new_id: MIG-20260717_1431-ContentSync
    updated_references:
    - context/migrations/INDEX.md
    preserved_history:
    - context/logs/2026/07/LOG-20260717_1432-CordialWillow-migration-applied.md
    - context/logs/2026/07/LOG-20260717_1432-TrueRose-migration-transitioned.md
---
