---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260422_0001-MigratedBinding-qa-engineer-junior-h796cc2
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-20260422_0001-MigratedRole-qa-engineer
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-code-fast
  target_kind: Artifact
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Provider-neutral code-fast routing for ROLE-20260422_0001-MigratedRole-qa-engineer junior; concrete model selected by runtime access gates.
  description: Provider-neutral code-fast model assignment for ROLE-20260422_0001-MigratedRole-qa-engineer junior
---
