---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-openai-gpt
  created: '2026-05-03T14:24:00Z'
spec:
  name: GPT-5.5
  kind: model-spec
  format: markdown
  provider: openai
  family: gpt
  legacy_model_id: MODEL-openai-gpt
  profile_ids:
  - gpt-5.5
  - gpt-5.2
  versions:
  - version_id: '5.5'
    status: ga
    context_window: 1000000
    pricing_usd_per_1m:
      input: 5.0
      output: 30.0
    pricing_note: 'OpenAI announcement: API soon/available at standard pricing; Batch/Flex
      half-rate; Priority 2.5x.'
    lifecycle: active
    source_urls: &id001
    - https://platform.openai.com/docs/models
    - https://openai.com/index/introducing-gpt-5-5/
  - version_id: '5.2'
    status: ga
    context_window: 400000
    pricing_usd_per_1m:
      input: 1.75
      output: 14.0
    pricing_note: OpenAI GPT-5.2 standard pricing.
    lifecycle: active
    source_urls:
    - https://platform.openai.com/docs/models
    - https://openai.com/index/introducing-gpt-5-5/
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
  status_page_url: https://status.openai.com/
  rationale: Frontier flagship (estimated, 2026-04-25)
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
    voice: 4
    audio_transcription: 4
    video_understanding: 5
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 3
    self_hosted_enterprise: 3
---
