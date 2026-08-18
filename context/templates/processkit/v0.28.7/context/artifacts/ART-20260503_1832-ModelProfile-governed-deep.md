---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1832-ModelProfile-governed-deep
  created: '2026-05-03T18:32:00Z'
spec:
  name: Governed deep model profile
  kind: model-profile
  profile_id: governed-deep
  description: Security, compliance, and privacy-sensitive deep work.
  selection:
    model_classes:
    - powerful
    task_classes:
    - security
    - architecture
    - privacy_sensitive
    min_dimensions:
      reasoning: 4
      governance: 5
    effort_floor: high
    effort_ceiling: max
  candidates:
  - rank: 1
    provider: anthropic
    model_spec: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  - rank: 2
    provider: anthropic
    model_spec: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  - rank: 3
    provider: meta
    model_spec: ART-20260503_1424-ModelSpec-meta-llama-4-maverick
  - rank: 4
    provider: mistral
    model_spec: ART-20260503_1424-ModelSpec-mistral-mistral-large
  - rank: 5
    provider: openai
    model_spec: ART-20260503_1424-ModelSpec-openai-gpt-5-pro
  produced_by: DEC-20260503_1829-LoyalComet
  tags:
  - model-routing
  - provider-neutral
---
# Governed deep model profile

Security, compliance, and privacy-sensitive deep work.

This artifact is provider-neutral. Concrete provider/model choices live only in the candidate list and are selected after runtime access gates are applied.
