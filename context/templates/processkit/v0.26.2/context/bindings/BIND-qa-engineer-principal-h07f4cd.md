---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-qa-engineer-principal-h07f4cd
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-code-deep
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Provider-neutral code-deep routing for ROLE-qa-engineer principal;
      concrete model selected by runtime access gates.
  description: Provider-neutral code-deep model assignment for ROLE-qa-engineer principal
---
