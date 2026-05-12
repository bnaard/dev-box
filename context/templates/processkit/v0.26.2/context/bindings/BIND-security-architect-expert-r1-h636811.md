---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-security-architect-expert-r1-h636811
  created: '2026-04-25T09:56:06+00:00'
spec:
  type: model-assignment
  subject: ROLE-security-architect
  target: ART-20260503_1832-ModelProfile-governed-deep
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Provider-neutral governed-deep routing for ROLE-security-architect
      expert; concrete model selected by runtime access gates.
  description: Provider-neutral governed-deep model assignment for ROLE-security-architect
    expert
---
