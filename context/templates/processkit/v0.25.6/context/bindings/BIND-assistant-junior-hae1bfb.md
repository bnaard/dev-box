---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-assistant-junior-hae1bfb
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-assistant
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-general-fast
  target_kind: Artifact
  conditions:
    seniority: junior
    rank: 1
    effort_floor: none
    effort_ceiling: low
    rationale: Provider-neutral general-fast routing for ROLE-assistant junior; concrete
      model selected by runtime access gates.
  description: Provider-neutral general-fast model assignment for ROLE-assistant junior
---
