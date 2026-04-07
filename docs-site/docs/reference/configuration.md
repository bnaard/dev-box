---
sidebar_position: 2
title: Configuration
---

# Configuration

`aibox.toml` is the single source of truth for an aibox project. All generated files derive from it.

## Full Specification

```toml
[aibox]
version = "0.11.0"                    # aibox version used to generate this project
base = "debian"                       # Base image

[container]
name = "my-app"                       # Container name
hostname = "my-app"                   # Container hostname
user = "aibox"                        # Container user (default: aibox)
post_create_command = "npm install"   # Command to run after container creation

[context]
packages = ["managed"]               # Process packages or presets
schema_version = "1.0.0"             # Context schema version (semver)

[addons.python.tools]                 # Addon: Python runtime
python = { version = "3.13" }
uv = { version = "0.7" }

[addons.rust.tools]                   # Addon: Rust toolchain
rustc = { version = "1.87" }
clippy = {}
rustfmt = {}

[skills]
include = ["data-science"]            # Extra skills beyond process packages
exclude = ["debugging"]               # Skills to remove from active set

[ai]
providers = ["claude", "aider"]       # AI providers to install

[customization]
theme = "gruvbox-dark"               # Color theme (7 options)
prompt = "default"                   # Starship preset (7 options)
layout = "dev"                       # Zellij layout (4 options)

[audio]
enabled = false                       # Enable audio bridging
pulse_server = "tcp:host.docker.internal:4714"
```

## Section Reference

### [aibox]

Top-level project metadata.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `version` | String (semver) | Yes | -- | Project version |
| `base` | String | No | `"debian"` | Base image: `debian` |

### [container]

Container configuration. Controls the generated `docker-compose.yml` and `Dockerfile`.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | Yes | -- | Container name (used by compose and runtime inspect) |
| `hostname` | String | No | `"aibox"` | Container hostname |
| `user` | String | No | `"aibox"` | Container user |
| `post_create_command` | String | No | -- | Command to run after container creation |
| `keepalive` | Boolean | No | `false` | Network keepalive (prevents OrbStack/VM NAT idle dropout) |

:::tip Customizing ports, packages, volumes, and environment variables
Use `Dockerfile.local` for installing additional packages, and `docker-compose.override.yml` for ports, volumes, and environment variables. Both files are scaffolded by `aibox init` and are never overwritten by `aibox sync`.
:::

### [context]

Context system configuration. Controls which context files and skills are scaffolded, and tracks the schema version.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `packages` | Array of strings | Yes | `["core"]` | Process packages or presets. Must include at least one. |
| `schema_version` | String (semver) | No | `"1.0.0"` | Context schema version |

**13 packages:** `core`, `tracking`, `standups`, `handover`, `product`, `code`, `research`, `documentation`, `design`, `architecture`, `security`, `data`, `operations`

**4 convenience presets** (expand to multiple packages):

| Preset | Expands to |
|--------|-----------|
| `managed` | core, tracking, standups, handover |
| `software` | core, tracking, standups, handover, code, architecture |
| `research-project` | core, tracking, standups, handover, research, documentation |
| `full-product` | core, tracking, standups, handover, code, architecture, design, product, security, operations |

### [addons]

Addons install language runtimes, tool bundles, and AI agents into the container. Each addon is a named table with a `tools` sub-table.

```toml
[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }
```

Run `aibox addon list` to see all 21 available addons, or `aibox addon info <name>` for tool details and supported versions. See the [Addons page](../addons/overview.md) for full documentation.

### [skills]

Skill management. The effective skill set is built from three sources in order:

1. **Process packages** -- skills bundled with the packages listed in `[context].packages`
2. **Addon skills** -- skills recommended by active addons (e.g., `python` addon auto-deploys `python-best-practices`)
3. **Include/exclude** -- manual overrides from this section

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `include` | Array of strings | No | `[]` | Additional skills to deploy beyond those from packages and addons |
| `exclude` | Array of strings | No | `[]` | Skills to remove from the active set |

Core skills (`agent-management`, `owner-profile`) cannot be excluded.

Run `aibox skill list` to see all 84 available skills and their deploy status. See the [Skills Library](../skills/index.md) for the full deployment model.

### [ai]

AI provider configuration. Providers listed here are automatically installed as addons.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `providers` | Array of strings | No | `["claude"]` | AI providers: `claude`, `aider`, `gemini`, `mistral` |

### [agents]

Controls how `aibox init` scaffolds the canonical agent entry document
(`AGENTS.md`) and the provider-specific entry files (`CLAUDE.md`, future
`CODEX.md`, …). The principle is **provider neutrality**: every agent
harness reads the same `AGENTS.md` so a project doesn't have to keep
N copies of the same instructions in sync. Provider files exist only
to satisfy specific harnesses' auto-load conventions (Claude Code
auto-loads `CLAUDE.md` at startup, etc.).

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `canonical` | String | No | `"AGENTS.md"` | Filename of the canonical agent entry document. Almost no one should override this — the default matches the [agents.md](https://agents.md/) ecosystem convention. |
| `provider_mode` | String | No | `"pointer"` | How provider files are scaffolded. `pointer` (recommended): provider files are thin pointers that say "see AGENTS.md". `full`: provider files contain the rich provider-flavoured content — use only when a project genuinely needs different instructions per harness. |

`aibox init` always creates `AGENTS.md` (write-if-missing — never
overwrites). When the Claude provider is enabled, it also creates
`CLAUDE.md`, either as a thin pointer (default) or with the full rich
content (`provider_mode = "full"`). Other providers (Aider, Gemini,
Mistral) use config files rather than markdown entries and are not
affected by this section.

Existing files are never overwritten. If you already have a hand-written
`AGENTS.md` or `CLAUDE.md`, `aibox init` leaves it alone.

### [customization]

Visual and layout configuration. See [Themes](../customization/themes.md) and [Layouts](../customization/layouts.md) for details.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `theme` | String | No | `"gruvbox-dark"` | Color theme: `gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`, `tokyo-night`, `nord`, `projectious` |
| `prompt` | String | No | `"default"` | Starship preset: `default`, `plain`, `arrow`, `minimal`, `nerd-font`, `pastel`, `bracketed` |
| `layout` | String | No | `"dev"` | Zellij layout: `dev`, `focus`, `cowork`, `browse` |

### [audio]

Audio bridging configuration.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | Boolean | No | `false` | Enable PulseAudio environment setup |
| `pulse_server` | String | No | `"tcp:host.docker.internal:4714"` | PulseAudio server address |

## Environment Variable Overrides

Some settings can be overridden via environment variables:

| Variable | Overrides | Description |
|----------|-----------|-------------|
| `AIBOX_HOST_ROOT` | `.aibox-home/` path | Host directory for persistent config (default: `.aibox-home/`) |
| `AIBOX_WORKSPACE_DIR` | Workspace mount source | Host directory mounted as `/workspace` |
| `AIBOX_LOG_LEVEL` | `--log-level` | Log verbosity (`trace`, `debug`, `info`, `warn`, `error`) |

Example:

```bash
AIBOX_WORKSPACE_DIR=/home/user/projects/my-app aibox start
```
