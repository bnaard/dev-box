---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-technical-writer-principal-h56b361
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  conditions:
    seniority: principal
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Principal tech writer — reference docs and API-design prose
  description: 'default-pack: Principal tech writer — reference docs and API-design
    prose'
---
