---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-technical-writer-principal-h56b361
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-writing-balanced
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral writing-balanced routing for ROLE-technical-writer
      principal; concrete model selected by runtime access gates.
  description: Provider-neutral writing-balanced model assignment for ROLE-technical-writer
    principal
---
