---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260429_1609-ShinyPine-model-assignment
  created: '2026-04-29T16:09:39+00:00'
spec:
  type: model-assignment
  subject: processkit
  target: MODEL-openai-gpt-5
  conditions:
    provider_preference:
    - OpenAI
    rationale: User confirmed this session uses Codex with an OpenAI subscription;
      prefer OpenAI equivalents over Anthropic defaults for this implementation plan.
  description: 'Project model bias: prefer OpenAI for Codex/OpenAI-subscription implementation
    sessions'
---
