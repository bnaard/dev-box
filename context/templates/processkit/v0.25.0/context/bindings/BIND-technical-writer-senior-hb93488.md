---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-technical-writer-senior-hb93488
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Senior tech writer — style-sensitive long-form authoring
  description: 'default-pack: Senior tech writer — style-sensitive long-form authoring'
---
