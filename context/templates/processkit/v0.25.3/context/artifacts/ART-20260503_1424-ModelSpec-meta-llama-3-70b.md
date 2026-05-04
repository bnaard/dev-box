---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-meta-llama-3-70b
  created: '2026-05-03T14:24:00Z'
spec:
  name: Llama 3.3 70B
  kind: model-spec
  format: markdown
  provider: meta
  family: llama-3-70b
  legacy_model_id: MODEL-meta-llama-3-70b
  profile_ids:
  - llama-3.3-70b
  versions:
  - version_id: '3.3'
    status: ga
    context_window: 128000
    pricing_note: 'No API cost; infra cost varies by hardware. Via third-party APIs
      (Groq, Together, etc.): ~$0.06-0.80/1M tokens depending on provider.'
    lifecycle: legacy
    source_urls: &id001
    - https://llama.meta.com/
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
    reliability: 3
    governance: 5
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://ai.meta.com/
  rationale: Open-source
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
    privacy_sensitive: 4
    self_hosted_enterprise: 4
---
