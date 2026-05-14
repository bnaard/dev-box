---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-database-engineer-senior-r1-h39bdeb
  created: '2026-04-29T16:13:22+00:00'
spec:
  type: model-assignment
  subject: ROLE-database-engineer
  target: ART-20260503_1832-ModelProfile-code-balanced
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral code-balanced routing for ROLE-database-engineer senior;
      concrete model selected by runtime access gates.
  description: Provider-neutral code-balanced model assignment for ROLE-database-engineer
    senior
---
