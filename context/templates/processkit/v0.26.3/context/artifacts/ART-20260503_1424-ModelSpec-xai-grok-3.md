---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-xai-grok-3
  created: '2026-05-03T14:24:00Z'
spec:
  name: Grok 3
  kind: model-spec
  format: markdown
  provider: xai
  family: grok-3
  legacy_model_id: MODEL-xai-grok-3
  profile_ids:
  - grok-3
  versions:
  - version_id: '3'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 3.0
      output: 15.0
    pricing_note: Available via xAI API and X Premium subscription; real-time web
      access available in some modes
    lifecycle: legacy
    source_urls: &id001
    - https://x.ai/api
    - https://docs.x.ai/docs/models
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 5
    engineering: 4
    speed: 3
    breadth: 4
    reliability: 4
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.x.ai/
  rationale: Frontier flagship
  lifecycle: legacy
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 4
    algorithm_design: 4
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 4
    test_generation: 3
    repo_navigation: 4
    agentic_workflow: 4
    tool_calling: 4
    structured_output: 4
    data_extraction: 4
    summarization: 4
    long_context_synthesis: 4
    rag: 4
    citation_answering: 4
    research_synthesis: 4
    math_reasoning: 5
    scientific_reasoning: 4
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 3
    multilingual_chat: 3
    classification: 3
    sentiment_analysis: 3
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 4
    sql_generation: 4
    spreadsheet_analysis: 4
    ocr: 4
    image_understanding: 4
    chart_understanding: 4
    diagram_reasoning: 4
    voice: 3
    audio_transcription: 3
    video_understanding: 3
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 2
    self_hosted_enterprise: 2
---
