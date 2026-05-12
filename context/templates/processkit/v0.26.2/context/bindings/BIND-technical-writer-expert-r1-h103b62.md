---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-technical-writer-expert-r1-h103b62
  created: '2026-04-25T09:56:13+00:00'
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  target: ART-20260503_1832-ModelProfile-writing-balanced
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Provider-neutral writing-balanced routing for ROLE-technical-writer
      expert; concrete model selected by runtime access gates.
  description: Provider-neutral writing-balanced model assignment for ROLE-technical-writer
    expert
---
