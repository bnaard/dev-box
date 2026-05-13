---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-google-gemma-3-27b
  created: '2026-05-03T14:24:00Z'
spec:
  name: Gemma 3 27B
  kind: model-spec
  format: markdown
  provider: google
  family: gemma-3-27b
  legacy_model_id: MODEL-google-gemma-3-27b
  profile_ids:
  - gemma-3-27b
  versions:
  - version_id: '3'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 0.15
      output: 0.45
    pricing_note: Open weights under Gemma Terms; self-hosting recommended for proprietary
      workloads
    governance_warning: 'Via Google API: G:2 (same as Gemini). Self-hosted open weights
      (Gemma Terms of Use): G:4 — data stays local, strong sovereignty, but no Anthropic-tier
      compliance certs.'
    lifecycle: legacy
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 3
    engineering: 3
    speed: 4
    breadth: 4
    reliability: 3
    governance: 4
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.cloud.google.com/
  rationale: Open-source frontier-class
  lifecycle: legacy
  source_urls: []
  model_classes: []
  task_suitability:
    architecture: 3
    algorithm_design: 3
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
    math_reasoning: 3
    scientific_reasoning: 3
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 3
    multilingual_chat: 3
    classification: 4
    sentiment_analysis: 4
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 3
    sql_generation: 3
    spreadsheet_analysis: 3
    ocr: 3
    image_understanding: 3
    chart_understanding: 3
    diagram_reasoning: 3
    voice: 2
    audio_transcription: 2
    video_understanding: 2
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 4
    self_hosted_enterprise: 4
---
