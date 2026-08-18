---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-deepseek-deepseek-v4-pro
  created: '2026-05-03T14:24:00Z'
spec:
  name: DeepSeek V4 Pro
  kind: model-spec
  format: markdown
  provider: deepseek
  family: deepseek-v4-pro
  legacy_model_id: MODEL-deepseek-deepseek-v4-pro
  profile_ids:
  - deepseek-v4-pro
  versions:
  - version_id: '1'
    status: ga
    context_window: 1000000
    pricing_usd_per_1m:
      input: 1.74
      output: 3.48
    pricing_note: DeepSeek V4 Pro pricing.
    governance_warning: G:1 — Chinese jurisdiction. Never use for PII, PHI, confidential
      IP, credentials, regulated data, or government/defense work.
    lifecycle: active
    source_urls: &id001
    - https://api-docs.deepseek.com/updates
    - https://api-docs.deepseek.com/quick_start/pricing
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 5
    engineering: 4
    speed: 3
    breadth: 5
    reliability: 3
    governance: 1
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.deepseek.com/
  rationale: Low-cost reasoning flagship
  lifecycle: active
  source_urls: *id001
  model_classes:
  - powerful
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
    scientific_reasoning: 5
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
    video_understanding: 5
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 2
    self_hosted_enterprise: 2
---
