---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-cohere-command-a
  created: '2026-05-03T14:24:00Z'
spec:
  name: Command A
  kind: model-spec
  format: markdown
  provider: cohere
  family: command-a
  legacy_model_id: MODEL-cohere-command-a
  profile_ids:
  - command-a
  versions:
  - version_id: '1'
    status: ga
    context_window: 256000
    pricing_usd_per_1m:
      input: 2.5
      output: 10.0
    pricing_note: Cohere Command A pricing; verify current plan.
    lifecycle: active
    source_urls: &id001
    - https://docs.cohere.com/v2/docs/models
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
    reliability: 5
    governance: 3
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.cohere.com/
  rationale: Enterprise RAG specialist
  lifecycle: active
  source_urls: *id001
  model_classes:
  - standard
  task_suitability:
    architecture: 4
    algorithm_design: 3
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 4
    test_generation: 4
    repo_navigation: 4
    agentic_workflow: 4
    tool_calling: 4
    structured_output: 4
    data_extraction: 4
    summarization: 4
    long_context_synthesis: 4
    rag: 5
    citation_answering: 5
    research_synthesis: 4
    math_reasoning: 3
    scientific_reasoning: 4
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
    translation: 4
    multilingual_chat: 4
    classification: 4
    sentiment_analysis: 4
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
    video_understanding: 4
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 4
    self_hosted_enterprise: 3
---
