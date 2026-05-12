---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-product-manager-expert-r1-hf330ef
  created: '2026-04-25T09:55:53+00:00'
spec:
  type: model-assignment
  subject: ROLE-product-manager
  target: ART-20260503_1832-ModelProfile-general-balanced
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral general-balanced routing for ROLE-product-manager
      expert; concrete model selected by runtime access gates.
  description: Provider-neutral general-balanced model assignment for ROLE-product-manager
    expert
---
