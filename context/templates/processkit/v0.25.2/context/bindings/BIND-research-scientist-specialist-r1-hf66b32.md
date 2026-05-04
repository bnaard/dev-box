---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-research-scientist-specialist-r1-hf66b32
  created: '2026-04-25T09:56:01+00:00'
spec:
  type: model-assignment
  subject: ROLE-research-scientist
  target: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Specialist research-scientist — inherits junior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist research-scientist'
---
