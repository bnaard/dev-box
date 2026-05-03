---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-qa-engineer-senior-h0436ab
  created: 2026-04-22 00:00:00+00:00
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Senior QA — test-strategy and flaky-test triage
  description: 'default-pack: Senior QA — test-strategy and flaky-test triage'
---
