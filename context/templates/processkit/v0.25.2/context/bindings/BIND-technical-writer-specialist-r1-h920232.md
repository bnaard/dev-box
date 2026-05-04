---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-technical-writer-specialist-r1-h920232
  created: '2026-04-25T09:56:11+00:00'
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  target: ART-20260503_1424-ModelSpec-anthropic-claude-haiku
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Seed: Specialist technical-writer — inherits junior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist technical-writer'
---
