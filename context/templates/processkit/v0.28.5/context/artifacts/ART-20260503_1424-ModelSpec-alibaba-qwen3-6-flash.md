---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-alibaba-qwen3-6-flash
  created: '2026-05-03T14:24:00Z'
spec:
  name: Qwen 3.6 Flash
  kind: model-spec
  format: markdown
  provider: alibaba
  family: qwen3.6-flash
  legacy_model_id: MODEL-alibaba-qwen3-6-flash
  profile_ids:
  - qwen3.6-flash
  versions:
  - version_id: '1'
    status: preview
    context_window: 1000000
    pricing_note: Official Model Studio catalog confirms availability; regional pricing
      varies.
    governance_warning: 'Via Alibaba Cloud API: G:1. Self-hosted (Apache 2.0): G:4.
      Recommended deployment is self-hosted for any proprietary code.'
    lifecycle: unverified
    source_urls: &id001
    - https://help.aliyun.com/zh/model-studio/models
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 4
    engineering: 5
    speed: 4
    breadth: 3
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.aliyun.com/
  rationale: Current Alibaba fast model
  lifecycle: unverified
  source_urls: *id001
  model_classes:
  - standard
  task_suitability:
    architecture: 4
    algorithm_design: 4
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 4
    test_generation: 4
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
    scientific_reasoning: 4
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
    translation: 4
    multilingual_chat: 4
    classification: 4
    sentiment_analysis: 4
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 4
    sql_generation: 4
    spreadsheet_analysis: 4
    ocr: 2
    image_understanding: 2
    chart_understanding: 2
    diagram_reasoning: 2
    voice: 3
    audio_transcription: 3
    video_understanding: 3
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 4
    self_hosted_enterprise: 5
---
