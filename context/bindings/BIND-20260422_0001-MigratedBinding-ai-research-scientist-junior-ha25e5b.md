---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260422_0001-MigratedBinding-ai-research-scientist-junior-ha25e5b
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-20260422_0001-MigratedRole-ai-research-scientist
  subject_kind: Role
  target: ART-20260503_1832-ModelProfile-research-deep
  target_kind: Artifact
  conditions:
    seniority: junior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Provider-neutral research-deep routing for ROLE-20260422_0001-MigratedRole-ai-research-scientist junior; concrete model selected by runtime access gates.
  description: Provider-neutral research-deep model assignment for ROLE-20260422_0001-MigratedRole-ai-research-scientist junior
---
