---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-moonshot-kimi-k2-6
  created: '2026-05-03T14:24:00Z'
spec:
  name: Kimi K2.6
  kind: model-spec
  format: markdown
  provider: moonshot
  family: kimi-k2.6
  legacy_model_id: MODEL-moonshot-kimi-k2-6
  profile_ids:
  - kimi-k2.6
  versions:
  - version_id: '1'
    status: ga
    context_window: 256000
    pricing_usd_per_1m:
      input: 0.95
      output: 4.0
    pricing_note: Moonshot Kimi K2.6 standard API pricing; cache-hit input may be
      cheaper.
    governance_warning: 'Chinese company. Via Moonshot API: G:1. Self-hosted open
      weights (Modified MIT, Hugging Face) raise governance to G:3–4.'
    lifecycle: active
    source_urls: &id001
    - https://platform.moonshot.ai/
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
    reliability: 4
    governance: 1
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://platform.moonshot.ai/
  rationale: Frontier open-weights (estimated, 2026-04-25)
  lifecycle: active
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
    tool_calling: 4
    structured_output: 4
    data_extraction: 5
    summarization: 5
    long_context_synthesis: 5
    rag: 5
    citation_answering: 5
    research_synthesis: 5
    math_reasoning: 5
    scientific_reasoning: 5
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
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
    voice: 3
    audio_transcription: 3
    video_understanding: 4
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 2
    self_hosted_enterprise: 3
---
