---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-ai-research-scientist-expert-r1-h3ea4c0
  created: '2026-04-25T09:55:43+00:00'
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  target: ART-20260503_1832-ModelProfile-research-deep
  target_kind: Artifact
  conditions:
    seniority: expert
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Provider-neutral research-deep routing for ROLE-ai-research-scientist
      expert; concrete model selected by runtime access gates.
  description: Provider-neutral research-deep model assignment for ROLE-ai-research-scientist
    expert
---
