---
apiVersion: processkit.projectious.work/v2
kind: Artifact
metadata:
  id: ART-20260505_1545-ModelSpec-xiaomi-mimo-7b
  created: '2026-05-05T15:45:00Z'
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
  - version_id: 7b-rl-0530
    status: ga
    context_window: 48000
    vendor_model_id: XiaomiMiMo/MiMo-7B-RL-0530
    pricing_usd_per_1m:
      input: null
      output: null
    pricing_note: MIT-licensed open weights; no official hosted API pricing was
      located. Self-hosting infrastructure cost applies.
    governance_warning: 'Self-hosted open weights: G:4. Xiaomi-hosted or
      third-party hosted deployments require separate data-retention and
      jurisdiction review before sensitive use.'
    lifecycle: active
    license: MIT
    source_urls: &id001
    - https://mimo.xiaomi.com/
    - https://huggingface.co/XiaomiMiMo/MiMo-7B-RL-0530
    - https://huggingface.co/XiaomiMiMo/MiMo-7B-Base
    - https://huggingface.co/papers/2505.07608
  efforts_supported:
  - none
  - low
  - medium
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
  equivalent_tier: s
  status_page_url: https://mimo.xiaomi.com/
  rationale: Small open-weight reasoning and coding model with strong math/code
    benchmark results for its size.
  lifecycle: active
  source_urls: *id001
  model_classes:
  - fast
  architecture:
    parameters: 8B
    training_tokens: 25T
    notes: 7B-class model with MTP/speculative-decoding support; Hugging Face
      reports 8B parameters for the published safetensors checkpoint.
  license:
    name: MIT
    open_weights: true
    commercial_use: true
  deployment:
    self_hostable: true
    recommended_runtimes:
    - SGLang
    - vLLM fork
    - Transformers
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
  tags:
  - model-routing
  - xiaomi
  - mimo
  - open-weights
  - reasoning
---
# MiMo-7B-RL-0530

MiMo-7B is Xiaomi's open-weight reasoning-model family. The verified
public lineage includes Base, SFT, RL-Zero, RL, and the later
`MiMo-7B-RL-0530` checkpoint.

The May 30 update reports the 0530 checkpoint at 97.2 on MATH500, 80.1
on AIME 2024, 70.2 on AIME 2025, 60.9 on LiveCodeBench v5, 52.2 on
LiveCodeBench v6, and 60.6 on GPQA-Diamond. That makes it a strong
small reasoning/coding candidate, not a frontier general-purpose model.

Use it when local/open-weight deployment, low latency, and math/code
reasoning matter more than broad multimodal coverage or mature hosted
enterprise controls.
