---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1832-ModelProfile-general-fast
  created: '2026-05-03T18:32:00Z'
spec:
  name: General fast model profile
  kind: model-profile
  profile_id: general-fast
  description: Low-latency general assistant work.
  selection:
    model_classes:
    - fast
    min_dimensions:
      speed: 4
      reliability: 3
    effort_floor: none
    effort_ceiling: medium
  candidates:
  - rank: 1
    provider: anthropic
    model_spec: ART-20260503_1424-ModelSpec-anthropic-claude-haiku
  - rank: 2
    provider: openai
    model_spec: ART-20260503_1424-ModelSpec-openai-o4-mini
  - rank: 3
    provider: google
    model_spec: ART-20260503_1424-ModelSpec-google-gemini-2-5-flash
  - rank: 4
    provider: xai
    model_spec: ART-20260503_1424-ModelSpec-xai-grok-4-1-fast
  - rank: 5
    provider: mistral
    model_spec: ART-20260503_1424-ModelSpec-mistral-mistral-small
  produced_by: DEC-20260503_1829-LoyalComet
  tags:
  - model-routing
  - provider-neutral
---
# General fast model profile

Low-latency general assistant work.

This artifact is provider-neutral. Concrete provider/model choices live only in the candidate list and are selected after runtime access gates are applied.
