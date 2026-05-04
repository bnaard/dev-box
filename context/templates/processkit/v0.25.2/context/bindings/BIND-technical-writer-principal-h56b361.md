---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-technical-writer-principal-h56b361
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  subject_kind: Role
  target: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Principal tech writer — reference docs and API-design prose
  description: 'default-pack: Principal tech writer — reference docs and API-design
    prose'
---
