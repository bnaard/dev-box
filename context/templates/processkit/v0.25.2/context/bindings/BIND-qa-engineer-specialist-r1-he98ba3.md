---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-qa-engineer-specialist-r1-he98ba3
  created: '2026-04-25T09:55:57+00:00'
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  target: ART-20260503_1424-ModelSpec-anthropic-claude-haiku
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Seed: Specialist qa-engineer — inherits junior defaults pending role-specific
      tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist qa-engineer'
---
