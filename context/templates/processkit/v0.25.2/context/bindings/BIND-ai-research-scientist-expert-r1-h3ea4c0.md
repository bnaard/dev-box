---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-ai-research-scientist-expert-r1-h3ea4c0
  created: '2026-04-25T09:55:43+00:00'
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  target: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: 'Seed: Expert ai-research-scientist — inherits senior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert ai-research-scientist'
---
