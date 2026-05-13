---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-deepseek-deepseek-v
  created: '2026-05-03T14:24:00Z'
spec:
  name: DeepSeek V3
  kind: model-spec
  format: markdown
  provider: deepseek
  family: deepseek-v
  legacy_model_id: MODEL-deepseek-deepseek-v
  profile_ids:
  - deepseek-v3
  versions:
  - version_id: '3'
    status: deprecated
    context_window: 64000
    pricing_usd_per_1m:
      input: 0.14
      output: 0.28
    pricing_note: Exceptional price-capability ratio; cheapest frontier-class model.
      G:1 severely limits safe use cases.
    governance_warning: G:1 — Chinese jurisdiction. Never use for PII, PHI, confidential
      IP, credentials, regulated data, or government/defense work.
    lifecycle: deprecated
    deprecated_after: '2026-07-24'
    replacement_model: deepseek-v4-flash
    source_urls: &id001
    - https://api-docs.deepseek.com/updates
    - https://api-docs.deepseek.com/quick_start/pricing
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 4
    engineering: 4
    speed: 4
    breadth: 3
    reliability: 3
    governance: 1
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.deepseek.com/
  rationale: Frontier-class, low cost
  lifecycle: deprecated
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 3
    algorithm_design: 4
    debugging: 3
    implementation: 3
    refactoring: 3
    code_review: 3
    test_generation: 3
    repo_navigation: 3
    agentic_workflow: 3
    tool_calling: 3
    structured_output: 3
    data_extraction: 3
    summarization: 3
    long_context_synthesis: 3
    rag: 3
    citation_answering: 3
    research_synthesis: 3
    math_reasoning: 4
    scientific_reasoning: 3
    legal_analysis: 2
    medical_admin: 2
    financial_analysis: 2
    translation: 2
    multilingual_chat: 2
    classification: 4
    sentiment_analysis: 4
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 3
    sql_generation: 3
    spreadsheet_analysis: 3
    ocr: 2
    image_understanding: 2
    chart_understanding: 2
    diagram_reasoning: 2
    voice: 2
    audio_transcription: 2
    video_understanding: 2
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 1
    self_hosted_enterprise: 2
---
