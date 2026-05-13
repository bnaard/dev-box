---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260513_1230-ValiantPanda-grandfather-historical-aibox-dogfood-schema-vocabulary
  created: '2026-05-13T12:30:44+00:00'
  updated: '2026-05-13T12:56:25+00:00'
spec:
  title: Grandfather historical aibox dogfood schema vocabulary after processkit v0.26.3
  state: superseded
  decision: Keep historical aibox dogfood WorkItem and LogEntry vocabulary valid in
    the live project schema while processkit upstream decides whether to restore or
    migrate it.
  context: Requested resolution of all pk-doctor errors, warnings, and actionable
    infos after integrating processkit v0.26.3. No schema-update MCP exists, so this
    records the governance decision for the local schema compatibility patch.
  rationale: 'processkit v0.26.3 removed vocabulary that already exists in append-only
    logs and old work items, causing pk-doctor regressions without changing the historical
    project facts. The live aibox dogfood context must remain doctor-clean while upstream
    processkit issue #46 tracks the broader compatibility policy.'
  consequences: New authoring can continue using the current standard vocabulary,
    while existing historical records remain valid. If processkit ships an official
    migration or grandfathering policy, this local compatibility patch can be reconciled
    with that release.
  deciders:
  - TEAMMEMBER-cora
  decided_at: '2026-05-13T12:30:44+00:00'
  superseded_by: DEC-20260513_1249-GrandSpruce-strictly-migrate-processkit-context-instead-of
---
