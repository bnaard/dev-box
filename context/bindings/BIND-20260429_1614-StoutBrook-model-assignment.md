---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260429_1614-StoutBrook-model-assignment
  created: '2026-04-29T16:14:13+00:00'
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  target: MODEL-openai-gpt-5
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Temporary implementation-scope override: force OpenAI GPT-5 ahead
      of existing same-rank Anthropic default while provider-bias routing bug is investigated
      in BACK-20260429_1610-CleverAsh-model-routing-is-not.'
  description: 'Implementation override: prefer OpenAI for senior QA engineer'
---
