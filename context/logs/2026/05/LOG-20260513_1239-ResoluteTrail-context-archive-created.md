---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260513_1239-ResoluteTrail-context-archive-created
  created: '2026-05-13T12:39:48+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-05-13T12:39:48+00:00'
  summary: Archived 1 context entities into ARCHIVE-20260513_123940-migration-rejected
  subject: ARCHIVE-20260513_123940-migration-rejected
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/05/ARCHIVE-20260513_123940-migration-rejected.tar.gz
    manifest_path: context/archives/2026/05/ARCHIVE-20260513_123940-migration-rejected.json
    entity_ids:
    - MIG-DISABLED-HARNESS-STATE
---
