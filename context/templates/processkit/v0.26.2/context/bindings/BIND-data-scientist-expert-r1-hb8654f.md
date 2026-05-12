---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-data-scientist-expert-r1-hb8654f
  created: '2026-04-25T09:55:50+00:00'
spec:
  type: model-assignment
  subject: ROLE-data-scientist
  target: ART-20260503_1832-ModelProfile-research-deep
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral research-deep routing for ROLE-data-scientist expert;
      concrete model selected by runtime access gates.
  description: Provider-neutral research-deep model assignment for ROLE-data-scientist
    expert
---
