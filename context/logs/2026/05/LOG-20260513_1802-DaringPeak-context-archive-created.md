---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260513_1802-DaringPeak-context-archive-created
  created: '2026-05-13T18:02:59+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-05-13T18:02:59+00:00'
  summary: Archived 1 context entities into ARCHIVE-20260513_180253-migration-applied
  subject: ARCHIVE-20260513_180253-migration-applied
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/05/ARCHIVE-20260513_180253-migration-applied.tar.gz
    manifest_path: context/archives/2026/05/ARCHIVE-20260513_180253-migration-applied.json
    entity_ids:
    - MIG-DISABLED-HARNESS-STATE
---
