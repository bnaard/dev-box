---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260429_1613-PluckyGarnet-model-assignment
  created: '2026-04-29T16:13:22+00:00'
spec:
  type: model-assignment
  subject: ROLE-database-engineer
  target: MODEL-openai-gpt-5
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: OpenAI GPT-5 for SQLite schema, FTS5, and vector-index integration
      work where no prior viable role binding existed.
  description: OpenAI senior database-engineer binding for index-search implementation
---
