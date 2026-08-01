---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260503_1424-ModelSpec-nvidia-nvidia-nemotron-3-super-120b
  created: '2026-05-03T14:24:00Z'
spec:
  name: NVIDIA Nemotron 3 Super 120B
  kind: model-spec
  format: markdown
  provider: nvidia
  family: nvidia-nemotron-3-super-120b
  legacy_model_id: MODEL-nvidia-nvidia-nemotron-3-super-120b
  profile_ids:
  - nvidia-nemotron-3-super-120b
  versions:
  - version_id: '1'
    status: ga
    context_window: 256000
    pricing_note: NVIDIA hosted/self-hosted deployment pricing varies.
    lifecycle: active
    source_urls: &id001
    - https://build.nvidia.com/nvidia/nemotron-3-super-120b-a12b/modelcard
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
    governance: 5
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://www.nvidia.com/en-us/ai-data-science/
  rationale: Open enterprise reasoning
  lifecycle: active
  source_urls: *id001
  model_classes:
  - standard
  task_suitability:
    architecture: 4
    algorithm_design: 4
    debugging: 4
    implementation: 4
    refactoring: 4
    code_review: 4
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
    math_reasoning: 4
    scientific_reasoning: 4
    legal_analysis: 4
    medical_admin: 4
    financial_analysis: 4
    translation: 3
    multilingual_chat: 3
    classification: 4
    sentiment_analysis: 4
    creative_writing: 3
    marketing_copy: 3
    data_analysis: 4
    sql_generation: 4
    spreadsheet_analysis: 4
    ocr: 3
    image_understanding: 3
    chart_understanding: 3
    diagram_reasoning: 3
    voice: 3
    audio_transcription: 3
    video_understanding: 3
    low_latency_chat: 4
    bulk_generation: 4
    privacy_sensitive: 5
    self_hosted_enterprise: 5
---
