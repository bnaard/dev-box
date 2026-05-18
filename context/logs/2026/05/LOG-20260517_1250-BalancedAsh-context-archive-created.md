---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260517_1250-BalancedAsh-context-archive-created
  created: '2026-05-17T12:50:27+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-05-17T12:50:27+00:00'
  summary: Archived 4 context entities into ARCHIVE-20260517_125021-migration-applied
  subject: ARCHIVE-20260517_125021-migration-applied
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/05/ARCHIVE-20260517_125021-migration-applied.tar.gz
    manifest_path: context/archives/2026/05/ARCHIVE-20260517_125021-migration-applied.json
    entity_ids:
    - MIG-RUNTIME-20260517T113158
    - MIG-RUNTIME-DRIFT-20260516T092734
    - MIG-20260515T183935
    - MIG-RUNTIME-DRIFT-20260515T183935
---
