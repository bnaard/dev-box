---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-security-architect-specialist-r1-hc3ae4a
  created: '2026-04-25T09:56:04+00:00'
spec:
  type: model-assignment
  subject: ROLE-security-architect
  target: ART-20260503_1832-ModelProfile-governed-deep
  target_kind: Artifact
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral governed-deep routing for ROLE-security-architect
      specialist; concrete model selected by runtime access gates.
  description: Provider-neutral governed-deep model assignment for ROLE-security-architect
    specialist
---
