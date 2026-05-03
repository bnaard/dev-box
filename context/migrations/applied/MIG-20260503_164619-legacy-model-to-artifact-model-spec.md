---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260503_164619-legacy-model-to-artifact-model-spec
  created: 2026-05-03 16:46:19.371013+02:00
  updated: '2026-05-03T16:55:42+00:00'
spec:
  source: aibox
  state: applied
  generated_by: aibox apply
  generated_at: 2026-05-03 16:46:19.371013+02:00
  summary: Legacy Model entities must migrate to Artifact model-spec records
  affected_groups:
  - models
  - bindings
  marker: legacy-model-to-artifact-model-spec
  started_at: '2026-05-03T16:55:42+00:00'
  applied_at: '2026-05-03T16:55:42+00:00'
  progress_notes:
  - timestamp: '2026-05-03T16:55:42+00:00'
    actor: mcp
    note: Resolved per owner request. Reviewed the generated legacy MODEL-* to Artifact
      model-spec migration. Kept it as an applied advisory because the current artifact/binding
      MCP tools do not support preserving the migration's intended ART-*-model-spec
      IDs or rewriting existing binding targets in place; avoiding raw context entity
      edits preserves the processkit MCP write contract.
---

# Migration MIG-20260503_164619-legacy-model-to-artifact-model-spec

processkit no longer treats `Model` as a live primitive. Convert legacy model descriptions into `Artifact` entities with `spec.kind: model-spec`, then point `model-assignment` bindings at those artifacts.

## Model entity conversions

