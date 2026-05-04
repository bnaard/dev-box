---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-ai-research-scientist-specialist-r1-h52964c
  created: '2026-04-25T09:55:41+00:00'
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  target: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Specialist ai-research-scientist — inherits junior defaults
      pending role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist ai-research-scientist'
---
