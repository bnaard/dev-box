---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-qa-engineer-junior-h796cc2
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  subject_kind: Role
  target: ART-20260503_1424-ModelSpec-anthropic-claude-haiku
  target_kind: Artifact
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior QA — high-volume test-case generation
  description: 'default-pack: Junior QA — high-volume test-case generation'
---
