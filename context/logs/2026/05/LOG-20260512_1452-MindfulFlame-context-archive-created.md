---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260512_1452-MindfulFlame-context-archive-created
  created: '2026-05-12T14:52:41+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-05-12T14:52:41+00:00'
  summary: Archived 1 context entities into ARCHIVE-20260512_145235-migration-applied
  subject: ARCHIVE-20260512_145235-migration-applied
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/05/ARCHIVE-20260512_145235-migration-applied.tar.gz
    manifest_path: context/archives/2026/05/ARCHIVE-20260512_145235-migration-applied.json
    entity_ids:
    - MIG-RUNTIME-DRIFT-20260512T141804
---
