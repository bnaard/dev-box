---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260422_0001-MigratedBinding-technical-writer-senior-hb93488
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-20260422_0001-MigratedRole-technical-writer
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-writing-balanced
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Provider-neutral writing-balanced routing for ROLE-20260422_0001-MigratedRole-technical-writer senior; concrete model selected by runtime access gates.
  description: Provider-neutral writing-balanced model assignment for ROLE-20260422_0001-MigratedRole-technical-writer senior
---
