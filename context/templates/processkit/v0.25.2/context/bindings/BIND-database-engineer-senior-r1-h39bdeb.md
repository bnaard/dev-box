---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-database-engineer-senior-r1-h39bdeb
  created: '2026-04-29T16:13:22+00:00'
spec:
  type: model-assignment
  subject: ROLE-database-engineer
  target: ART-20260503_1424-ModelSpec-openai-gpt-5
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: OpenAI GPT-5 for SQLite schema, FTS5, and vector-index integration
      work where no prior viable role binding existed.
  description: OpenAI senior database-engineer binding for index-search implementation
---
