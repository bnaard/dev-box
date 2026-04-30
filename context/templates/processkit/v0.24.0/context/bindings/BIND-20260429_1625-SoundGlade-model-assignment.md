---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260429_1625-SoundGlade-model-assignment
  created: '2026-04-29T16:25:55+00:00'
spec:
  type: model-assignment
  subject: processkit
  target: MODEL-anthropic-claude-sonnet
  conditions:
    blocked: true
    rationale: User requested OpenAI model routing for this Codex/OpenAI-subscription
      implementation. Veto Anthropic Sonnet in scope=processkit while model-routing
      provider-independence bug is investigated.
  description: 'Implementation scope veto: block Anthropic Sonnet for Codex/OpenAI
    session'
---
