---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-security-architect-senior-h142586
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-security-architect
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-governed-deep
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Provider-neutral governed-deep routing for ROLE-security-architect
      senior; concrete model selected by runtime access gates.
  description: Provider-neutral governed-deep model assignment for ROLE-security-architect
    senior
---
