---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260429_1626-HonestBeam-model-assignment
  created: '2026-04-29T16:26:04+00:00'
spec:
  type: model-assignment
  subject: processkit
  target: MODEL-anthropic-claude-haiku
  conditions:
    blocked: true
    rationale: User requested OpenAI model routing for this Codex/OpenAI-subscription
      implementation. Veto Anthropic Haiku in scope=processkit while model-routing
      provider-independence bug is investigated.
  description: 'Implementation scope veto: block Anthropic Haiku for Codex/OpenAI
    session'
---
