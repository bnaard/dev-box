---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-assistant-senior-h771629
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-assistant
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-general-balanced
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Provider-neutral general-balanced routing for ROLE-assistant senior;
      concrete model selected by runtime access gates.
  description: Provider-neutral general-balanced model assignment for ROLE-assistant
    senior
---
