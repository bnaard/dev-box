---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-google-gemini-flash
  created: '2026-05-03T14:24:00Z'
spec:
  name: Gemini Flash 2.0
  kind: model-spec
  format: markdown
  provider: google
  family: gemini-flash
  legacy_model_id: MODEL-google-gemini-flash
  profile_ids:
  - gemini-flash-2.0
  versions:
  - version_id: '2.0'
    status: ga
    context_window: 1000000
    pricing_usd_per_1m:
      input: 0.075
      output: 0.3
    pricing_note: Extremely low cost; 1M context sweep costs cents; best throughput/dollar
      of any listed model
    lifecycle: legacy
    source_urls: &id001
    - https://ai.google.dev/gemini-api/docs/models
    - https://ai.google.dev/gemini-api/docs/pricing
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 3
    engineering: 3
    speed: 5
    breadth: 5
    reliability: 3
    governance: 2
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier fast
  lifecycle: legacy
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
    scientific_reasoning: 4
    legal_analysis: 2
    medical_admin: 2
    financial_analysis: 2
    translation: 4
    multilingual_chat: 4
    classification: 5
    sentiment_analysis: 5
    creative_writing: 4
    marketing_copy: 4
    data_analysis: 3
    sql_generation: 3
    spreadsheet_analysis: 3
    ocr: 4
    image_understanding: 4
    chart_understanding: 4
    diagram_reasoning: 4
    voice: 5
    audio_transcription: 5
    video_understanding: 4
    low_latency_chat: 5
    bulk_generation: 5
    privacy_sensitive: 2
    self_hosted_enterprise: 2
---
