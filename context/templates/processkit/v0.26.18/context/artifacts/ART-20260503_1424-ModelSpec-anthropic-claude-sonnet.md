---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-anthropic-claude-sonnet
  created: '2026-05-03T14:24:00Z'
spec:
  name: Claude Sonnet 4.6
  kind: model-spec
  format: markdown
  provider: anthropic
  family: claude-sonnet
  legacy_model_id: MODEL-anthropic-claude-sonnet
  profile_ids:
  - claude-sonnet-4.6
  - claude-sonnet-4.5
  versions:
  - version_id: '4.6'
    status: preview
    context_window: 200000
    pricing_usd_per_1m:
      input: 3.0
      output: 15.0
    pricing_note: Best price-capability ratio in the Claude family for engineering
      tasks
    lifecycle: unverified
    source_urls: &id001
    - https://docs.anthropic.com/en/docs/about-claude/models/overview
    - https://www.anthropic.com/news/claude-opus-4-7
  - version_id: '4.5'
    status: ga
    context_window: 200000
    pricing_usd_per_1m:
      input: 3.0
      output: 15.0
    pricing_note: Anthropic Sonnet 4.5 pricing.
    lifecycle: active
    source_urls:
    - https://docs.anthropic.com/en/docs/about-claude/models/overview
    - https://www.anthropic.com/news/claude-opus-4-7
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 4
    engineering: 5
    speed: 3
    breadth: 4
    reliability: 4
    governance: 5
  modalities:
  - text
  - vision
  - tools
  - computer-use
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.anthropic.com/
  rationale: Frontier mid-size
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
    test_generation: 3
    repo_navigation: 4
    agentic_workflow: 4
    tool_calling: 4
    structured_output: 4
    data_extraction: 3
    summarization: 3
    long_context_synthesis: 3
    rag: 3
    citation_answering: 3
    research_synthesis: 3
    math_reasoning: 3
    scientific_reasoning: 3
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
    translation: 3
    multilingual_chat: 3
    classification: 3
    sentiment_analysis: 3
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 4
    sql_generation: 4
    spreadsheet_analysis: 4
    ocr: 3
    image_understanding: 3
    chart_understanding: 3
    diagram_reasoning: 3
    voice: 2
    audio_transcription: 2
    video_understanding: 2
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 4
    self_hosted_enterprise: 3
---
