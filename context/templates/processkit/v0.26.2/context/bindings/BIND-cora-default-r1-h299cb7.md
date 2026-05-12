---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-cora-default-r1-h299cb7
  created: '2026-04-29T16:09:45+00:00'
spec:
  type: model-assignment
  subject: TEAMMEMBER-cora
  target: ART-20260503_1832-ModelProfile-general-balanced
  target_kind: Artifact
  conditions:
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral general-balanced routing for TEAMMEMBER-cora default;
      concrete model selected by runtime access gates.
  description: Provider-neutral general-balanced model assignment for TEAMMEMBER-cora
    default
---
