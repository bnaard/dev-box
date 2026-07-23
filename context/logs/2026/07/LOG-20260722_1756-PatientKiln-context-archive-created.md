---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260722_1756-PatientKiln-context-archive-created
  created: '2026-07-22T17:56:54+00:00'
spec:
  event_type: context_archive.created
  timestamp: '2026-07-22T17:56:54+00:00'
  summary: Archived 1 context entities into ARCHIVE-20260722_175647-decisionrecord-superseded
  subject: ARCHIVE-20260722_175647-decisionrecord-superseded
  subject_kind: Archive
  actor: processkit-context-archiving
  details:
    archive_path: context/archives/2026/07/ARCHIVE-20260722_175647-decisionrecord-superseded.tar.gz
    manifest_path: context/archives/2026/07/ARCHIVE-20260722_175647-decisionrecord-superseded.json
    entity_ids:
    - DEC-20260722_1744-SnappyHare-remove-personal-contact-from-sensitive-data
---
