---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-openai-gpt-5-3-codex-spark
  created: '2026-05-03T14:24:00Z'
spec:
  name: GPT-5.3-Codex-Spark
  kind: model-spec
  format: markdown
  provider: openai
  family: gpt-5.3-codex-spark
  legacy_model_id: MODEL-openai-gpt-5-3-codex-spark
  profile_ids:
  - gpt-5.3-codex-spark
  versions:
  - version_id: '1'
    status: ga
    context_window: 128000
    pricing_note: Exposed by local Codex catalog as supported_in_api=false; API token
      pricing is not applicable.
    lifecycle: active
    source_urls: &id001
    - https://platform.openai.com/docs/models
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 4
    engineering: 4
    speed: 5
    breadth: 3
    reliability: 4
    governance: 2
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.openai.com/
  rationale: Ultra-fast Codex coding model
  lifecycle: active
  source_urls: *id001
  model_classes:
  - fast
  task_suitability:
    architecture: 3
    algorithm_design: 3
    debugging: 4
    implementation: 4
    refactoring: 3
    code_review: 3
    test_generation: 5
    repo_navigation: 4
    agentic_workflow: 4
    tool_calling: 4
    structured_output: 4
    data_extraction: 3
    summarization: 3
    long_context_synthesis: 2
    rag: 3
    citation_answering: 3
    research_synthesis: 2
    math_reasoning: 3
    scientific_reasoning: 3
    legal_analysis: 2
    medical_admin: 2
    financial_analysis: 2
    translation: 3
    multilingual_chat: 3
    classification: 4
    sentiment_analysis: 4
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 3
    sql_generation: 4
    spreadsheet_analysis: 3
    ocr: 2
    image_understanding: 2
    chart_understanding: 2
    diagram_reasoning: 3
    voice: 1
    audio_transcription: 1
    video_understanding: 1
    low_latency_chat: 5
    bulk_generation: 5
    privacy_sensitive: 2
    self_hosted_enterprise: 1
---
