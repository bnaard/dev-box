---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-aws-amazon-nova-2-lite
  created: '2026-05-03T14:24:00Z'
spec:
  name: Amazon Nova 2 Lite
  kind: model-spec
  format: markdown
  provider: aws
  family: amazon-nova-2-lite
  legacy_model_id: MODEL-aws-amazon-nova-2-lite
  profile_ids:
  - amazon-nova-2-lite
  versions:
  - version_id: '1'
    status: ga
    context_window: 300000
    pricing_note: Amazon Bedrock pricing varies by region/model; verify before budgeting.
    lifecycle: active
    source_urls: &id001
    - https://aws.amazon.com/bedrock/amazon-nova/
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 3
    engineering: 3
    speed: 5
    breadth: 5
    reliability: 3
    governance: 4
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://health.aws.amazon.com/health/status
  rationale: Bedrock fast multimodal
  lifecycle: active
  source_urls: *id001
  model_classes:
  - fast
  task_suitability:
    architecture: 3
    algorithm_design: 3
    debugging: 3
    implementation: 3
    refactoring: 3
    code_review: 3
    test_generation: 4
    repo_navigation: 3
    agentic_workflow: 3
    tool_calling: 4
    structured_output: 4
    data_extraction: 4
    summarization: 4
    long_context_synthesis: 4
    rag: 4
    citation_answering: 4
    research_synthesis: 4
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
    voice: 5
    audio_transcription: 5
    video_understanding: 4
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 4
    self_hosted_enterprise: 4
---
