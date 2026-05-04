---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-processkit-default-r1-h86a081
  created: '2026-04-29T16:25:55+00:00'
spec:
  type: model-assignment
  subject: processkit
  target: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  target_kind: Artifact
  conditions:
    blocked: true
    rationale: User requested OpenAI model routing for this Codex/OpenAI-subscription
      implementation. Veto Anthropic Sonnet in scope=processkit while model-routing
      provider-independence bug is investigated.
  description: 'Implementation scope veto: block Anthropic Sonnet for Codex/OpenAI
    session'
---
