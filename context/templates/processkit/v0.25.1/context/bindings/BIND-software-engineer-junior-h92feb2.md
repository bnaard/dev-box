---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-software-engineer-junior-h92feb2
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-software-engineer
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior IC coding — fast, cheap, sufficient for routine tickets
  description: 'default-pack: Junior IC coding — fast, cheap, sufficient for routine
    tickets'
---
