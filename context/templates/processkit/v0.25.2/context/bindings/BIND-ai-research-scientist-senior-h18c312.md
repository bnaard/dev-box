---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-ai-research-scientist-senior-h18c312
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  subject_kind: Role
  target: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  target_kind: Artifact
  conditions:
    seniority: senior
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Senior AI researcher — frontier reasoning for novel ML work
  description: 'default-pack: Senior AI researcher — frontier reasoning for novel
    ML work'
---
