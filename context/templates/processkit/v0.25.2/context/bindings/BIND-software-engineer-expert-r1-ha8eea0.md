---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-software-engineer-expert-r1-ha8eea0
  created: '2026-04-25T09:55:36+00:00'
spec:
  type: model-assignment
  subject: ROLE-software-engineer
  target: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Expert software-engineer — inherits senior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert software-engineer — inherits senior defaults pending
    role-specific tuning'
---
