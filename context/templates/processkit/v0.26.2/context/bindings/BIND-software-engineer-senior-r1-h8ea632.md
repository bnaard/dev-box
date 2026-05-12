---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-software-engineer-senior-r1-h8ea632
  created: '2026-04-29T16:13:08+00:00'
spec:
  type: model-assignment
  subject: ROLE-software-engineer
  target: ART-20260503_1832-ModelProfile-code-balanced
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral code-balanced routing for ROLE-software-engineer senior;
      concrete model selected by runtime access gates.
  description: Provider-neutral code-balanced model assignment for ROLE-software-engineer
    senior
---
