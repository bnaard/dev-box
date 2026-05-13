---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-openai-o4-mini
  created: '2026-05-03T14:24:00Z'
spec:
  name: o4-mini
  kind: model-spec
  format: markdown
  provider: openai
  family: o4-mini
  legacy_model_id: MODEL-openai-o4-mini
  profile_ids:
  - o4-mini
  versions:
  - version_id: '1'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 1.1
      output: 4.4
    pricing_note: Best cost-efficiency among reasoning models; ~10x cheaper than o3
    lifecycle: legacy
    source_urls: &id001
    - https://platform.openai.com/docs/models
    - https://openai.com/index/introducing-gpt-5-5/
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 4
    engineering: 4
    speed: 3
    breadth: 3
    reliability: 4
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.openai.com/
  rationale: Frontier reasoning mid-size
  lifecycle: legacy
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
    math_reasoning: 4
    scientific_reasoning: 3
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
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
    privacy_sensitive: 2
    self_hosted_enterprise: 2
---
