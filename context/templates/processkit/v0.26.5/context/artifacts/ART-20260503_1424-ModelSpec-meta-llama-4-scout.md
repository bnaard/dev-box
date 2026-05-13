---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-meta-llama-4-scout
  created: '2026-05-03T14:24:00Z'
spec:
  name: Llama 4 Scout
  kind: model-spec
  format: markdown
  provider: meta
  family: llama-4-scout
  legacy_model_id: MODEL-meta-llama-4-scout
  profile_ids:
  - llama-4-scout
  versions:
  - version_id: '1'
    status: ga
    context_window: 1000000
    pricing_note: Open-weight/self-hosted or third-party hosted; infra/API provider
      cost varies.
    lifecycle: active
    source_urls: &id001
    - https://llama.meta.com/
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 3
    engineering: 3
    speed: 4
    breadth: 5
    reliability: 3
    governance: 5
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://ai.meta.com/
  rationale: Open-weight efficient multimodal
  lifecycle: active
  source_urls: *id001
  model_classes:
  - fast
  - standard
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
    tool_calling: 4
    structured_output: 4
    data_extraction: 4
    summarization: 4
    long_context_synthesis: 4
    rag: 4
    citation_answering: 4
    research_synthesis: 4
    math_reasoning: 3
    scientific_reasoning: 4
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
    translation: 4
    multilingual_chat: 4
    classification: 4
    sentiment_analysis: 4
    creative_writing: 4
    marketing_copy: 4
    data_analysis: 3
    sql_generation: 3
    spreadsheet_analysis: 3
    ocr: 4
    image_understanding: 4
    chart_understanding: 4
    diagram_reasoning: 4
    voice: 4
    audio_transcription: 4
    video_understanding: 4
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 5
    self_hosted_enterprise: 5
---
