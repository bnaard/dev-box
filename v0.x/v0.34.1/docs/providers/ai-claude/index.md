# Claude


# Claude Code

[Claude Code](https://claude.ai) by Anthropic is a terminal-based AI coding assistant. It's the default provider in aibox.

## Setup

```toml
[ai]
harnesses = [
  { harness = "claude", enable = true, install = true },
]
```

Run `aibox apply`, then inside the container:

```bash
claude   # Launches Claude Code CLI
```

On first launch, Claude prompts for authentication via browser login.

## Configuration

Claude's configuration, cache, and account state are persisted under `.aibox-home/`
and mounted into the container. aibox preserves Claude's primary config directory
(`.claude/`), top-level account state (`.claude.json`), and XDG state/cache
locations used by current Claude Code releases.

Key files:
- `.claude/settings.json` — Claude Code settings
- `.claude.json` — Claude Code account/install state
- `.cache/claude*`, `.config/claude/`, `.local/share/claude/`, `.local/state/claude/` — Claude Code login and runtime state
- `.claude/projects/` — Per-project memory and context
- `.claude/skills/<name>/SKILL.md` — generated processkit command shims when Claude is enabled in processkit mode

The generated `.claude/skills/` entries are adapters. The canonical skill
instructions remain in `context/skills/`. Harness-only projects do not generate
processkit command shims.

## Audio (Voice)

Claude Code supports voice input. To enable it, configure [audio bridging](../container/audio.md):

```toml
[audio]
enabled = true
```

## MCP Integration

Claude Code's native MCP client reads `.mcp.json`. aibox generates `.mcp.json` automatically on `aibox apply`, merging processkit built-in servers in processkit mode, team servers from `aibox.toml [ai.mcp]`, and personal servers from `.aibox-local.toml [mcp]`.

`.mcp.json` is **gitignored** — it is regenerated on every `aibox apply` and must not be committed.

To add MCP servers:

```toml
# aibox.toml — team-shared servers
[[ai.mcp.servers]]
name    = "github"
command = "npx"
args    = ["-y", "@modelcontextprotocol/server-github"]

# .aibox-local.toml — personal servers (not committed)
[[mcp.servers]]
name    = "my-internal-tool"
command = "npx"
args    = ["-y", "@acme/internal-mcp-server"]
```

## tmux Integration

When Claude is configured as a provider, tmux layouts include a dedicated Claude pane:

- **dev layout:** Claude gets its own window
- **focus layout:** Claude gets its own window
- **cowork layout:** Claude appears in a side-by-side pane next to the editor


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.1/docs/providers/ai-claude/index.md
