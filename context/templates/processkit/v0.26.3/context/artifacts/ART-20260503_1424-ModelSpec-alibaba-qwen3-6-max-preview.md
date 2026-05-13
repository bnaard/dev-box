---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-alibaba-qwen3-6-max-preview
  created: '2026-05-03T14:24:00Z'
spec:
  name: Qwen 3.6-Max-Preview
  kind: model-spec
  format: markdown
  provider: alibaba
  family: qwen3.6-max-preview
  legacy_model_id: MODEL-alibaba-qwen3-6-max-preview
  profile_ids:
  - qwen3.6-max-preview
  versions:
  - version_id: '1'
    status: preview
    context_window: 1000000
    pricing_note: 'ESTIMATED — preview released 2026-04-20. CONFIRMED to top six benchmarks
      per coverage: SWE-bench Pro, Terminal-Bench 2.0, SkillsBench, QwenClawBench,
      QwenWebBench, SciCode. Pricing not yet published; preview access only. Run Workflow
      C to validate.'
    governance_warning: 'Chinese company, proprietary preview (no open weights). Via
      Alibaba Cloud API: G:1. Self-hosting NOT available.'
    lifecycle: unverified
    source_urls: &id001
    - https://qwenlm.github.io/
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 5
    engineering: 5
    speed: 3
    breadth: 5
    reliability: 4
    governance: 1
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.aliyun.com/
  rationale: Frontier proprietary preview (estimated, 2026-04-25)
  lifecycle: unverified
  source_urls: *id001
  model_classes: []
  task_suitability:
    architecture: 4
    algorithm_design: 4
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 4
    test_generation: 3
    repo_navigation: 4
    agentic_workflow: 4
    tool_calling: 4
    structured_output: 4
    data_extraction: 4
    summarization: 4
    long_context_synthesis: 4
    rag: 4
    citation_answering: 4
    research_synthesis: 4
    math_reasoning: 4
    scientific_reasoning: 4
    legal_analysis: 3
    medical_admin: 3
    financial_analysis: 3
    translation: 4
    multilingual_chat: 4
    classification: 3
    sentiment_analysis: 3
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 4
    sql_generation: 4
    spreadsheet_analysis: 4
    ocr: 4
    image_understanding: 4
    chart_understanding: 4
    diagram_reasoning: 4
    voice: 3
    audio_transcription: 3
    video_understanding: 4
    low_latency_chat: 3
    bulk_generation: 3
    privacy_sensitive: 1
    self_hosted_enterprise: 2
---