### `context/models/MODEL-alibaba-qwen2-5-72b.md` -> `context/artifacts/ART-alibaba-qwen2-5-72b-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-alibaba-qwen2-5-72b-model-spec
spec:
  name: 'Model spec: alibaba qwen2-5-72b'
  kind: model-spec
  legacy_model_id: MODEL-alibaba-qwen2-5-72b
  source_model_file: context/models/MODEL-alibaba-qwen2-5-72b.md
  provider: alibaba
  family: qwen2-5-72b
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-alibaba-qwen2-5-coder-32b.md` -> `context/artifacts/ART-alibaba-qwen2-5-coder-32b-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-alibaba-qwen2-5-coder-32b-model-spec
spec:
  name: 'Model spec: alibaba qwen2-5-coder-32b'
  kind: model-spec
  legacy_model_id: MODEL-alibaba-qwen2-5-coder-32b
  source_model_file: context/models/MODEL-alibaba-qwen2-5-coder-32b.md
  provider: alibaba
  family: qwen2-5-coder-32b
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-alibaba-qwen3-235b.md` -> `context/artifacts/ART-alibaba-qwen3-235b-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-alibaba-qwen3-235b-model-spec
spec:
  name: 'Model spec: alibaba qwen3-235b'
  kind: model-spec
  legacy_model_id: MODEL-alibaba-qwen3-235b
  source_model_file: context/models/MODEL-alibaba-qwen3-235b.md
  provider: alibaba
  family: qwen3-235b
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-anthropic-claude-haiku.md` -> `context/artifacts/ART-anthropic-claude-haiku-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-anthropic-claude-haiku-model-spec
spec:
  name: 'Model spec: anthropic claude-haiku'
  kind: model-spec
  legacy_model_id: MODEL-anthropic-claude-haiku
  source_model_file: context/models/MODEL-anthropic-claude-haiku.md
  provider: anthropic
  family: claude-haiku
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-anthropic-claude-opus.md` -> `context/artifacts/ART-anthropic-claude-opus-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-anthropic-claude-opus-model-spec
spec:
  name: 'Model spec: anthropic claude-opus'
  kind: model-spec
  legacy_model_id: MODEL-anthropic-claude-opus
  source_model_file: context/models/MODEL-anthropic-claude-opus.md
  provider: anthropic
  family: claude-opus
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-anthropic-claude-sonnet.md` -> `context/artifacts/ART-anthropic-claude-sonnet-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-anthropic-claude-sonnet-model-spec
spec:
  name: 'Model spec: anthropic claude-sonnet'
  kind: model-spec
  legacy_model_id: MODEL-anthropic-claude-sonnet
  source_model_file: context/models/MODEL-anthropic-claude-sonnet.md
  provider: anthropic
  family: claude-sonnet
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-cohere-command-r-plus.md` -> `context/artifacts/ART-cohere-command-r-plus-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-cohere-command-r-plus-model-spec
spec:
  name: 'Model spec: cohere command-r-plus'
  kind: model-spec
  legacy_model_id: MODEL-cohere-command-r-plus
  source_model_file: context/models/MODEL-cohere-command-r-plus.md
  provider: cohere
  family: command-r-plus
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-deepseek-deepseek-r.md` -> `context/artifacts/ART-deepseek-deepseek-r-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-deepseek-deepseek-r-model-spec
spec:
  name: 'Model spec: deepseek deepseek-r'
  kind: model-spec
  legacy_model_id: MODEL-deepseek-deepseek-r
  source_model_file: context/models/MODEL-deepseek-deepseek-r.md
  provider: deepseek
  family: deepseek-r
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-deepseek-deepseek-v.md` -> `context/artifacts/ART-deepseek-deepseek-v-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-deepseek-deepseek-v-model-spec
spec:
  name: 'Model spec: deepseek deepseek-v'
  kind: model-spec
  legacy_model_id: MODEL-deepseek-deepseek-v
  source_model_file: context/models/MODEL-deepseek-deepseek-v.md
  provider: deepseek
  family: deepseek-v
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-google-gemini-2-5-flash.md` -> `context/artifacts/ART-google-gemini-2-5-flash-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-google-gemini-2-5-flash-model-spec
spec:
  name: 'Model spec: google gemini-2-5-flash'
  kind: model-spec
  legacy_model_id: MODEL-google-gemini-2-5-flash
  source_model_file: context/models/MODEL-google-gemini-2-5-flash.md
  provider: google
  family: gemini-2-5-flash
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-google-gemini-2-5-pro.md` -> `context/artifacts/ART-google-gemini-2-5-pro-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-google-gemini-2-5-pro-model-spec
spec:
  name: 'Model spec: google gemini-2-5-pro'
  kind: model-spec
  legacy_model_id: MODEL-google-gemini-2-5-pro
  source_model_file: context/models/MODEL-google-gemini-2-5-pro.md
  provider: google
  family: gemini-2-5-pro
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-google-gemini-3-1-pro.md` -> `context/artifacts/ART-google-gemini-3-1-pro-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-google-gemini-3-1-pro-model-spec
spec:
  name: 'Model spec: google gemini-3-1-pro'
  kind: model-spec
  legacy_model_id: MODEL-google-gemini-3-1-pro
  source_model_file: context/models/MODEL-google-gemini-3-1-pro.md
  provider: google
  family: gemini-3-1-pro
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-google-gemini-3-flash.md` -> `context/artifacts/ART-google-gemini-3-flash-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-google-gemini-3-flash-model-spec
spec:
  name: 'Model spec: google gemini-3-flash'
  kind: model-spec
  legacy_model_id: MODEL-google-gemini-3-flash
  source_model_file: context/models/MODEL-google-gemini-3-flash.md
  provider: google
  family: gemini-3-flash
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-google-gemini-flash.md` -> `context/artifacts/ART-google-gemini-flash-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-google-gemini-flash-model-spec
spec:
  name: 'Model spec: google gemini-flash'
  kind: model-spec
  legacy_model_id: MODEL-google-gemini-flash
  source_model_file: context/models/MODEL-google-gemini-flash.md
  provider: google
  family: gemini-flash
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-google-gemma-3-27b.md` -> `context/artifacts/ART-google-gemma-3-27b-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-google-gemma-3-27b-model-spec
spec:
  name: 'Model spec: google gemma-3-27b'
  kind: model-spec
  legacy_model_id: MODEL-google-gemma-3-27b
  source_model_file: context/models/MODEL-google-gemma-3-27b.md
  provider: google
  family: gemma-3-27b
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-meta-llama-3-70b.md` -> `context/artifacts/ART-meta-llama-3-70b-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-meta-llama-3-70b-model-spec
spec:
  name: 'Model spec: meta llama-3-70b'
  kind: model-spec
  legacy_model_id: MODEL-meta-llama-3-70b
  source_model_file: context/models/MODEL-meta-llama-3-70b.md
  provider: meta
  family: llama-3-70b
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-microsoft-phi.md` -> `context/artifacts/ART-microsoft-phi-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-microsoft-phi-model-spec
spec:
  name: 'Model spec: microsoft phi'
  kind: model-spec
  legacy_model_id: MODEL-microsoft-phi
  source_model_file: context/models/MODEL-microsoft-phi.md
  provider: microsoft
  family: phi
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-minimax-minimax-m.md` -> `context/artifacts/ART-minimax-minimax-m-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-minimax-minimax-m-model-spec
spec:
  name: 'Model spec: minimax minimax-m'
  kind: model-spec
  legacy_model_id: MODEL-minimax-minimax-m
  source_model_file: context/models/MODEL-minimax-minimax-m.md
  provider: minimax
  family: minimax-m
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-minimax-minimax-text.md` -> `context/artifacts/ART-minimax-minimax-text-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-minimax-minimax-text-model-spec
spec:
  name: 'Model spec: minimax minimax-text'
  kind: model-spec
  legacy_model_id: MODEL-minimax-minimax-text
  source_model_file: context/models/MODEL-minimax-minimax-text.md
  provider: minimax
  family: minimax-text
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-mistral-codestral.md` -> `context/artifacts/ART-mistral-codestral-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-mistral-codestral-model-spec
spec:
  name: 'Model spec: mistral codestral'
  kind: model-spec
  legacy_model_id: MODEL-mistral-codestral
  source_model_file: context/models/MODEL-mistral-codestral.md
  provider: mistral
  family: codestral
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-mistral-mistral-deep-think.md` -> `context/artifacts/ART-mistral-mistral-deep-think-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-mistral-mistral-deep-think-model-spec
spec:
  name: 'Model spec: mistral mistral-deep-think'
  kind: model-spec
  legacy_model_id: MODEL-mistral-mistral-deep-think
  source_model_file: context/models/MODEL-mistral-mistral-deep-think.md
  provider: mistral
  family: mistral-deep-think
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-mistral-mistral-large.md` -> `context/artifacts/ART-mistral-mistral-large-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-mistral-mistral-large-model-spec
spec:
  name: 'Model spec: mistral mistral-large'
  kind: model-spec
  legacy_model_id: MODEL-mistral-mistral-large
  source_model_file: context/models/MODEL-mistral-mistral-large.md
  provider: mistral
  family: mistral-large
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-mistral-mistral-medium.md` -> `context/artifacts/ART-mistral-mistral-medium-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-mistral-mistral-medium-model-spec
spec:
  name: 'Model spec: mistral mistral-medium'
  kind: model-spec
  legacy_model_id: MODEL-mistral-mistral-medium
  source_model_file: context/models/MODEL-mistral-mistral-medium.md
  provider: mistral
  family: mistral-medium
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-mistral-mistral-small.md` -> `context/artifacts/ART-mistral-mistral-small-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-mistral-mistral-small-model-spec
spec:
  name: 'Model spec: mistral mistral-small'
  kind: model-spec
  legacy_model_id: MODEL-mistral-mistral-small
  source_model_file: context/models/MODEL-mistral-mistral-small.md
  provider: mistral
  family: mistral-small
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-openai-gpt-4o.md` -> `context/artifacts/ART-openai-gpt-4o-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-openai-gpt-4o-model-spec
spec:
  name: 'Model spec: openai gpt-4o'
  kind: model-spec
  legacy_model_id: MODEL-openai-gpt-4o
  source_model_file: context/models/MODEL-openai-gpt-4o.md
  provider: openai
  family: gpt-4o
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-openai-gpt-5.md` -> `context/artifacts/ART-openai-gpt-5-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-openai-gpt-5-model-spec
spec:
  name: 'Model spec: openai gpt-5'
  kind: model-spec
  legacy_model_id: MODEL-openai-gpt-5
  source_model_file: context/models/MODEL-openai-gpt-5.md
  provider: openai
  family: gpt-5
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-openai-gpt-5-pro.md` -> `context/artifacts/ART-openai-gpt-5-pro-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-openai-gpt-5-pro-model-spec
spec:
  name: 'Model spec: openai gpt-5-pro'
  kind: model-spec
  legacy_model_id: MODEL-openai-gpt-5-pro
  source_model_file: context/models/MODEL-openai-gpt-5-pro.md
  provider: openai
  family: gpt-5-pro
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-openai-o3.md` -> `context/artifacts/ART-openai-o3-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-openai-o3-model-spec
spec:
  name: 'Model spec: openai o3'
  kind: model-spec
  legacy_model_id: MODEL-openai-o3
  source_model_file: context/models/MODEL-openai-o3.md
  provider: openai
  family: o3
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-openai-o4-mini.md` -> `context/artifacts/ART-openai-o4-mini-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-openai-o4-mini-model-spec
spec:
  name: 'Model spec: openai o4-mini'
  kind: model-spec
  legacy_model_id: MODEL-openai-o4-mini
  source_model_file: context/models/MODEL-openai-o4-mini.md
  provider: openai
  family: o4-mini
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-xai-grok-3.md` -> `context/artifacts/ART-xai-grok-3-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-xai-grok-3-model-spec
spec:
  name: 'Model spec: xai grok-3'
  kind: model-spec
  legacy_model_id: MODEL-xai-grok-3
  source_model_file: context/models/MODEL-xai-grok-3.md
  provider: xai
  family: grok-3
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-xai-grok-3-5.md` -> `context/artifacts/ART-xai-grok-3-5-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-xai-grok-3-5-model-spec
spec:
  name: 'Model spec: xai grok-3-5'
  kind: model-spec
  legacy_model_id: MODEL-xai-grok-3-5
  source_model_file: context/models/MODEL-xai-grok-3-5.md
  provider: xai
  family: grok-3-5
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-xai-grok-4.md` -> `context/artifacts/ART-xai-grok-4-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-xai-grok-4-model-spec
spec:
  name: 'Model spec: xai grok-4'
  kind: model-spec
  legacy_model_id: MODEL-xai-grok-4
  source_model_file: context/models/MODEL-xai-grok-4.md
  provider: xai
  family: grok-4
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-xai-grok-4-1.md` -> `context/artifacts/ART-xai-grok-4-1-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-xai-grok-4-1-model-spec
spec:
  name: 'Model spec: xai grok-4-1'
  kind: model-spec
  legacy_model_id: MODEL-xai-grok-4-1
  source_model_file: context/models/MODEL-xai-grok-4-1.md
  provider: xai
  family: grok-4-1
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

