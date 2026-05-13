---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260513_1256-CleverBridge-migration-applied
  created: '2026-05-13T12:56:31+00:00'
spec:
  event_type: migration.applied
  timestamp: '2026-05-13T12:56:31+00:00'
  summary: Strict processkit schema and storage migration applied
  actor: codex
  subject: DEC-20260513_1249-GrandSpruce-strictly-migrate-processkit-context-instead-of
  subject_kind: DecisionRecord
  details:
    scope: processkit context plus scaffolded instructions
    changes:
    - Removed legacy schema vocabulary allowlists
    - Removed pk-doctor mixed filename policy exceptions
    - Migrated legacy LogEntry event types and feature WorkItem type
    - Migrated Role and Binding IDs, filenames, and references to deterministic policy
    - Updated AGENTS, provider pointer template, processkit skills, compliance contract,
      and docs to require migration over grandfathering
    verification_pending:
    - reindex
    - pk-doctor
    - aibox doctor
---
