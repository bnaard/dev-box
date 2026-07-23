---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-google-gemini-3-flash-preview
  created: '2026-05-03T14:24:00Z'
spec:
  name: Gemini 3 Flash Preview
  kind: model-spec
  format: markdown
  provider: google
  family: gemini-3-flash-preview
  legacy_model_id: MODEL-google-gemini-3-flash-preview
  profile_ids:
  - gemini-3-flash-preview
  versions:
  - version_id: '1'
    status: ga
    context_window: 1048000
    pricing_usd_per_1m:
      input: 0.5
      output: 3.0
    pricing_note: Google Gemini 3 Flash Preview pricing.
    lifecycle: active
    source_urls: &id001
    - https://ai.google.dev/gemini-api/docs/models
    - https://ai.google.dev/gemini-api/docs/pricing
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 4
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
  equivalent_tier: l
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier fast multimodal preview
  lifecycle: active
  source_urls: *id001
  model_classes:
  - fast
  - standard
  task_suitability:
    architecture: 3
    algorithm_design: 4
    debugging: 3
    implementation: 3
    refactoring: 3
    code_review: 3
    test_generation: 4
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
    math_reasoning: 4
    scientific_reasoning: 4
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
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
    voice: 5
    audio_transcription: 5
    video_understanding: 5
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 2
    self_hosted_enterprise: 2
---
