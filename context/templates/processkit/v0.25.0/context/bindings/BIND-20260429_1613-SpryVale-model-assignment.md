---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260429_1613-SpryVale-model-assignment
  created: '2026-04-29T16:13:38+00:00'
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  target: MODEL-openai-gpt-5
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: OpenAI GPT-5 equivalent to Claude Sonnet for concise technical documentation
      updates in Codex/OpenAI-subscription sessions.
  description: OpenAI senior technical-writer binding for index-search docs
---
