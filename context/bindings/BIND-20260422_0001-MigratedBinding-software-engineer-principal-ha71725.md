---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260422_0001-MigratedBinding-software-engineer-principal-ha71725
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-20260422_0001-MigratedRole-software-engineer
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-code-deep
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Provider-neutral code-deep routing for ROLE-20260422_0001-MigratedRole-software-engineer principal; concrete model selected by runtime access gates.
  description: Provider-neutral code-deep model assignment for ROLE-20260422_0001-MigratedRole-software-engineer principal
---
