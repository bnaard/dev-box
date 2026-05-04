---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-software-engineer-senior-r1-h8ea632
  created: '2026-04-29T16:13:08+00:00'
spec:
  type: model-assignment
  subject: ROLE-software-engineer
  target: ART-20260503_1424-ModelSpec-openai-gpt-5
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: OpenAI GPT-5 equivalent to Claude Sonnet for senior implementation
      work in Codex/OpenAI-subscription sessions.
  description: OpenAI senior software-engineer binding for index-search implementation
---