### `context/models/MODEL-xai-grok-4-heavy.md` -> `context/artifacts/ART-xai-grok-4-heavy-model-spec.md`

Create this artifact:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-xai-grok-4-heavy-model-spec
spec:
  name: 'Model spec: xai grok-4-heavy'
  kind: model-spec
  legacy_model_id: MODEL-xai-grok-4-heavy
  source_model_file: context/models/MODEL-xai-grok-4-heavy.md
  provider: xai
  family: grok-4-heavy
---
```

Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.

## Binding rewrites

Rewrite these `model-assignment` bindings so the target is the model-spec Artifact and `target_kind` is `Artifact`:

- `context/bindings/BIND-20260425_0955-BoldSage-model-assignment.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-CuriousDew-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-FierceHare-model-assignment.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-GentleGlade-model-assignment.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-GrandGrove-model-assignment.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-KindPeak-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-NeatButter-model-assignment.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-SleekStream-model-assignment.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-SnowyReef-model-assignment.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-ThriftyGlade-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-ThriftySpruce-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0955-TidyArch-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0956-CleverRobin-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0956-GentleHare-model-assignment.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0956-LoyalDaisy-model-assignment.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0956-NeatGrove-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0956-RoyalEagle-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0956-SoundRaven-model-assignment.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0956-ThriftyClover-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260425_0956-WiseBrook-model-assignment.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1609-ShinyPine-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1609-SpryBrook-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1609-SunnyEagle-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1613-AmberAsh-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1613-PluckyGarnet-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1613-ProudFox-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1613-SpryVale-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1613-StoutAnt-model-assignment.md`: `target: MODEL-openai-gpt-5-pro` -> `target: ART-openai-gpt-5-pro-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1614-DaringWillow-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1614-FierceFalcon-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1614-StoutBrook-model-assignment.md`: `target: MODEL-openai-gpt-5` -> `target: ART-openai-gpt-5-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1625-SoundGlade-model-assignment.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1626-HonestBeam-model-assignment.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-20260429_1626-LivelyDew-model-assignment.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-ai-research-scientist-junior-ha25e5b.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-ai-research-scientist-principal-h5e96d8.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-ai-research-scientist-senior-h18c312.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-assistant-junior-hae1bfb.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-assistant-principal-hb5ac7b.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-assistant-senior-h771629.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-data-scientist-junior-he28d96.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-data-scientist-principal-h192434.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-data-scientist-senior-h8816d2.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-product-manager-junior-h9275a0.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-product-manager-principal-h723e55.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-product-manager-senior-hf3f8e9.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-qa-engineer-junior-h796cc2.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-qa-engineer-principal-h07f4cd.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-qa-engineer-senior-h0436ab.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-research-scientist-junior-h87372f.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-research-scientist-principal-hb4e3bb.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-research-scientist-senior-h14c6d3.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-security-architect-junior-hf79510.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-security-architect-principal-h2c8917.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-security-architect-senior-h142586.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-software-engineer-junior-h92feb2.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-software-engineer-principal-ha71725.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-software-engineer-senior-h7bd319.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-solutions-architect-junior-heb66a8.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-solutions-architect-principal-h5264ce.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-solutions-architect-senior-h603b59.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-technical-writer-junior-h431a38.md`: `target: MODEL-anthropic-claude-haiku` -> `target: ART-anthropic-claude-haiku-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-technical-writer-principal-h56b361.md`: `target: MODEL-anthropic-claude-opus` -> `target: ART-anthropic-claude-opus-model-spec`; set `target_kind: Artifact`
- `context/bindings/BIND-technical-writer-senior-hb93488.md`: `target: MODEL-anthropic-claude-sonnet` -> `target: ART-anthropic-claude-sonnet-model-spec`; set `target_kind: Artifact`

## Legacy schema cleanup

- Archive or remove `context/schemas/model.yaml` after all model files and bindings have been migrated.
- Archive or remove `context/models/` after the new `context/artifacts/ART-*-model-spec.md` files validate.

## Validation

- Run `pk-doctor` and resolve any artifact or binding schema errors.
- Rebuild or refresh model-recommender projections so `model_scores.json` is derived from model-spec artifacts, not hidden `MODEL-*` records.
