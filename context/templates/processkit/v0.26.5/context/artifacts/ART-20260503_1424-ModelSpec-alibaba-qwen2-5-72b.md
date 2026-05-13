---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-alibaba-qwen2-5-72b
  created: '2026-05-03T14:24:00Z'
spec:
  name: Qwen 2.5 72B
  kind: model-spec
  format: markdown
  provider: alibaba
  family: qwen2-5-72b
  legacy_model_id: MODEL-alibaba-qwen2-5-72b
  profile_ids:
  - qwen2.5-72b
  versions:
  - version_id: '2.5'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 0.4
      output: 1.2
    pricing_note: 'Self-hosted: GPU infra cost only. Via Together/Groq/etc.: ~$0.20–0.90/1M.
      Alibaba Cloud API available but G:1.'
    governance_warning: 'Via Alibaba Cloud API: G:1 (Chinese jurisdiction). Self-hosted
      (Apache 2.0 license): governance rises to G:4 — no data leaves your infra, but
      no compliance certs from Alibaba.'
    lifecycle: legacy
    source_urls: &id001
    - https://qwenlm.github.io/
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 4
    engineering: 4
    speed: 4
    breadth: 3
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.aliyun.com/
  rationale: Open-source frontier
  lifecycle: legacy
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 3
    algorithm_design: 4
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 3
    test_generation: 3
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
    math_reasoning: 4
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
    voice: 2
    audio_transcription: 2
    video_understanding: 2
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 4
    self_hosted_enterprise: 4
---
