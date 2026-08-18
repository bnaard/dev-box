---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260818_1546-FocusedMoon-context-archive-created
  created: '2026-08-18T15:46:33+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-08-18T15:46:33+00:00'
  summary: Archived 2 context entities into ARCHIVE-20260818_154619-migration-applied
  subject: ARCHIVE-20260818_154619-migration-applied
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/08/ARCHIVE-20260818_154619-migration-applied.tar.gz
    manifest_path: context/archives/2026/08/ARCHIVE-20260818_154619-migration-applied.json
    entity_ids:
    - MIG-20260717_1534-ContentSync-processkit-content-sync
    - MIG-20260717_1431-ContentSync
---
