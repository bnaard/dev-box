---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-minimax-minimax-m
  created: '2026-05-03T14:24:00Z'
spec:
  name: MiniMax M2.5
  kind: model-spec
  format: markdown
  provider: minimax
  family: minimax-m
  legacy_model_id: MODEL-minimax-minimax-m
  profile_ids:
  - minimax-m2.5
  versions:
  - version_id: '2.5'
    status: preview
    context_window: 1000000
    pricing_usd_per_1m:
      input: 0.2
      output: 1.1
    pricing_note: ESTIMATED — updated MiniMax. Run Workflow C to validate.
    governance_warning: 'Chinese company. Via MiniMax API: G:1. Self-hosted open weights
      raise governance to G:3.'
    lifecycle: unverified
    source_urls: &id001
    - https://platform.minimax.io/docs/api-reference/api-overview
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 4
    engineering: 3
    speed: 3
    breadth: 5
    reliability: 3
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: l
  status_page_url: https://www.minimax.io/
  rationale: Long-context (estimated)
  lifecycle: unverified
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 3
    algorithm_design: 3
    debugging: 2
    implementation: 2
    refactoring: 2
    code_review: 2
    test_generation: 2
    repo_navigation: 2
    agentic_workflow: 2
    tool_calling: 2
    structured_output: 2
    data_extraction: 3
    summarization: 3
    long_context_synthesis: 3
    rag: 3
    citation_answering: 3
    research_synthesis: 3
    math_reasoning: 3
    scientific_reasoning: 3
    legal_analysis: 2
    medical_admin: 2
    financial_analysis: 2
    translation: 3
    multilingual_chat: 3
    classification: 3
    sentiment_analysis: 3
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 3
    sql_generation: 3
    spreadsheet_analysis: 3
    ocr: 3
    image_understanding: 3
    chart_understanding: 3
    diagram_reasoning: 3
    voice: 2
    audio_transcription: 2
    video_understanding: 3
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 2
    self_hosted_enterprise: 3
---
