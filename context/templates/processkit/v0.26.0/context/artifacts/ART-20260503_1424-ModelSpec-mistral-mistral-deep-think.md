---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-mistral-mistral-deep-think
  created: '2026-05-03T14:24:00Z'
spec:
  name: Mistral Deep Think
  kind: model-spec
  format: markdown
  provider: mistral
  family: mistral-deep-think
  legacy_model_id: MODEL-mistral-mistral-deep-think
  profile_ids:
  - mistral-deep-think
  versions:
  - version_id: '1'
    status: preview
    context_window: 128000
    pricing_usd_per_1m:
      input: 1.5
      output: 6.0
    pricing_note: Mistral's reasoning-mode model; EU-based alternative to o3/DeepSeek
      R1 for regulated data
    lifecycle: unverified
    source_urls: &id001
    - https://docs.mistral.ai/models/overview
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 4
    engineering: 4
    speed: 2
    breadth: 3
    reliability: 4
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.mistral.ai/
  rationale: Reasoning / extended thinking
  lifecycle: unverified
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 3
    algorithm_design: 3
    debugging: 3
    implementation: 3
    refactoring: 3
    code_review: 3
    test_generation: 3
    repo_navigation: 3
    agentic_workflow: 3
    tool_calling: 3
    structured_output: 3
    data_extraction: 3
    summarization: 3
    long_context_synthesis: 3
    rag: 3
    citation_answering: 3
    research_synthesis: 3
    math_reasoning: 3
    scientific_reasoning: 3
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 3
    multilingual_chat: 3
    classification: 2
    sentiment_analysis: 2
    creative_writing: 2
    marketing_copy: 2
    data_analysis: 3
    sql_generation: 3
    spreadsheet_analysis: 3
    ocr: 2
    image_understanding: 2
    chart_understanding: 2
    diagram_reasoning: 2
    voice: 2
    audio_transcription: 2
    video_understanding: 2
    low_latency_chat: 2
    bulk_generation: 2
    privacy_sensitive: 3
    self_hosted_enterprise: 3
---
