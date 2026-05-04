---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-machine-learning-engineer-senior-r1-hd54ded
  created: '2026-04-29T16:13:28+00:00'
spec:
  type: model-assignment
  subject: ROLE-machine-learning-engineer
  target: ART-20260503_1424-ModelSpec-openai-gpt-5-pro
  target_kind: Artifact
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
