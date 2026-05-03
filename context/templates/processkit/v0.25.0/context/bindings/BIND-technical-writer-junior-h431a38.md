---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-technical-writer-junior-h431a38
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior tech writer — fast drafts of routine docs
  description: 'default-pack: Junior tech writer — fast drafts of routine docs'
---
