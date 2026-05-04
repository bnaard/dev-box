---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-solutions-architect-specialist-r1-h1125a0
  created: '2026-04-25T09:56:08+00:00'
spec:
  type: model-assignment
  subject: ROLE-solutions-architect
  target: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Seed: Specialist solutions-architect — inherits junior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist solutions-architect'
---
