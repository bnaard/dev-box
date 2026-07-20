---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-mistral-mistral-small
  created: '2026-05-03T14:24:00Z'
spec:
  name: Mistral Small 4
  kind: model-spec
  format: markdown
  provider: mistral
  family: mistral-small
  legacy_model_id: MODEL-mistral-mistral-small
  profile_ids:
  - mistral-small-4
  - mistral-small-3
  versions:
  - version_id: '4'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 0.1
      output: 0.3
    pricing_note: Mistral Small-family low-cost pricing; verify current Small 4 rate.
    lifecycle: active
    source_urls: &id001
    - https://docs.mistral.ai/models/overview
  - version_id: '3'
    status: ga
    context_window: 32000
    pricing_usd_per_1m:
      input: 0.1
      output: 0.3
    pricing_note: Mistral's entry-level model; very cheap, EU-based; good for classification
      and routing
    lifecycle: legacy
    source_urls:
    - https://docs.mistral.ai/models/overview
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 2
    engineering: 3
    speed: 5
    breadth: 2
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: s
  status_page_url: https://status.mistral.ai/
  rationale: EU fast small
  lifecycle: active
  source_urls: *id001
  model_classes:
  - fast
  task_suitability:
    architecture: 3
    algorithm_design: 2
    debugging: 3
    implementation: 3
    refactoring: 3
    code_review: 3
    test_generation: 4
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
    translation: 4
    multilingual_chat: 4
    classification: 5
    sentiment_analysis: 5
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 3
    sql_generation: 3
    spreadsheet_analysis: 3
    ocr: 2
    image_understanding: 2
    chart_understanding: 2
    diagram_reasoning: 2
    voice: 3
    audio_transcription: 3
    video_understanding: 2
    low_latency_chat: 5
    bulk_generation: 5
    privacy_sensitive: 4
    self_hosted_enterprise: 4
---
