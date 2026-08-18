---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1832-ModelProfile-writing-balanced
  created: '2026-05-03T18:32:00Z'
spec:
  name: Writing balanced model profile
  kind: model-profile
  profile_id: writing-balanced
  description: Style-sensitive documentation and narrative synthesis.
  selection:
    model_classes:
    - standard
    - powerful
    task_classes:
    - summarization
    - documentation
    - writing
    min_dimensions:
      reliability: 4
      breadth: 4
    effort_floor: low
    effort_ceiling: high
  candidates:
  - rank: 1
    provider: anthropic
    model_spec: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  - rank: 2
    provider: openai
    model_spec: ART-20260503_1424-ModelSpec-openai-gpt-5
  - rank: 3
    provider: google
    model_spec: ART-20260503_1424-ModelSpec-google-gemini-2-5-pro
  - rank: 4
    provider: mistral
    model_spec: ART-20260503_1424-ModelSpec-mistral-mistral-medium
  - rank: 5
    provider: cohere
    model_spec: ART-20260503_1424-ModelSpec-cohere-command-r-plus
  produced_by: DEC-20260503_1829-LoyalComet
  tags:
  - model-routing
  - provider-neutral
---
# Writing balanced model profile

Style-sensitive documentation and narrative synthesis.

This artifact is provider-neutral. Concrete provider/model choices live only in the candidate list and are selected after runtime access gates are applied.
