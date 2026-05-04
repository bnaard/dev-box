---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-software-engineer-principal-ha71725
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-software-engineer
  subject_kind: Role
  target: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Principal IC coding — reserve for architecture-scale changes
  description: 'default-pack: Principal IC coding — reserve for architecture-scale
    changes'
---
