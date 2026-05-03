---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-ai-research-scientist-junior-ha25e5b
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: junior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Junior AI researcher — familiar with agentic+tool-use patterns
  description: 'default-pack: Junior AI researcher — familiar with agentic+tool-use
    patterns'
---
