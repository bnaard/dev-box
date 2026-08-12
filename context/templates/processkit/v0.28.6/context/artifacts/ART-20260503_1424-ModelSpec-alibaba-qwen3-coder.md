---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-alibaba-qwen3-coder
  created: '2026-05-03T14:24:00Z'
spec:
  name: Qwen3 Coder
  kind: model-spec
  format: markdown
  provider: alibaba
  family: qwen3-coder
  legacy_model_id: MODEL-alibaba-qwen3-coder
  profile_ids:
  - qwen3-coder
  versions:
  - version_id: '1'
    status: ga
    context_window: 256000
    pricing_usd_per_1m:
      input: 0.2
      output: 0.6
    pricing_note: Open-weight/third-party estimate; verify hosting provider.
    governance_warning: 'Via Alibaba Cloud API: G:1. Self-hosted (Apache 2.0): G:4.
      Recommended deployment is self-hosted for any proprietary code.'
    lifecycle: active
    source_urls: &id001
    - https://qwenlm.github.io/
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
  rationale: Open-weight coding specialist
  lifecycle: active
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
