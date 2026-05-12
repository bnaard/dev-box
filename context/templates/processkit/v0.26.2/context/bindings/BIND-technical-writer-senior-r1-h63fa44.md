---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-technical-writer-senior-r1-h63fa44
  created: '2026-04-29T16:14:18+00:00'
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  target: ART-20260503_1832-ModelProfile-writing-balanced
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Provider-neutral writing-balanced routing for ROLE-technical-writer
      senior; concrete model selected by runtime access gates.
  description: Provider-neutral writing-balanced model assignment for ROLE-technical-writer
    senior
---
