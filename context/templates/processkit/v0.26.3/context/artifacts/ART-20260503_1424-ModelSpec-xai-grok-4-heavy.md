---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-xai-grok-4-heavy
  created: '2026-05-03T14:24:00Z'
spec:
  name: Grok 4 Heavy
  kind: model-spec
  format: markdown
  provider: xai
  family: grok-4-heavy
  legacy_model_id: MODEL-xai-grok-4-heavy
  profile_ids:
  - grok-4-heavy
  versions:
  - version_id: '4'
    status: preview
    context_window: 256000
    pricing_usd_per_1m:
      input: 10.0
      output: 40.0
    pricing_note: ESTIMATED — compute-heavy reasoning variant analogous to o3. Run
      Workflow C to validate.
    lifecycle: legacy
    source_urls: &id001
    - https://x.ai/api
    - https://docs.x.ai/docs/models
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 5
    engineering: 5
    speed: 1
    breadth: 4
    reliability: 5
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.x.ai/
  rationale: Frontier reasoning flagship (estimated)
  lifecycle: legacy
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 5
    algorithm_design: 5
    debugging: 5
    implementation: 5
    refactoring: 5
    code_review: 5
    test_generation: 3
    repo_navigation: 5
    agentic_workflow: 5
    tool_calling: 4
    structured_output: 4
    data_extraction: 4
    summarization: 4
    long_context_synthesis: 4
    rag: 4
    citation_answering: 4
    research_synthesis: 4
    math_reasoning: 5
    scientific_reasoning: 4
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
    translation: 4
    multilingual_chat: 4
    classification: 2
    sentiment_analysis: 2
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 5
    sql_generation: 5
    spreadsheet_analysis: 5
    ocr: 4
    image_understanding: 4
    chart_understanding: 4
    diagram_reasoning: 4
    voice: 3
    audio_transcription: 3
    video_understanding: 3
    low_latency_chat: 2
    bulk_generation: 2
    privacy_sensitive: 3
    self_hosted_enterprise: 2
---
