---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-research-scientist-expert-r1-h3e65e9
  created: '2026-04-25T09:56:02+00:00'
spec:
  type: model-assignment
  subject: ROLE-research-scientist
  target: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: 'Seed: Expert research-scientist — inherits senior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert research-scientist'
---
