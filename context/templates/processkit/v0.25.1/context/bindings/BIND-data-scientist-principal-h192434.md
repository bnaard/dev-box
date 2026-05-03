---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-data-scientist-principal-h192434
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-data-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Principal data scientist — methodology design and audits
  description: 'default-pack: Principal data scientist — methodology design and audits'
---
