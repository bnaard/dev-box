---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-assistant-senior-h771629
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-assistant
  subject_kind: Role
  target: ART-20260503_1424-ModelSpec-anthropic-claude-haiku
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Senior assistant — still fast, slightly more thinking budget
  description: 'default-pack: Senior assistant — still fast, slightly more thinking
    budget'
---
