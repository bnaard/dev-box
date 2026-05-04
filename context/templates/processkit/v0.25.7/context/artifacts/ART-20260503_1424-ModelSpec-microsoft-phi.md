---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-microsoft-phi
  created: '2026-05-03T14:24:00Z'
spec:
  name: Phi-4
  kind: model-spec
  format: markdown
  provider: microsoft
  family: phi
  legacy_model_id: MODEL-microsoft-phi
  profile_ids:
  - phi-4
  versions:
  - version_id: '4'
    status: ga
    context_window: 16000
    pricing_usd_per_1m:
      input: 0.07
      output: 0.14
    pricing_note: MIT license; cheapest STEM-capable model; via Azure AI Foundry with
      enterprise DPA available
    lifecycle: legacy
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 4
    engineering: 3
    speed: 5
    breadth: 2
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: l
  status_page_url: https://azure.status.microsoft/
  rationale: Small / efficient
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
    math_reasoning: 5
    scientific_reasoning: 3
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 2
    multilingual_chat: 2
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
