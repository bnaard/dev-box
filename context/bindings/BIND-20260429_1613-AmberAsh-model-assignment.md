---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260429_1613-AmberAsh-model-assignment
  created: '2026-04-29T16:13:14+00:00'
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  target: MODEL-openai-gpt-5
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: OpenAI GPT-5 equivalent to Claude Sonnet for senior QA strategy and
      test design in Codex/OpenAI-subscription sessions.
  description: OpenAI senior QA binding for index-search implementation
---
