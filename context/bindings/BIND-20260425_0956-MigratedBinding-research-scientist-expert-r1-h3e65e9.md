---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-20260425_0956-MigratedBinding-research-scientist-expert-r1-h3e65e9
  created: '2026-04-25T09:56:02+00:00'
spec:
  type: model-assignment
  subject: ROLE-20260422_0001-MigratedRole-research-scientist
  target: ART-20260503_1832-ModelProfile-research-deep
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Provider-neutral research-deep routing for ROLE-20260422_0001-MigratedRole-research-scientist expert; concrete model selected by runtime access gates.
  description: Provider-neutral research-deep model assignment for ROLE-20260422_0001-MigratedRole-research-scientist expert
---
