---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-qa-engineer-expert-r1-h68a730
  created: '2026-04-25T09:55:59+00:00'
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  target: ART-20260503_1832-ModelProfile-code-balanced
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral code-balanced routing for ROLE-qa-engineer expert;
      concrete model selected by runtime access gates.
  description: Provider-neutral code-balanced model assignment for ROLE-qa-engineer
    expert
---
