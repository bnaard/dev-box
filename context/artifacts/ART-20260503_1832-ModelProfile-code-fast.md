---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1832-ModelProfile-code-fast
  created: '2026-05-03T18:32:00Z'
spec:
  name: Code fast model profile
  kind: model-profile
  profile_id: code-fast
  description: Routine code edits, tests, and small reviews.
  selection:
    model_classes:
    - fast
    - standard
    task_classes:
    - coding
    - debugging
    - code_review
    min_dimensions:
      engineering: 3
      speed: 4
    effort_floor: low
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
    provider: alibaba
    model_spec: ART-20260503_1424-ModelSpec-alibaba-qwen2-5-coder-32b
  - rank: 5
    provider: mistral
    model_spec: ART-20260503_1424-ModelSpec-mistral-codestral
  produced_by: DEC-20260503_1829-LoyalComet
  tags:
  - model-routing
  - provider-neutral
---
# Code fast model profile

Routine code edits, tests, and small reviews.

This artifact is provider-neutral. Concrete provider/model choices live only in the candidate list and are selected after runtime access gates are applied.
