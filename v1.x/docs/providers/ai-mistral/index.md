# Mistral

LLMS index: [llms.txt](/aibox/v1.x/llms.txt)

---

# Mistral (SDK)

<div class="alert alert-secondary" role="alert"><div class="h4 alert-heading" role="heading">SDK addon — not an interactive CLI</div>


The `ai-mistral` addon installs the **mistralai Python SDK**, not an interactive coding CLI. It is intended for projects that call the Mistral API programmatically. For an interactive coding experience, use [Claude](./ai-claude.md), [Gemini](./ai-gemini.md), [OpenAI Codex](./ai-openai.md), or [Copilot](./ai-copilot.md) instead.
</div>


[Mistral AI](https://mistral.ai) provides large language models via Python SDK.

## Setup

```toml
[ai]
model_providers = ["mistral"]

[addons.ai-mistral.tools]
mistral = {}
```

`mistral` is retained as a legacy harness value for old configs, but it is not
a current interactive CLI harness. Use the addon directly for SDK installs.
Run `aibox apply`. Inside the container the `mistralai` Python SDK is available
for scripting:

```python
from mistralai import Mistral
client = Mistral(api_key="...")
```

## API Key

```toml
[container.environment]
MISTRAL_API_KEY = "..."
```

## MCP Integration

aibox generates `.mcp.json` (the Claude Code MCP format) on `aibox apply` when a compatible harness is enabled, merging processkit built-in servers in processkit mode, team servers from `aibox.toml [ai.mcp]`, and personal servers from `.aibox-local.toml [mcp]`. A custom Mistral SDK-based tool you build can read MCP server registrations from this file.

`.mcp.json` is **gitignored** — it is regenerated on every `aibox apply` and must not be committed.

## Installation

The Mistral AI SDK is installed via pip (`pip install --no-cache-dir mistralai`).
