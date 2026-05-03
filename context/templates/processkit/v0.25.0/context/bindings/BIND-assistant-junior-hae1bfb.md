---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-assistant-junior-hae1bfb
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-assistant
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  conditions:
    seniority: junior
    rank: 1
    effort_floor: none
    effort_ceiling: low
    rationale: Junior assistant — fast, cheap, low-latency chat
  description: 'default-pack: Junior assistant — fast, cheap, low-latency chat'
---
