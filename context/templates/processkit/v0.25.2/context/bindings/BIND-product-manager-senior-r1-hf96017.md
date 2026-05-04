---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-product-manager-senior-r1-hf96017
  created: '2026-04-29T16:09:53+00:00'
spec:
  type: model-assignment
  subject: ROLE-product-manager
  target: ART-20260503_1424-ModelSpec-openai-gpt-5
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: OpenAI equivalent to Claude Sonnet for senior PM discovery synthesis
      and acceptance planning.
  description: OpenAI senior product-manager binding for index-search implementation
    plan
---
