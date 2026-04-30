---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260429_1613-ProudFox-model-assignment
  created: '2026-04-29T16:13:08+00:00'
spec:
  type: model-assignment
  subject: ROLE-software-engineer
  target: MODEL-openai-gpt-5
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: OpenAI GPT-5 equivalent to Claude Sonnet for senior implementation
      work in Codex/OpenAI-subscription sessions.
  description: OpenAI senior software-engineer binding for index-search implementation
---
