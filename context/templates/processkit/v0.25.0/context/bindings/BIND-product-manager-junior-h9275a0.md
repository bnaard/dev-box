---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-product-manager-junior-h9275a0
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-product-manager
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior PM — fast iteration on briefs and summaries
  description: 'default-pack: Junior PM — fast iteration on briefs and summaries'
---
