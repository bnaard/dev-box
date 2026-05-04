---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-software-engineer-senior-r1-h79acd6
  created: '2026-04-29T16:14:08+00:00'
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
    rationale: 'Temporary implementation-scope override: force OpenAI GPT-5 ahead
      of existing same-rank Anthropic default while provider-bias routing bug is investigated
      in BACK-20260429_1610-CleverAsh-model-routing-is-not.'
  description: 'Implementation override: prefer OpenAI for senior software-engineer'
---
