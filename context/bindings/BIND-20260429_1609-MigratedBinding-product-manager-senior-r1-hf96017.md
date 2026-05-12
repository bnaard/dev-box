---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260429_1609-MigratedBinding-product-manager-senior-r1-hf96017
  created: '2026-04-29T16:09:53+00:00'
spec:
  type: model-assignment
  subject: ROLE-20260422_0001-MigratedRole-product-manager
  target: ART-20260503_1832-ModelProfile-general-balanced
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral general-balanced routing for ROLE-20260422_0001-MigratedRole-product-manager senior; concrete model selected by runtime access gates.
  description: Provider-neutral general-balanced model assignment for ROLE-20260422_0001-MigratedRole-product-manager senior
---
