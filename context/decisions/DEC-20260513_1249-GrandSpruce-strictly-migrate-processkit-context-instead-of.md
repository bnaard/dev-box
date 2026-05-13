---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260513_1249-GrandSpruce-strictly-migrate-processkit-context-instead-of
  created: '2026-05-13T12:49:37+00:00'
  updated: '2026-05-13T12:56:25+00:00'
spec:
  title: Strictly migrate processkit context instead of grandfathering legacy schema
    and filename policy
  state: accepted
  decision: For this aibox repo, processkit doctor compliance should be achieved by
    migrating legacy event types, WorkItem types, Role/Binding IDs, filenames, and
    references to the current schema and deterministic storage policy, not by extending
    legacy schema allowlists or suppressing filename-policy findings.
  rationale: The owner explicitly asked for the stricter path. Strict migration gives
    future doctor runs real signal and prevents new projects or scaffolded instructions
    from treating local exceptions as acceptable steady state.
  alternatives:
  - option: Keep v0.26.4 grandfathering
    reason: Lower risk but allows legacy vocabulary and mixed filename policy to remain
      indefinitely.
  - option: Only document exceptions
    reason: Doctor-clean but does not satisfy strict schema and storage adherence.
  consequences: Historical LogEntry event_type values and one legacy WorkItem type
    will be rewritten to canonical vocabulary. Duplicate migrated Role/Binding entities
    will be removed or renamed to deterministic IDs. Local pk-doctor grandfathering
    branches and legacy schema allowlists will be removed, so future drift becomes
    actionable again.
  decided_at: '2026-05-13T12:49:37+00:00'
  supersedes: DEC-20260513_1232-KeenGlade-accept-mixed-role-and-binding-filename
---
