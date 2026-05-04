---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-qa-engineer-senior-r1-h4db0e9
  created: '2026-04-29T16:13:14+00:00'
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  target: ART-20260503_1424-ModelSpec-openai-gpt-5
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: OpenAI GPT-5 equivalent to Claude Sonnet for senior QA strategy and
      test design in Codex/OpenAI-subscription sessions.
  description: OpenAI senior QA binding for index-search implementation
---
