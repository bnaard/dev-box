---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260820_0746-SureRose-context-archive-created
  created: '2026-08-20T07:46:14+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-08-20T07:46:14+00:00'
  summary: Archived 2 context entities into ARCHIVE-20260820_074604-migration-applied
  subject: ARCHIVE-20260820_074604-migration-applied
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/08/ARCHIVE-20260820_074604-migration-applied.tar.gz
    manifest_path: context/archives/2026/08/ARCHIVE-20260820_074604-migration-applied.json
    entity_ids:
    - MIG-20260720_1408-SnappyComet-canonicalize-note-spec-key-order-for
    - MIG-20260720_1350-ContentSync-processkit-content-sync
---
