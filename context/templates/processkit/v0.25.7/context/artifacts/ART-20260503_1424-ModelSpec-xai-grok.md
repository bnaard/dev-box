---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-xai-grok
  created: '2026-05-03T14:24:00Z'
spec:
  name: Grok 4.20
  kind: model-spec
  format: markdown
  provider: xai
  family: grok
  legacy_model_id: MODEL-xai-grok
  profile_ids:
  - grok-4.20
  versions:
  - version_id: '4.20'
    status: ga
    context_window: 2000000
    pricing_usd_per_1m:
      input: 2.0
      output: 6.0
    pricing_note: xAI Grok 4.20 API pricing.
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
    reasoning: 5
    engineering: 5
    speed: 4
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
  rationale: Frontier reasoning
  lifecycle: active
  source_urls: *id001
  model_classes:
  - standard
  - powerful
  task_suitability:
    architecture: 5
    algorithm_design: 5
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
    math_reasoning: 5
    scientific_reasoning: 5
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
    translation: 5
    multilingual_chat: 5
    classification: 4
    sentiment_analysis: 4
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
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 3
    self_hosted_enterprise: 3
---
