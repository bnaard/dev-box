---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1832-ModelProfile-code-deep
  created: '2026-05-03T18:32:00Z'
spec:
  name: Code deep model profile
  kind: model-profile
  profile_id: code-deep
  description: Architecture-scale engineering and complex debugging.
  selection:
    model_classes:
    - powerful
    task_classes:
    - architecture
    - coding
    - debugging
    min_dimensions:
      engineering: 5
      reasoning: 4
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
    provider: openai
    model_spec: ART-20260503_1424-ModelSpec-openai-gpt-5-2-codex
  - rank: 4
    provider: google
    model_spec: ART-20260503_1424-ModelSpec-google-gemini-3-1-pro
  - rank: 5
    provider: xai
    model_spec: ART-20260503_1424-ModelSpec-xai-grok-4-heavy
  produced_by: DEC-20260503_1829-LoyalComet
  tags:
  - model-routing
  - provider-neutral
---
# Code deep model profile

Architecture-scale engineering and complex debugging.

This artifact is provider-neutral. Concrete provider/model choices live only in the candidate list and are selected after runtime access gates are applied.
