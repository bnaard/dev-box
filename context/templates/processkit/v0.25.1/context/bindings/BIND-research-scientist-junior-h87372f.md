---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-research-scientist-junior-h87372f
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-research-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: junior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Junior researcher — paper reading and hypothesis drafting
  description: 'default-pack: Junior researcher — paper reading and hypothesis drafting'
---
