# Model Characteristics

This reference defines the provider-neutral fields processkit tracks for
models. It is intentionally broader than any single provider API because
frontier APIs, hosted open-weight endpoints, and self-hosted open models expose
different knobs.

## Coverage Matrix

| Area | Field(s) | Status |
|---|---|---|
| Identity | `provider`, `family`, `versions[].version_id`, `versions[].vendor_model_id` | Covered |
| Lifecycle | `versions[].status`, `released_at`, `deprecated_at`, `successor` | Covered |
| Routing class | `model_classes`, config `model_classes.fast/standard/powerful` | Covered |
| Effort / thinking | `efforts_supported`, `versions[].max_thinking_budget` | Covered |
| Capability | `dimensions.reasoning/engineering/speed/breadth/reliability/governance` | Covered |
| Modalities | `modalities`, `supported_parameters` | Covered |
| Context / output | `versions[].context_window`, `versions[].max_output` | Covered |
| Token accounting | `versions[].token_accounting`, `pricing_usd_per_1m.*` | Covered |
| Cost | `versions[].pricing_usd_per_1m` | Covered |
| Latency / throughput | `versions[].latency_p50_ms`, `versions[].rate_limits` | Covered |
| Access | `access_tier`, model-recommender user config | Covered |
| Preference | model-assignment Bindings, config `preferred_providers` | Covered |
| Governance | `jurisdiction`, `data_privacy`, `license`, `deployment` | Covered |
| Open-weight metadata | `architecture`, `license`, `deployment` | Covered |
| Operations | `status_page_url`, config `blocked_models` | Covered |

## Required Baseline

Every `Model` must have:

- `provider`
- `family`
- at least one `versions[]` entry with `version_id` and `status`
- `efforts_supported`
- `dimensions`
- `modalities`
- `access_tier`
- `equivalent_tier`

The remaining fields are optional because vendors disclose them unevenly.
Absence must be treated conservatively for governance and production routing:
unknown privacy, rate limit, or license posture does not satisfy a hard
requirement.

## Model Classes

Model classes are routing shortcuts, not capability claims:

| Class | Use | Typical constraints |
|---|---|---|
| `fast` | bulk transforms, simple routing, latency-sensitive work | high Speed, low cost, adequate Reliability |
| `standard` | default coding, review, analysis, tool use | balanced Engineering, Reliability, cost |
| `powerful` | hardest architecture, research, debugging, high-cost mistakes | strongest Reasoning/Engineering/Reliability |

Projects should map these classes in `model-recommender/mcp/user_config.json`
or through `set_config(model_classes=...)`. Callers should use
`get_model_for_class()` instead of hard-coding vendor IDs.

## Open-Weight Fields

Open-source and open-weight models need facts that hosted APIs often hide:

- `architecture.model_type`
- `architecture.parameter_count`
- `architecture.active_parameters`
- `architecture.tokenizer`
- `architecture.quantization`
- `license.name`
- `license.open_weights`
- `license.commercial_use`
- `deployment.self_hostable`
- `deployment.min_vram_gb`
- `deployment.supported_runtimes`

For a hosted open-weight endpoint, keep the model license fields on the model
and put endpoint-specific retention, residency, rate-limit, and price facts in
the version or hosting provider overlay when that exists.

## Token Accounting

Token consumption is not just input plus output. Providers may separately count
or bill:

- cached input reads and writes
- internal reasoning tokens
- image, audio, or video units
- tool calls, web search, or code execution
- request-level minimums

Use `token_accounting` to explain how usage is counted and
`pricing_usd_per_1m` for numeric prices. When the provider does not disclose a
detail, omit it instead of guessing.

## Research Sources

Refresh this reference from primary provider docs and model cards:

- OpenAI model/API docs and pricing
- Anthropic model/pricing/docs
- Google Gemini model docs and pricing
- Hugging Face model card metadata conventions
- Meta Llama model cards and license
- Mistral model docs and open-weight license docs
- Serving-layer docs such as vLLM or Ollama for self-hosted deployment fields

