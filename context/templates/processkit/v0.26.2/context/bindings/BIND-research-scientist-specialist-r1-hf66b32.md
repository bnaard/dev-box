---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-research-scientist-specialist-r1-hf66b32
  created: '2026-04-25T09:56:01+00:00'
spec:
  type: model-assignment
  subject: ROLE-research-scientist
  target: ART-20260503_1832-ModelProfile-research-deep
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral research-deep routing for ROLE-research-scientist
      specialist; concrete model selected by runtime access gates.
  description: Provider-neutral research-deep model assignment for ROLE-research-scientist
    specialist
---
