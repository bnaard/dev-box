---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-anthropic-claude-haiku
  created: '2026-05-03T14:24:00Z'
spec:
  name: Claude Haiku 4.5
  kind: model-spec
  format: markdown
  provider: anthropic
  family: claude-haiku
  legacy_model_id: MODEL-anthropic-claude-haiku
  profile_ids:
  - claude-haiku-4.5
  versions:
  - version_id: '4.5'
    status: ga
    context_window: 200000
    pricing_usd_per_1m:
      input: 0.25
      output: 1.25
    pricing_note: Cheapest Claude; excellent value for high-volume, low-complexity
      tasks
    lifecycle: active
    source_urls: &id001
    - https://docs.anthropic.com/en/docs/about-claude/models/overview
    - https://www.anthropic.com/news/claude-opus-4-7
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 3
    engineering: 3
    speed: 5
    breadth: 3
    reliability: 4
    governance: 5
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.anthropic.com/
  rationale: Frontier fast
  lifecycle: active
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 3
    algorithm_design: 3
    debugging: 3
    implementation: 3
    refactoring: 3
    code_review: 4
    test_generation: 4
    repo_navigation: 3
    agentic_workflow: 3
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
    translation: 4
    multilingual_chat: 4
    classification: 5
    sentiment_analysis: 5
    creative_writing: 4
    marketing_copy: 4
    data_analysis: 3
    sql_generation: 3
    spreadsheet_analysis: 3
    ocr: 3
    image_understanding: 3
    chart_understanding: 3
    diagram_reasoning: 3
    voice: 3
    audio_transcription: 3
    video_understanding: 2
    low_latency_chat: 5
    bulk_generation: 5
    privacy_sensitive: 5
    self_hosted_enterprise: 3
---
