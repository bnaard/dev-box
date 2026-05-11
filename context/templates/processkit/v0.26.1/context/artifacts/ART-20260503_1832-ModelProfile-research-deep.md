---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1832-ModelProfile-research-deep
  created: '2026-05-03T18:32:00Z'
spec:
  name: Research deep model profile
  kind: model-profile
  profile_id: research-deep
  description: Scientific, market, and technical research synthesis.
  selection:
    model_classes:
    - powerful
    task_classes:
    - research
    - scientific_reasoning
    - summarization
    min_dimensions:
      reasoning: 5
      breadth: 4
    effort_floor: high
    effort_ceiling: max
  candidates:
  - rank: 1
    provider: anthropic
    model_spec: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  - rank: 2
    provider: openai
    model_spec: ART-20260503_1424-ModelSpec-openai-gpt-5-pro
  - rank: 3
    provider: google
    model_spec: ART-20260503_1424-ModelSpec-google-gemini-3-1-pro
  - rank: 4
    provider: deepseek
    model_spec: ART-20260503_1424-ModelSpec-deepseek-deepseek-r
  - rank: 5
    provider: xai
    model_spec: ART-20260503_1424-ModelSpec-xai-grok-4-heavy
  produced_by: DEC-20260503_1829-LoyalComet
  tags:
  - model-routing
  - provider-neutral
---
# Research deep model profile

Scientific, market, and technical research synthesis.

This artifact is provider-neutral. Concrete provider/model choices live only in the candidate list and are selected after runtime access gates are applied.
