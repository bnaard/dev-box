---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-cohere-command-r-plus
  created: '2026-05-03T14:24:00Z'
spec:
  name: Command R+
  kind: model-spec
  format: markdown
  provider: cohere
  family: command-r-plus
  legacy_model_id: MODEL-cohere-command-r-plus
  profile_ids:
  - command-r-plus
  versions:
  - version_id: '1'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 2.5
      output: 10.0
    pricing_note: Weights available for self-hosting; Cohere is SOC 2 certified; HIPAA
      BAA available for enterprise
    lifecycle: legacy
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
    breadth: 3
    reliability: 4
    governance: 3
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.cohere.com/
  rationale: Enterprise RAG specialist
  lifecycle: legacy
  source_urls: *id001
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
    tool_calling: 4
    structured_output: 4
    data_extraction: 3
    summarization: 3
    long_context_synthesis: 3
    rag: 4
    citation_answering: 4
    research_synthesis: 3
    math_reasoning: 3
    scientific_reasoning: 3
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 4
    multilingual_chat: 4
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
    voice: 3
    audio_transcription: 3
    video_understanding: 2
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 3
    self_hosted_enterprise: 3
---
