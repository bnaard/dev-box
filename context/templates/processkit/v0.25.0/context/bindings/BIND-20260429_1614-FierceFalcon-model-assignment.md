---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260429_1614-FierceFalcon-model-assignment
  created: '2026-04-29T16:14:18+00:00'
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  target: MODEL-openai-gpt-5
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Temporary implementation-scope override: force OpenAI GPT-5 ahead
      of existing same-rank Anthropic default while provider-bias routing bug is investigated
      in BACK-20260429_1610-CleverAsh-model-routing-is-not.'
  description: 'Implementation override: prefer OpenAI for senior technical-writer'
---
