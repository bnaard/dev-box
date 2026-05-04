---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-product-manager-principal-h723e55
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-product-manager
  subject_kind: Role
  target: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Principal PM — frontier reasoning for portfolio-scale trade-offs
  description: 'default-pack: Principal PM — frontier reasoning for portfolio-scale
    trade-offs'
---
