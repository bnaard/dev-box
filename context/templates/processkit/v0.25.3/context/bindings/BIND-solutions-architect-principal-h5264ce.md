---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-solutions-architect-principal-h5264ce
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-solutions-architect
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-general-deep
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Provider-neutral general-deep routing for ROLE-solutions-architect
      principal; concrete model selected by runtime access gates.
  description: Provider-neutral general-deep model assignment for ROLE-solutions-architect
    principal
---
