---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-research-scientist-senior-h14c6d3
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-research-scientist
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-research-deep
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Provider-neutral research-deep routing for ROLE-research-scientist
      senior; concrete model selected by runtime access gates.
  description: Provider-neutral research-deep model assignment for ROLE-research-scientist
    senior
---
