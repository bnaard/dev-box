---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260418_1130-CalmHarbor-migration-applied
  created: '2026-04-18T11:30:00Z'
spec:
  event_type: migration.applied
  timestamp: '2026-04-18T11:30:00Z'
  actor: claude-opus-4-7
  summary: Applied MIG-20260418T090634 — processkit v0.17.0 -> v0.18.1 (8 no-op conflicts,
    49 new accepted, 23 removed deleted)
  related:
  - MIG-20260418T090634
  details:
    transitioned_from: pending
    transitioned_to: applied
    files_deleted: 23
    files_accepted_new: 49
    conflicts_resolved: 8
    conflicts_decision: |
      All 8 conflicts resolved as no-ops. The 4 skill-gate scripts were pre-patched in the
      prior session (LOG-20260418_0900) as the hookEventName intermediate hotfix and now
      match v0.18.1 byte-for-byte. Schemas and skills/INDEX.md were already at v0.18.1.
      AGENTS.md keeps local v2 compliance markers — upstream still ships v1 markers, this
      gap is a known upstream inconsistency tracked by processkit (handover open_thread #3).
---
