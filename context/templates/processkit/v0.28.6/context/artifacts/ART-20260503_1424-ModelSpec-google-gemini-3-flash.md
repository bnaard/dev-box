---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-google-gemini-3-flash
  created: '2026-05-03T14:24:00Z'
spec:
  name: Gemini 3 Flash
  kind: model-spec
  format: markdown
  provider: google
  family: gemini-3-flash
  legacy_model_id: MODEL-google-gemini-3-flash
  profile_ids:
  - gemini-3-flash
  versions:
  - version_id: '3'
    status: preview
    context_window: 2000000
    pricing_usd_per_1m:
      input: 0.05
      output: 0.2
    pricing_note: ESTIMATED — next-gen Google Flash. Run Workflow C to validate.
    lifecycle: unverified
    source_urls: &id001
    - https://ai.google.dev/gemini-api/docs/models
    - https://ai.google.dev/gemini-api/docs/pricing
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 4
    engineering: 3
    speed: 5
    breadth: 5
    reliability: 3
    governance: 2
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: l
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier fast (estimated)
  lifecycle: unverified
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 3
    algorithm_design: 3
    debugging: 2
    implementation: 2
    refactoring: 2
    code_review: 2
    test_generation: 3
    repo_navigation: 2
    agentic_workflow: 2
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
    legal_analysis: 2
    medical_admin: 2
    financial_analysis: 2
    translation: 3
    multilingual_chat: 3
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
    privacy_sensitive: 2
    self_hosted_enterprise: 1
---
