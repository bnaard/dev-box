---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-xai-grok-4
  created: '2026-05-03T14:24:00Z'
spec:
  name: Grok 4
  kind: model-spec
  format: markdown
  provider: xai
  family: grok-4
  legacy_model_id: MODEL-xai-grok-4
  profile_ids:
  - grok-4
  versions:
  - version_id: '4'
    status: preview
    context_window: 256000
    pricing_usd_per_1m:
      input: 5.0
      output: 20.0
    pricing_note: ESTIMATED — major xAI release. Run Workflow C to validate.
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
    engineering: 5
    speed: 3
    breadth: 4
    reliability: 5
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.x.ai/
  rationale: Frontier flagship (estimated)
  lifecycle: legacy
  source_urls: *id001
  model_classes: []
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
    data_extraction: 4
    summarization: 4
    long_context_synthesis: 4
    rag: 4
    citation_answering: 4
    research_synthesis: 4
    math_reasoning: 5
    scientific_reasoning: 4
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
    translation: 4
    multilingual_chat: 4
    classification: 3
    sentiment_analysis: 3
    creative_writing: 4
    marketing_copy: 4
    data_analysis: 5
    sql_generation: 5
    spreadsheet_analysis: 5
    ocr: 4
    image_understanding: 4
    chart_understanding: 4
    diagram_reasoning: 4
    voice: 4
    audio_transcription: 4
    video_understanding: 3
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 3
    self_hosted_enterprise: 2
---
