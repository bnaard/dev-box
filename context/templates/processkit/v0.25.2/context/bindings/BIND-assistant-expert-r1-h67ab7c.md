---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-assistant-expert-r1-h67ab7c
  created: '2026-04-25T09:55:46+00:00'
spec:
  type: model-assignment
  subject: ROLE-assistant
  target: ART-20260503_1424-ModelSpec-anthropic-claude-haiku
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Seed: Expert assistant — inherits senior defaults pending role-specific
      tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert assistant'
---
