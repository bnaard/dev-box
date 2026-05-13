---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-deepseek-deepseek-r
  created: '2026-05-03T14:24:00Z'
spec:
  name: DeepSeek R1
  kind: model-spec
  format: markdown
  provider: deepseek
  family: deepseek-r
  legacy_model_id: MODEL-deepseek-deepseek-r
  profile_ids:
  - deepseek-r1
  versions:
  - version_id: '1'
    status: deprecated
    context_window: 64000
    pricing_usd_per_1m:
      input: 0.55
      output: 2.19
    pricing_note: ~20x cheaper than o3 for comparable reasoning; G:1 severely limits
      safe use cases.
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
    reasoning: 5
    engineering: 4
    speed: 3
    breadth: 3
    reliability: 3
    governance: 1
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.deepseek.com/
  rationale: Frontier reasoning, low cost
  lifecycle: deprecated
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 4
    algorithm_design: 4
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 3
    test_generation: 3
    repo_navigation: 4
    agentic_workflow: 4
    tool_calling: 2
    structured_output: 2
    data_extraction: 3
    summarization: 3
    long_context_synthesis: 3
    rag: 3
    citation_answering: 3
    research_synthesis: 3
    math_reasoning: 4
    scientific_reasoning: 4
    legal_analysis: 2
    medical_admin: 2
    financial_analysis: 2
    translation: 2
    multilingual_chat: 2
    classification: 2
    sentiment_analysis: 2
    creative_writing: 2
    marketing_copy: 2
    data_analysis: 4
    sql_generation: 4
    spreadsheet_analysis: 4
    ocr: 2
    image_understanding: 2
    chart_understanding: 2
    diagram_reasoning: 2
    voice: 2
    audio_transcription: 2
    video_understanding: 2
    low_latency_chat: 2
    bulk_generation: 2
    privacy_sensitive: 1
    self_hosted_enterprise: 2
---
