---
apiVersion: processkit.projectious.work/v2
kind: Binding
metadata:
  id: BIND-processkit-default-r1-he60b97
  created: '2026-04-29T16:09:39+00:00'
spec:
  type: model-assignment
  subject: processkit
  target: ART-20260503_1424-ModelSpec-openai-gpt-5
  target_kind: Artifact
  conditions:
    provider_preference:
    - OpenAI
    rationale: User confirmed this session uses Codex with an OpenAI subscription;
      prefer OpenAI equivalents over Anthropic defaults for this implementation plan.
  description: 'Project model bias: prefer OpenAI for Codex/OpenAI-subscription implementation
    sessions'
---
