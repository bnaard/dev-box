---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-openai-gpt-5-2-pro
  created: '2026-05-03T14:24:00Z'
spec:
  name: GPT-5.2 Pro
  kind: model-spec
  format: markdown
  provider: openai
  family: gpt-5.2-pro
  legacy_model_id: MODEL-openai-gpt-5-2-pro
  profile_ids:
  - gpt-5.2-pro
  versions:
  - version_id: '1'
    status: ga
    context_window: 400000
    pricing_usd_per_1m:
      input: 21.0
      output: 168.0
    pricing_note: OpenAI GPT-5.2 Pro pricing.
    lifecycle: active
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
    reasoning: 5
    engineering: 5
    speed: 2
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
  rationale: Frontier premium
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
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
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
    privacy_sensitive: 3
    self_hosted_enterprise: 3
---
