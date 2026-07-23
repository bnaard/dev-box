---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-xai-grok-4-1
  created: '2026-05-03T14:24:00Z'
spec:
  name: Grok 4.1
  kind: model-spec
  format: markdown
  provider: xai
  family: grok-4-1
  legacy_model_id: MODEL-xai-grok-4-1
  profile_ids:
  - grok-4.1
  versions:
  - version_id: '4.1'
    status: preview
    context_window: 256000
    pricing_usd_per_1m:
      input: 5.0
      output: 20.0
    pricing_note: ESTIMATED — minor update to Grok 4. Run Workflow C to validate.
    lifecycle: unverified
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
  rationale: Frontier flagship (estimated)
  lifecycle: unverified
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 4
    algorithm_design: 4
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 4
    test_generation: 4
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
    math_reasoning: 4
    scientific_reasoning: 4
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 4
    multilingual_chat: 4
    classification: 3
    sentiment_analysis: 3
    creative_writing: 4
    marketing_copy: 4
    data_analysis: 4
    sql_generation: 4
    spreadsheet_analysis: 4
    ocr: 4
    image_understanding: 4
    chart_understanding: 4
    diagram_reasoning: 4
    voice: 3
    audio_transcription: 3
    video_understanding: 4
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 2
    self_hosted_enterprise: 2
---
