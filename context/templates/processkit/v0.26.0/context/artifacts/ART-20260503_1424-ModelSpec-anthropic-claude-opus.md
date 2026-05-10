---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-anthropic-claude-opus
  created: '2026-05-03T14:24:00Z'
spec:
  name: Claude Opus 4.7
  kind: model-spec
  format: markdown
  provider: anthropic
  family: claude-opus
  legacy_model_id: MODEL-anthropic-claude-opus
  profile_ids:
  - claude-opus-4.7
  - claude-opus-4.6
  versions:
  - version_id: '4.7'
    status: ga
    context_window: 1000000
    pricing_usd_per_1m:
      input: 5.0
      output: 25.0
    pricing_note: Anthropic Opus 4.7 pricing; 1M context.
    lifecycle: active
    source_urls: &id001
    - https://docs.anthropic.com/en/docs/about-claude/models/overview
    - https://www.anthropic.com/news/claude-opus-4-7
  - version_id: '4.6'
    status: preview
    context_window: 200000
    pricing_usd_per_1m:
      input: 15.0
      output: 75.0
    pricing_note: Most expensive Claude; reserve for tasks where quality justifies
      cost
    lifecycle: unverified
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
    reasoning: 5
    engineering: 5
    speed: 2
    breadth: 5
    reliability: 5
    governance: 5
  modalities:
  - text
  - vision
  - audio
  - tools
  - computer-use
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.anthropic.com/
  rationale: Frontier flagship
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
    legal_analysis: 5
    medical_admin: 5
    financial_analysis: 5
    translation: 5
    multilingual_chat: 5
    classification: 3
    sentiment_analysis: 3
    creative_writing: 4
    marketing_copy: 4
    data_analysis: 5
    sql_generation: 5
    spreadsheet_analysis: 5
    ocr: 5
    image_understanding: 5
    chart_understanding: 5
    diagram_reasoning: 5
    voice: 4
    audio_transcription: 4
    video_understanding: 5
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 5
    self_hosted_enterprise: 4
---
