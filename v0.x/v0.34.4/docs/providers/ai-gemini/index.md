# Gemini


# Gemini

[Gemini CLI](https://github.com/google-gemini/gemini-cli) is Google's command-line interface for Gemini AI models.

## Setup

```toml
[ai]
harnesses = [
  { harness = "gemini", enable = true, install = true },
]
```

Run `aibox apply`, then inside the container:

```bash
gemini   # Launches Gemini CLI
```

## API Key

```toml
[container.environment]
GOOGLE_API_KEY = "..."
```

## Configuration

Gemini's configuration is persisted in `.aibox-home/.gemini/`, mounted at `/home/aibox/.gemini/`.

## MCP Integration

Gemini CLI reads `.gemini/settings.json`. aibox generates this file automatically on `aibox apply`, merging processkit built-in servers in processkit mode, team servers from `aibox.toml [ai.mcp]`, and personal servers from `.aibox-local.toml [mcp]`.

`.gemini/settings.json` is **gitignored** — it is regenerated on every `aibox apply` and must not be committed.

To add MCP servers:

```toml
# aibox.toml — team-shared servers
[[ai.mcp.servers]]
name    = "github"
command = "npx"
args    = ["-y", "@modelcontextprotocol/server-github"]

# .aibox-local.toml — personal servers
[[mcp.servers]]
name    = "my-internal-tool"
command = "npx"
args    = ["-y", "@acme/internal-mcp-server"]
```

## Installation

Gemini CLI is installed via npm (`npm install -g @google/generative-ai-cli`), with a pip fallback.


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.4/docs/providers/ai-gemini/index.md
