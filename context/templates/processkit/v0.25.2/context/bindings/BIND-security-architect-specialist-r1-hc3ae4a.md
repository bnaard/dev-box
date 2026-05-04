---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-security-architect-specialist-r1-hc3ae4a
  created: '2026-04-25T09:56:04+00:00'
spec:
  type: model-assignment
  subject: ROLE-security-architect
  target: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Specialist security-architect — inherits junior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist security-architect'
---
