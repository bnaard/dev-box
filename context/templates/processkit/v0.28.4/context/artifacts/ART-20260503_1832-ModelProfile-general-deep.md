---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1832-ModelProfile-general-deep
  created: '2026-05-03T18:32:00Z'
spec:
  name: General deep model profile
  kind: model-profile
  profile_id: general-deep
  description: High-effort cross-domain reasoning.
  selection:
    model_classes:
    - powerful
    min_dimensions:
      reasoning: 5
      reliability: 4
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
    provider: xai
    model_spec: ART-20260503_1424-ModelSpec-xai-grok-4-heavy
  - rank: 5
    provider: deepseek
    model_spec: ART-20260503_1424-ModelSpec-deepseek-deepseek-r
  produced_by: DEC-20260503_1829-LoyalComet
  tags:
  - model-routing
  - provider-neutral
---
# General deep model profile

High-effort cross-domain reasoning.

This artifact is provider-neutral. Concrete provider/model choices live only in the candidate list and are selected after runtime access gates are applied.
