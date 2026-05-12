---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-product-manager-specialist-r1-h42cae9
  created: '2026-04-25T09:55:52+00:00'
spec:
  type: model-assignment
  subject: ROLE-product-manager
  target: ART-20260503_1832-ModelProfile-general-fast
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Provider-neutral general-fast routing for ROLE-product-manager specialist;
      concrete model selected by runtime access gates.
  description: Provider-neutral general-fast model assignment for ROLE-product-manager
    specialist
---
