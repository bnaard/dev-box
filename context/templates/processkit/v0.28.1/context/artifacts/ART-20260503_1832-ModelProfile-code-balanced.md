---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1832-ModelProfile-code-balanced
  created: '2026-05-03T18:32:00Z'
spec:
  name: Code balanced model profile
  kind: model-profile
  profile_id: code-balanced
  description: Repository-scale implementation and review.
  selection:
    model_classes:
    - standard
    - powerful
    task_classes:
    - coding
    - architecture
    - debugging
    min_dimensions:
      engineering: 4
      reliability: 4
    effort_floor: medium
    effort_ceiling: high
  candidates:
  - rank: 1
    provider: anthropic
    model_spec: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  - rank: 2
    provider: openai
    model_spec: ART-20260503_1424-ModelSpec-openai-gpt-5
  - rank: 3
    provider: openai
    model_spec: ART-20260503_1424-ModelSpec-openai-gpt-5-2-codex
  - rank: 4
    provider: google
    model_spec: ART-20260503_1424-ModelSpec-google-gemini-2-5-pro
  - rank: 5
    provider: alibaba
    model_spec: ART-20260503_1424-ModelSpec-alibaba-qwen3-coder
  - rank: 6
    provider: mistral
    model_spec: ART-20260503_1424-ModelSpec-mistral-codestral
  produced_by: DEC-20260503_1829-LoyalComet
  tags:
  - model-routing
  - provider-neutral
---
# Code balanced model profile

Repository-scale implementation and review.

This artifact is provider-neutral. Concrete provider/model choices live only in the candidate list and are selected after runtime access gates are applied.
