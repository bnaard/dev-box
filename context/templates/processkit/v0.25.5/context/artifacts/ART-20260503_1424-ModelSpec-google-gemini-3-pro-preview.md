---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-google-gemini-3-pro-preview
  created: '2026-05-03T14:24:00Z'
spec:
  name: Gemini 3 Pro Preview
  kind: model-spec
  format: markdown
  provider: google
  family: gemini-3-pro-preview
  legacy_model_id: MODEL-google-gemini-3-pro-preview
  profile_ids:
  - gemini-3-pro-preview
  versions:
  - version_id: '1'
    status: ga
    context_window: 1048000
    pricing_usd_per_1m:
      input: 2.0
      output: 12.0
    pricing_note: Google pricing up to 200K input; higher tier above 200K is $4/$18.
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
    reasoning: 5
    engineering: 5
    speed: 3
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
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier multimodal preview
  lifecycle: active
  source_urls: *id001
  model_classes:
  - powerful
  task_suitability:
    architecture: 5
    algorithm_design: 5
    debugging: 5
    implementation: 5
    refactoring: 5
    code_review: 5
    test_generation: 4
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
    creative_writing: 4
    marketing_copy: 4
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
