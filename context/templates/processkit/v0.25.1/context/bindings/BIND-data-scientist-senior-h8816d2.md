---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-data-scientist-senior-h8816d2
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-data-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Senior data scientist — modelling and notebook work
  description: 'default-pack: Senior data scientist — modelling and notebook work'
---
