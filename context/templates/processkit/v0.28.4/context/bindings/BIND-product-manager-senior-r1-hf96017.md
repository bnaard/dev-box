---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-product-manager-senior-r1-hf96017
  created: '2026-04-29T16:09:53+00:00'
spec:
  type: model-assignment
  subject: ROLE-product-manager
  target: ART-20260503_1832-ModelProfile-general-balanced
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral general-balanced routing for ROLE-product-manager
      senior; concrete model selected by runtime access gates.
  description: Provider-neutral general-balanced model assignment for ROLE-product-manager
    senior
---
