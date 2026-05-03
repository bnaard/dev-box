---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260429_1613-StoutAnt-model-assignment
  created: '2026-04-29T16:13:28+00:00'
spec:
  type: model-assignment
  subject: ROLE-machine-learning-engineer
  target: MODEL-openai-gpt-5-pro
  conditions:
    seniority: senior
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: OpenAI GPT-5 Pro for embedding-policy, semantic retrieval, and hybrid
      ranking design in Phase 2.
  description: OpenAI senior machine-learning-engineer binding for sqlite-vec semantic
    search
---
