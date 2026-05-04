---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-security-architect-principal-h2c8917
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-security-architect
  subject_kind: Role
  target: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  target_kind: Artifact
  conditions:
    seniority: principal
    rank: 1
    effort_floor: extra-high
    effort_ceiling: max
    rationale: Principal security architect — adversarial deep-dive; G:5
  description: 'default-pack: Principal security architect — adversarial deep-dive;
    G:5'
---
