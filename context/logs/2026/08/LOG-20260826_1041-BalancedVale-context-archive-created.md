---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260826_1041-BalancedVale-context-archive-created
  created: '2026-08-26T10:41:12+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-08-26T10:41:12+00:00'
  summary: Archived 1 context entities into ARCHIVE-20260826_104056-migration-applied
  subject: ARCHIVE-20260826_104056-migration-applied
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/08/ARCHIVE-20260826_104056-migration-applied.tar.gz
    manifest_path: context/archives/2026/08/ARCHIVE-20260826_104056-migration-applied.json
    entity_ids:
    - MIG-20260726_1903-ContentSync-processkit-content-sync
---
