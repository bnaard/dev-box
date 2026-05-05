---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-xai-grok-4-1-fast
  created: '2026-05-03T14:24:00Z'
spec:
  name: Grok 4.1 Fast
  kind: model-spec
  format: markdown
  provider: xai
  family: grok-4.1-fast
  legacy_model_id: MODEL-xai-grok-4-1-fast
  profile_ids:
  - grok-4.1-fast
  versions:
  - version_id: '1'
    status: ga
    context_window: 2000000
    pricing_usd_per_1m:
      input: 0.2
      output: 0.5
    pricing_note: xAI Grok 4.1 Fast pricing.
    lifecycle: active
    source_urls: &id001
    - https://x.ai/api
    - https://docs.x.ai/docs/models
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 4
    engineering: 5
    speed: 5
    breadth: 5
    reliability: 5
    governance: 2
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.x.ai/
  rationale: Fast frontier
  lifecycle: active
  source_urls: *id001
  model_classes:
  - fast
  task_suitability:
    architecture: 5
    algorithm_design: 4
    debugging: 5
    implementation: 5
    refactoring: 5
    code_review: 5
    test_generation: 5
    repo_navigation: 5
    agentic_workflow: 5
    tool_calling: 5
    structured_output: 5
    data_extraction: 5
    summarization: 5
    long_context_synthesis: 5
    rag: 5
    citation_answering: 5
    research_synthesis: 5
    math_reasoning: 4
    scientific_reasoning: 4
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
    translation: 5
    multilingual_chat: 5
    classification: 5
    sentiment_analysis: 5
    creative_writing: 5
    marketing_copy: 5
    data_analysis: 5
    sql_generation: 5
    spreadsheet_analysis: 5
    ocr: 5
    image_understanding: 5
    chart_understanding: 5
    diagram_reasoning: 5
    voice: 5
    audio_transcription: 5
    video_understanding: 5
    low_latency_chat: 5
    bulk_generation: 5
    privacy_sensitive: 3
    self_hosted_enterprise: 3
---
