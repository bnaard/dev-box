---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-data-scientist-expert-r1-hb8654f
  created: '2026-04-25T09:55:50+00:00'
spec:
  type: model-assignment
  subject: ROLE-data-scientist
  target: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Expert data-scientist — inherits senior defaults pending role-specific
      tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert data-scientist'
---
