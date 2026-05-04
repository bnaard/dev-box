---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-technical-writer-senior-r1-h93e646
  created: '2026-04-29T16:13:38+00:00'
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  target: ART-20260503_1424-ModelSpec-openai-gpt-5
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: OpenAI GPT-5 equivalent to Claude Sonnet for concise technical documentation
      updates in Codex/OpenAI-subscription sessions.
  description: OpenAI senior technical-writer binding for index-search docs
---
