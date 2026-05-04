---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-openai-o3
  created: '2026-05-03T14:24:00Z'
spec:
  name: o3
  kind: model-spec
  format: markdown
  provider: openai
  family: o3
  legacy_model_id: MODEL-openai-o3
  profile_ids:
  - o3
  versions:
  - version_id: '1'
    status: ga
    context_window: 200000
    pricing_usd_per_1m:
      input: 2.0
      output: 8.0
    pricing_note: Legacy reasoning model; lower price than original roster entry.
    lifecycle: legacy
    source_urls: &id001
    - https://platform.openai.com/docs/models
    - https://openai.com/index/introducing-gpt-5-5/
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
    speed: 1
    breadth: 3
    reliability: 4
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.openai.com/
  rationale: Frontier reasoning flagship
  lifecycle: legacy
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 4
    algorithm_design: 5
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 4
    test_generation: 3
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
    math_reasoning: 5
    scientific_reasoning: 4
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 4
    multilingual_chat: 4
    classification: 2
    sentiment_analysis: 2
    creative_writing: 2
    marketing_copy: 2
    data_analysis: 4
    sql_generation: 4
    spreadsheet_analysis: 4
    ocr: 3
    image_understanding: 3
    chart_understanding: 3
    diagram_reasoning: 3
    voice: 2
    audio_transcription: 2
    video_understanding: 3
    low_latency_chat: 2
    bulk_generation: 2
    privacy_sensitive: 2
    self_hosted_enterprise: 2
---
