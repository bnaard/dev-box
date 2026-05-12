---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260422_0001-MigratedBinding-research-scientist-principal-hb4e3bb
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-20260422_0001-MigratedRole-research-scientist
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-research-deep
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: extra-high
    effort_ceiling: max
    rationale: Provider-neutral research-deep routing for ROLE-20260422_0001-MigratedRole-research-scientist principal; concrete model selected by runtime access gates.
  description: Provider-neutral research-deep model assignment for ROLE-20260422_0001-MigratedRole-research-scientist principal
---
