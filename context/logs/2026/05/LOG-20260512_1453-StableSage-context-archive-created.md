---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260512_1453-StableSage-context-archive-created
  created: '2026-05-12T14:53:05+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-05-12T14:53:05+00:00'
  summary: Archived 1 context entities into ARCHIVE-20260512_145300-migration-rejected
  subject: ARCHIVE-20260512_145300-migration-rejected
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/05/ARCHIVE-20260512_145300-migration-rejected.tar.gz
    manifest_path: context/archives/2026/05/ARCHIVE-20260512_145300-migration-rejected.json
    entity_ids:
    - MIG-DISABLED-HARNESS-STATE
---
