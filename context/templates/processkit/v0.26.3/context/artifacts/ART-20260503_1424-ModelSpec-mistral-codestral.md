---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-mistral-codestral
  created: '2026-05-03T14:24:00Z'
spec:
  name: Codestral
  kind: model-spec
  format: markdown
  provider: mistral
  family: codestral
  legacy_model_id: MODEL-mistral-codestral
  profile_ids:
  - codestral
  versions:
  - version_id: '1'
    status: ga
    context_window: 256000
    pricing_usd_per_1m:
      input: 0.3
      output: 0.9
    pricing_note: Mistral's code-focused model; weights available for self-hosting;
      strong fill-in-the-middle
    lifecycle: legacy
    source_urls: &id001
    - https://docs.mistral.ai/models/overview
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 3
    engineering: 5
    speed: 4
    breadth: 3
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.mistral.ai/
  rationale: Coding specialist
  lifecycle: legacy
  source_urls: *id001
  model_classes: []
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
    tool_calling: 3
    structured_output: 3
    data_extraction: 3
    summarization: 3
    long_context_synthesis: 3
    rag: 3
    citation_answering: 3
    research_synthesis: 3
    math_reasoning: 3
    scientific_reasoning: 2
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
    ocr: 2
    image_understanding: 2
    chart_understanding: 2
    diagram_reasoning: 2
    voice: 2
    audio_transcription: 2
    video_understanding: 2
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 4
    self_hosted_enterprise: 4
---
