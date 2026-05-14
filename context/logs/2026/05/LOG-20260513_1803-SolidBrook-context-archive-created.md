---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260513_1803-SolidBrook-context-archive-created
  created: '2026-05-13T18:03:05+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-05-13T18:03:05+00:00'
  summary: Archived 10 context entities into ARCHIVE-20260513_180259-migration-rejected
  subject: ARCHIVE-20260513_180259-migration-rejected
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/05/ARCHIVE-20260513_180259-migration-rejected.tar.gz
    manifest_path: context/archives/2026/05/ARCHIVE-20260513_180259-migration-rejected.json
    entity_ids:
    - MIG-RUNTIME-20260513T145522
    - MIG-RUNTIME-20260512T044831
    - disabled-harness-state-REJECTED
    - MIG-RUNTIME-20260508T152425
    - MIG-RUNTIME-20260508T115429
    - MIG-RUNTIME-20260502T072936
    - MIG-RUNTIME-20260502T071656
    - MIG-20260429T100822
    - MIG-20260425T235248
    - MIG-RUNTIME-20260425T235247
---
