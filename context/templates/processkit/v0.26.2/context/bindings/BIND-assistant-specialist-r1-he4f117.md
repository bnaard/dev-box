---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-assistant-specialist-r1-he4f117
  created: '2026-04-25T09:55:45+00:00'
spec:
  type: model-assignment
  subject: ROLE-assistant
  target: ART-20260503_1832-ModelProfile-general-fast
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: none
    effort_ceiling: low
    rationale: Provider-neutral general-fast routing for ROLE-assistant specialist;
      concrete model selected by runtime access gates.
  description: Provider-neutral general-fast model assignment for ROLE-assistant specialist
---
