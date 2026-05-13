---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260513_1348-RobustWren-context-archive-created
  created: '2026-05-13T13:48:29+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-05-13T13:48:29+00:00'
  summary: Archived 1 context entities into ARCHIVE-20260513_134823-migration-applied
  subject: ARCHIVE-20260513_134823-migration-applied
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/05/ARCHIVE-20260513_134823-migration-applied.tar.gz
    manifest_path: context/archives/2026/05/ARCHIVE-20260513_134823-migration-applied.json
    entity_ids:
    - MIG-RUNTIME-20260513T134624
---
