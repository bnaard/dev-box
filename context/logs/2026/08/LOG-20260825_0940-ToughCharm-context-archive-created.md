---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260825_0940-ToughCharm-context-archive-created
  created: '2026-08-25T09:40:17+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-08-25T09:40:17+00:00'
  summary: Archived 1 context entities into ARCHIVE-20260825_094006-migration-applied
  subject: ARCHIVE-20260825_094006-migration-applied
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/08/ARCHIVE-20260825_094006-migration-applied.tar.gz
    manifest_path: context/archives/2026/08/ARCHIVE-20260825_094006-migration-applied.json
    entity_ids:
    - MIG-20260722_1623-ContentSync-processkit-content-sync
---
