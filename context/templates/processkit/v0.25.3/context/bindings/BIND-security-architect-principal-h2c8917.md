---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-security-architect-principal-h2c8917
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-security-architect
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-governed-deep
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: extra-high
    effort_ceiling: max
    rationale: Provider-neutral governed-deep routing for ROLE-security-architect
      principal; concrete model selected by runtime access gates.
  description: Provider-neutral governed-deep model assignment for ROLE-security-architect
    principal
---
