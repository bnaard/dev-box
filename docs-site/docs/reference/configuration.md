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
ports = ["8000:8000", "5432:5432"]    # Port mappings (host:container)
extra_packages = ["postgresql-client"] # Additional apt packages
environment = { MY_VAR = "value" }    # Environment variables
post_create_command = "npm install"   # Command to run after container creation
vscode_extensions = [                 # VS Code extensions to install
    "ms-python.python",
    "ms-python.vscode-pylance",
]

# [[container.extra_volumes]]
# source = "host-path"
# target = "container-path"
# read_only = false

[process]
packages = ["managed"]               # Process packages or presets

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

[context]
schema_version = "1.0.0"             # Context schema version (semver)

[ai]
providers = ["claude", "aider"]       # AI providers to install

[appearance]
theme = "gruvbox-dark"               # Color theme (7 options)
prompt = "default"                   # Starship preset (6 options)

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
| `ports` | Array of strings | No | `[]` | Port mappings in `host:container` format |
| `extra_packages` | Array of strings | No | `[]` | Additional apt packages to install |
| `extra_volumes` | Array of objects | No | `[]` | Additional volume mounts (see below) |
| `environment` | Map of strings | No | `{}` | Environment variables set in the container |
| `post_create_command` | String | No | -- | Command to run after container creation |
| `vscode_extensions` | Array of strings | No | `[]` | VS Code extensions to install |
| `keepalive` | Boolean | No | `false` | Network keepalive (prevents OrbStack/VM NAT idle dropout) |

#### Extra Volumes

Each entry in `extra_volumes` has:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `source` | String | Yes | -- | Host path |
| `target` | String | Yes | -- | Container path |
| `read_only` | Boolean | No | `false` | Mount as read-only |

### [process]

Process packages control which context files and skills are scaffolded.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `packages` | Array of strings | Yes | `["core"]` | Process packages or presets. Must include at least one. |

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

1. **Process packages** -- skills bundled with the packages listed in `[process].packages`
2. **Addon skills** -- skills recommended by active addons (e.g., `python` addon auto-deploys `python-best-practices`)
3. **Include/exclude** -- manual overrides from this section

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `include` | Array of strings | No | `[]` | Additional skills to deploy beyond those from packages and addons |
| `exclude` | Array of strings | No | `[]` | Skills to remove from the active set |

Core skills (`agent-management`, `owner-profile`) cannot be excluded.

Run `aibox skill list` to see all 84 available skills and their deploy status. See the [Skills Library](../skills/index.md) for the full deployment model.

### [context]

Context system configuration.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `schema_version` | String (semver) | No | `"1.0.0"` | Context schema version |

### [ai]

AI provider configuration. Providers listed here are automatically installed as addons.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `providers` | Array of strings | No | `["claude"]` | AI providers: `claude`, `aider`, `gemini`, `mistral` |

### [appearance]

Color theme configuration. See [Themes](../customization/themes.md) for details and previews.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `theme` | String | No | `"gruvbox-dark"` | Color theme: `gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`, `tokyo-night`, `nord`, `projectious` |
| `prompt` | String | No | `"default"` | Starship preset: `default`, `plain`, `minimal`, `nerd-font`, `pastel`, `bracketed` |

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
