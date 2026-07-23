---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-mistral-mistral-large
  created: '2026-05-03T14:24:00Z'
spec:
  name: Mistral Large 3
  kind: model-spec
  format: markdown
  provider: mistral
  family: mistral-large
  legacy_model_id: MODEL-mistral-mistral-large
  profile_ids:
  - mistral-large-3
  versions:
  - version_id: '3'
    status: preview
    context_window: 128000
    pricing_usd_per_1m:
      input: 2.0
      output: 6.0
    pricing_note: EU-based pricing in EUR also available; Le Chat Enterprise has volume
      discounts
    lifecycle: unverified
    source_urls: &id001
    - https://docs.mistral.ai/models/overview
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 3
    engineering: 4
    speed: 4
    breadth: 3
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: l
  status_page_url: https://status.mistral.ai/
  rationale: Mid-tier frontier
  lifecycle: unverified
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 3
    algorithm_design: 3
    debugging: 3
    implementation: 3
    refactoring: 3
    code_review: 2
    test_generation: 3
    repo_navigation: 3
    agentic_workflow: 3
    tool_calling: 3
    structured_output: 3
    data_extraction: 2
    summarization: 2
    long_context_synthesis: 2
    rag: 2
    citation_answering: 2
    research_synthesis: 2
    math_reasoning: 2
    scientific_reasoning: 2
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 3
    multilingual_chat: 3
    classification: 3
    sentiment_analysis: 3
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
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 3
    self_hosted_enterprise: 3
---
