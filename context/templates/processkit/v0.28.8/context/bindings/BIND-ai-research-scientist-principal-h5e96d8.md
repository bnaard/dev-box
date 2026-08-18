---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-ai-research-scientist-principal-h5e96d8
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-research-deep
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: extra-high
    effort_ceiling: max
    rationale: Provider-neutral research-deep routing for ROLE-ai-research-scientist
      principal; concrete model selected by runtime access gates.
  description: Provider-neutral research-deep model assignment for ROLE-ai-research-scientist
    principal
---
