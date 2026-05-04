---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-assistant-principal-hb5ac7b
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-assistant
  subject_kind: Role
  target: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Principal assistant — full frontier capabilities when needed
  description: 'default-pack: Principal assistant — full frontier capabilities when
    needed'
---
