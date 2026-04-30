---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260429_1626-LivelyDew-model-assignment
  created: '2026-04-29T16:26:00+00:00'
spec:
  type: model-assignment
  subject: processkit
  target: MODEL-anthropic-claude-opus
  conditions:
    blocked: true
    rationale: User requested OpenAI model routing for this Codex/OpenAI-subscription
      implementation. Veto Anthropic Opus in scope=processkit while model-routing
      provider-independence bug is investigated.
  description: 'Implementation scope veto: block Anthropic Opus for Codex/OpenAI session'
---
