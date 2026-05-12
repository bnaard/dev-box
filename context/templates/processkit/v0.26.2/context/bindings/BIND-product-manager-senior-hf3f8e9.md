---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-product-manager-senior-hf3f8e9
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-product-manager
  subject_kind: Role
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
