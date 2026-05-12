---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-assistant-expert-r1-h67ab7c
  created: '2026-04-25T09:55:46+00:00'
spec:
  type: model-assignment
  subject: ROLE-assistant
  target: ART-20260503_1832-ModelProfile-general-fast
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Provider-neutral general-fast routing for ROLE-assistant expert; concrete
      model selected by runtime access gates.
  description: Provider-neutral general-fast model assignment for ROLE-assistant expert
---
