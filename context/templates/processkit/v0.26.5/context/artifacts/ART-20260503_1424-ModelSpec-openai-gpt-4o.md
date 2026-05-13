---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-openai-gpt-4o
  created: '2026-05-03T14:24:00Z'
spec:
  name: GPT-4o
  kind: model-spec
  format: markdown
  provider: openai
  family: gpt-4o
  legacy_model_id: MODEL-openai-gpt-4o
  profile_ids:
  - gpt-4o
  versions:
  - version_id: '1'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 2.5
      output: 10.0
    pricing_note: Strong value for multimodal tasks; audio in/out included in standard
      pricing
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
  dimensions:
    reasoning: 4
    engineering: 4
    speed: 3
    breadth: 5
    reliability: 4
    governance: 2
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.openai.com/
  rationale: Frontier multi-modal
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
    voice: 4
    audio_transcription: 4
    video_understanding: 4
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 2
    self_hosted_enterprise: 2
---
