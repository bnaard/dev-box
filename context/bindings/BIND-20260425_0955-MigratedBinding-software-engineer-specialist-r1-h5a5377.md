---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260425_0955-MigratedBinding-software-engineer-specialist-r1-h5a5377
  created: '2026-04-25T09:55:36+00:00'
spec:
  type: model-assignment
  subject: ROLE-20260422_0001-MigratedRole-software-engineer
  target: ART-20260503_1832-ModelProfile-code-fast
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Provider-neutral code-fast routing for ROLE-20260422_0001-MigratedRole-software-engineer specialist; concrete model selected by runtime access gates.
  description: Provider-neutral code-fast model assignment for ROLE-20260422_0001-MigratedRole-software-engineer specialist
---
