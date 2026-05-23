---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260522_1949-FocusedEagle-context-archive-created
  created: '2026-05-22T19:49:26+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-05-22T19:49:26+00:00'
  summary: Archived 2 context entities into ARCHIVE-20260522_194920-migration-applied
  subject: ARCHIVE-20260522_194920-migration-applied
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/05/ARCHIVE-20260522_194920-migration-applied.tar.gz
    manifest_path: context/archives/2026/05/ARCHIVE-20260522_194920-migration-applied.json
    entity_ids:
    - MIG-20260518T153318
    - MIG-20260517T170647
---
