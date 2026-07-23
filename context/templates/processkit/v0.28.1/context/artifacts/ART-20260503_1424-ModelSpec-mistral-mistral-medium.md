---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-mistral-mistral-medium
  created: '2026-05-03T14:24:00Z'
spec:
  name: Mistral Medium 3.5
  kind: model-spec
  format: markdown
  provider: mistral
  family: mistral-medium
  legacy_model_id: MODEL-mistral-mistral-medium
  profile_ids:
  - mistral-medium-3.5
  - mistral-medium-3
  versions:
  - version_id: '3.5'
    status: ga
    context_window: 256000
    pricing_usd_per_1m:
      input: 1.5
      output: 7.5
    pricing_note: Mistral Medium 3.5 pricing.
    lifecycle: active
    source_urls: &id001
    - https://docs.mistral.ai/models/overview
  - version_id: '3'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 0.4
      output: 2.0
    pricing_note: Between Small and Large; good balance of speed and capability
    lifecycle: legacy
    source_urls:
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
    breadth: 4
    reliability: 3
    governance: 4
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: l
  status_page_url: https://status.mistral.ai/
  rationale: EU frontier mid-size
  lifecycle: active
  source_urls: *id001
  model_classes:
  - standard
  task_suitability:
    architecture: 3
    algorithm_design: 3
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 3
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
    math_reasoning: 3
    scientific_reasoning: 4
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 4
    multilingual_chat: 4
    classification: 4
    sentiment_analysis: 4
    creative_writing: 4
    marketing_copy: 4
    data_analysis: 3
    sql_generation: 3
    spreadsheet_analysis: 3
    ocr: 4
    image_understanding: 4
    chart_understanding: 4
    diagram_reasoning: 4
    voice: 4
    audio_transcription: 4
    video_understanding: 4
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 4
    self_hosted_enterprise: 4
---
