---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-solutions-architect-expert-r1-h850329
  created: '2026-04-25T09:56:09+00:00'
spec:
  type: model-assignment
  subject: ROLE-solutions-architect
  target: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Expert solutions-architect — inherits senior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert solutions-architect'
---
