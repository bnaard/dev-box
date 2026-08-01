---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-xiaomi-mimo-7b
  created: '2026-05-03T14:24:00Z'
spec:
  name: MiMo-7B-RL-0530
  kind: model-spec
  format: markdown
  provider: xiaomi
  family: mimo-7b
  legacy_model_id: MODEL-xiaomi-mimo-7b
  profile_ids:
  - mimo-7b
  versions:
  - version_id: '1'
    status: ga
    context_window: 48000
    pricing_note: MIT-licensed open weights; no official hosted API pricing was located.
      Self-hosting infrastructure cost applies.
    governance_warning: 'Self-hosted open weights: G:4. Xiaomi-hosted or third-party
      hosted deployments require separate data-retention and jurisdiction review before
      sensitive use.'
    lifecycle: active
    source_urls: &id001
    - https://mimo.xiaomi.com/
    - https://huggingface.co/XiaomiMiMo/MiMo-7B-RL-0530
    - https://huggingface.co/XiaomiMiMo/MiMo-7B-Base
    - https://huggingface.co/papers/2505.07608
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
    breadth: 2
    reliability: 3
    governance: 4
  modalities:
  - text
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://mimo.xiaomi.com/
  rationale: Small reasoning
  lifecycle: active
  source_urls: *id001
  model_classes:
  - fast
  task_suitability:
    architecture: 3
    algorithm_design: 4
    debugging: 4
    implementation: 4
    refactoring: 3
    code_review: 3
    test_generation: 4
    repo_navigation: 3
    agentic_workflow: 3
    tool_calling: 2
    structured_output: 3
    data_extraction: 3
    summarization: 3
    long_context_synthesis: 3
    rag: 3
    citation_answering: 3
    research_synthesis: 3
    math_reasoning: 5
    scientific_reasoning: 4
    legal_analysis: 2
    medical_admin: 2
    financial_analysis: 3
    translation: 4
    multilingual_chat: 4
    classification: 4
    sentiment_analysis: 4
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 4
    sql_generation: 4
    spreadsheet_analysis: 3
    ocr: 1
    image_understanding: 1
    chart_understanding: 2
    diagram_reasoning: 2
    voice: 1
    audio_transcription: 1
    video_understanding: 1
    low_latency_chat: 5
    bulk_generation: 5
    privacy_sensitive: 4
    self_hosted_enterprise: 4
---
