---
sidebar_position: 2
title: Configuration
---

# Configuration

`aibox.toml` is the single source of truth for an aibox project. All generated files derive from it.

## Full Specification

```toml
[aibox]
project_name = "my-app"               # Human-readable project name
profile      = "human-dev"            # Usage profile: human-dev or headless-runner

[container]
name     = "my-app"                   # Container name
hostname = "my-app"                   # Container hostname
user     = "aibox"                    # Container user (default: aibox)

[container.image]
release_version = "latest"            # aibox image/CLI version, or "latest"
base            = "debian"            # Base image flavor

[container.paths]
devcontainer_json       = ".devcontainer/devcontainer.json"
docker_compose          = ".devcontainer/docker-compose.yml"
docker_compose_override = ".devcontainer/docker-compose.override.yml"
dockerfile              = ".devcontainer/Dockerfile"
dockerfile_local        = ".devcontainer/Dockerfile.local"
local_env               = ".aibox-local.env"

[container.lifecycle]
post_create_command = "npm install"   # Command to run after container creation
keepalive = false                     # Periodic DNS keepalive for idle network timeouts

[container.resource_thresholds]
memory_mib_warn = 4096                # Optional warning limit for cgroup memory usage in MiB
process_count_warn = 400              # Optional warning limit for total live processes; 0 disables
processkit_mcp_python_warn = 50       # Optional warning limit for live Python MCP server processes; 0 disables
oom_kill_warn = 0                     # Optional warning threshold for cgroup OOM kill count

[container.environment]
NODE_ENV = "development"              # Project-wide env vars (non-secret; use .aibox-local.toml for secrets)

[[container.extra_volumes]]
source    = "~/.config/gh"            # Host path (~ expanded)
target    = "/home/aibox/.config/gh"  # Container path
read_only = true

[processkit]
source   = "https://github.com/projectious-work/processkit.git"
version  = "latest"                   # "latest", "unset", or a real tag
src_path = "src"
# branch = "main"                     # Optional; tarball-first, branch as fallback
# release_asset_url_template = "..."  # Optional, for non-GitHub hosts

[processkit.context]
schema_version = "1.0.0"              # Context schema version (semver)

[addons.python.tools]                 # Addon: Python runtime
python = { version = "3.13" }
uv     = { version = "0.11.10" }

[addons.rust.tools]                   # Addon: Rust toolchain
rustc   = { version = "1.87" }
clippy  = {}
rustfmt = {}

[addons.git-ui.tools]                 # Optional: GitHub CLI and lazygit
gh      = {}
lazygit = {}

[ai]
model_providers = ["anthropic"]       # Optional API-key/provider hints

[ai.harness.claude]
enabled = true                        # Generate harness config
install = true                        # Install in-container CLI recipe

[ai.harness.codex]
enabled = true
install = true
version = "latest"                    # Optional CLI version pin

[ai.agents]
canonical     = "AGENTS.md"
provider_mode = "pointer"             # pointer | full

[ai.mcp]
# Team-shared MCP servers merged into all generated MCP client configs
# (see also [[mcp.servers]] in .aibox-local.toml for personal servers)

[[ai.mcp.servers]]
name    = "my-team-tool"              # Unique server name
command = "npx"                       # Executable to run
args    = ["-y", "@acme/team-server"] # Arguments
# [ai.mcp.servers.env]                # Optional environment variables
# API_KEY = "..."

[customization]
theme  = "gruvbox-dark"               # Color theme (7 options)
mode   = "auto"                       # Theme mode: auto, light, dark
prompt = "default"                    # Starship preset (7 options)
layout = "dev"                        # tmux layout (6 options)

[customization.tmux.status]
mode = "powerline"                    # powerline | plain | disabled

[audio]
enabled      = false                  # Enable audio bridging
backend      = "pulseaudio"           # Audio bridge backend
install      = true                   # Install container audio tools
pulse_server = "tcp:host.docker.internal:4714"
```

## Section Reference

### [aibox]

Top-level project metadata.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `project_name` | String | No | `container.name` | Human-readable project name |
| `profile` | String | No | `"human-dev"` | Usage profile: `human-dev` or experimental `headless-runner` |

`profile` is currently a compatibility signal for addon selection and doctor
warnings. It does not change the generated image tag yet; `headless-runner`
is reserved for automation-safe configurations that avoid subscription CLI
tools and interactive desktop helpers.

For automation, `aibox describe workspace-manifest -o json` emits a sorted,
read-only `aibox.workspace-manifest.v0` projection of this file. The projection
uses processkit's canonical `Artifact{kind=workspace-manifest}` kind, while the
machine-readable JSON shape remains aibox-owned until processkit publishes a
more detailed Artifact schema.

`aibox describe provider-backends -o json` similarly emits
`aibox.provider-backends.v0-preview`, an aibox-local index of supported harness
backends, addon availability, and MCP registration/permission targets.
`aibox doctor` uses the same preview model to warn about selected backends that
cannot participate in MCP automation, have no permission projection, or conflict
with the `headless-runner` profile.

`aibox describe image-provenance-policy -o json` emits
`aibox.image-provenance-policy.v0-preview`, which summarizes the configured
GHCR image tag or tag template, generated Dockerfile/Compose files, runtime
version/profile markers, selected addons, and the host-side release phase
command template.

### [container]

Container configuration. Controls the generated `docker-compose.yml` and `Dockerfile`.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | Yes | -- | Container name (used by compose and runtime inspect) |
| `hostname` | String | No | `"aibox"` | Container hostname |
| `user` | String | No | `"aibox"` | Container user |
| `post_create_command` | String | No | -- | Command to run after container creation |
| `keepalive` | Boolean | No | `false` | Periodic DNS keepalive for development environments with idle network timeouts. |
| `environment` | Map (String → String) | No | `{}` | Environment variables injected into the container. Suitable for non-secret project-wide values; use `.aibox-local.toml` for secrets. |
| `extra_volumes` | Array of ExtraVolume | No | `[]` | Additional bind mounts. Each entry has `source`, `target`, and optional `read_only`. |
| `resource_thresholds` | Table | No | see below | Warning thresholds used by `aibox doctor` for cgroup/procfs pressure signals. |

Generated Compose files include a top-level project name, an explicit service
image, and `container_name = [container].name`. Container runtime and Compose
views should therefore show each aibox project under its configured
project/container identity instead of a generic `devcontainer` group.

#### [container.resource_thresholds]

These thresholds are warnings only. They do not stop `aibox up` or fail the
container build; they make `aibox doctor` surface resource pressure before the
operating system starts killing processes.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `memory_mib_warn` | Integer | unset | Warn when cgroup `memory.current` exceeds this many MiB. |
| `process_count_warn` | Integer | `400` | Warn when `/proc` contains more processes than this. Set to `0` to disable. |
| `processkit_mcp_python_warn` | Integer | `50` | Warn when many Python processkit MCP server processes are live. Set to `0` to disable. |
| `oom_kill_warn` | Integer | `0` | Warn when cgroup `memory.events` reports more OOM kills than this. |

#### [[container.extra_volumes]]

Each entry in the `extra_volumes` array is an `ExtraVolume` with these fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `source` | String | Yes | -- | Host path (supports `~` expansion) |
| `target` | String | Yes | -- | Container path where the volume is mounted |
| `read_only` | Boolean | No | `false` | Mount the volume read-only |

Example — mount your GitHub CLI config:

```toml
[[container.extra_volumes]]
source    = "~/.config/gh"
target    = "/home/aibox/.config/gh"
read_only = true
```

:::tip Customizing ports, packages, volumes, and environment variables
Use `Dockerfile.local` for installing additional packages, and `docker-compose.override.yml` for ports and additional services. Both files are scaffolded by `aibox init` and are never overwritten by `aibox apply`.

Environment variables and bind mounts can also be configured directly in `[container.environment]` / `[[container.extra_volumes]]` in `aibox.toml`, or — for secrets and per-developer settings that should not be committed — in [`.aibox-local.toml`](./local-config.md).
:::

### .aibox-local.toml

`.aibox-local.toml` is a personal, gitignored overlay for per-developer settings that should never be committed — API keys, personal bind mounts, and similar secrets. It lives next to `aibox.toml` in the project root and is automatically added to `.gitignore` by `aibox init` and `aibox apply`.

Three sections are supported:

- **`[container.environment]`** — merged on top of `aibox.toml`'s `[container.environment]`. Local values win on conflicts.
- **`[[container.extra_volumes]]`** — appended after any volumes declared in `aibox.toml`.
- **`[[mcp.servers]]`** — personal MCP servers appended to the team MCP servers from `aibox.toml [ai.mcp]`. All sources are merged into each generated MCP client config file.

All other configuration (container name, addons, processkit version, etc.) must remain in `aibox.toml`.

See the dedicated [Local Config reference](./local-config.md) for a full example and merge-behavior details.

### [processkit.context]

Context-system metadata owned by the processkit content layer.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `schema_version` | String (semver) | No | `"1.0.0"` | Context schema version |
| `packages` | Array of strings | No | `["product"]` | Legacy compatibility field. New projects use explicit standard skills in `[skills].enabled`; canonical config rendering omits this field. |

Use `[skills].enabled` and `[skills].disabled` for explicit skill-level overrides.

### [addons]

Addons install language runtimes and tool bundles into the container. AI
harnesses are selected under `[ai]`; aibox may still use internal addon recipes
to install their CLIs.

```toml
[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.11.10" }
```

For interactive Git tooling:

```toml
[addons.git-ui.tools]
gh = {}
lazygit = {}
```

Omit `git-ui` if the project does not need GitHub CLI or lazygit in the
container. The aibox repo may select it for maintenance workflows, but it is
not required for every generated project.

Run `aibox get addon` to see all available addons, or `aibox describe addon <name>` for tool details and supported versions. See the [Addons page](../addons/overview.md) for full documentation.

### [skills]

Controls which skills from processkit are installed into `context/skills/`.
Fresh `aibox.toml` scaffolds list the standard processkit operating skills in
`enabled`. If `enabled` is empty, aibox falls back to installing every skill in
the pinned processkit version minus anything listed in `disabled`.

```toml
[skills]
enabled = ["pk-doctor", "status-briefing"]  # install only these plus core skills
disabled = ["research-with-confidence"]     # omit from the default all-skills set
```

`include` and `exclude` are mutually exclusive: use one or the other, not both.
Both accept skill names (the filename without `.md`). An empty `[skills]` table
(or omitting the section entirely) installs all skills.

See the [Skills page](../skills/index.md) for the full processkit boundary.

### [ai]

AI harness and model-provider configuration. Harnesses listed here participate
in generated agent/MCP config; in-container CLI installation is controlled by
`[ai.harness.<name>]`.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `model_providers` | Array of strings | No | `[]` | Optional API-key/provider hints: `anthropic`, `openai`, `google`, `mistral`. |
| `harness.<name>.enabled` | Boolean | No | `false` unless set or paired with other controls | Include this harness in generated runtime config. Names: `claude`, `codex`, `gemini`, `aider`, `continue`, `cursor`, `copilot`, `opencode`, `hermes`. |
| `harness.<name>.install` | Boolean | No | `true` | Install the matching in-container CLI recipe when available. |
| `harness.<name>.version` | String | No | addon's default | Optional CLI version pin. |

Per-harness install controls live below `[ai.harness.<name>]`:

```toml
[ai.harness.codex]
enabled = true       # include Codex in generated config
install = true       # install the Codex CLI in the container
version = "latest"   # optional; use "latest" or a concrete CLI version
```

Legacy compact harness lists, `providers = [...]`, and `[addons.ai-*.tools]`
inputs are still accepted for compatibility. Use `aibox apply --standardize-config`
to rewrite a schema-clean config into the current canonical shape.

### [processkit]

The **load-bearing** content section. Configures the content source
the project consumes — skills, primitives, processes, package YAMLs, and the
canonical `AGENTS.md` template. The default upstream is the canonical
[projectious-work/processkit](https://github.com/projectious-work/processkit)
repo, but any processkit-compatible source works (forks, self-hosted, private
mirrors).

If `version` is the sentinel `unset`, both `aibox init` and `aibox apply` skip
the processkit fetch entirely. Pin a real tag (e.g. `v0.25.4`) to land the
content. The downloaded tarball is git-tracked under
`context/templates/processkit/<version>/` so derived projects always have the
original to diff against.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `source` | String | No | `https://github.com/projectious-work/processkit.git` | Git URL of the content source. |
| `version` | String | No | `unset` | Semver tag to consume. The sentinel `unset` skips fetching until a real tag is set. |
| `src_path` | String | No | `src` | Subdirectory inside the source repo containing the shippable payload. Auto-detected for flat release-asset tarballs. |
| `branch` | String | No | _(none)_ | Optional branch override for testing pre-release work. Discouraged but supported. |
| `release_asset_url_template` | String | No | _(GitHub-style default)_ | URL template for the release-asset tarball. Placeholders: `{source}` (`.git` stripped), `{version}`, `{org}`, `{name}`. Set this for non-GitHub hosts. |

#### Fetch strategy

The fetcher tries strategies in priority order:

1. **Branch override** (if `branch` is set) — `git clone --branch <name>`.
2. **Release-asset tarball** — downloads a purpose-built `.tar.gz` from
   the URL built from `release_asset_url_template` (or the GitHub-style
   default `{source}/releases/download/{version}/{name}-{version}.tar.gz`).
   When a sibling `<asset>.sha256` file is present, the tarball bytes
   are verified against it before extraction. The verified SHA256 is
   recorded in `aibox.lock` as `release_asset_sha256` for bit-exact
   reproducibility.
3. **Host auto-tarball** — falls back to GitHub / GitLab's auto-generated
   `archive/refs/tags/<version>.tar.gz` when no release asset is
   available.
4. **Git clone** of the tag — last resort for hosts that serve neither
   tarball form (typical for self-hosted git over SSH).

The release-asset path lets producers (processkit and any compatible
content source) ship a smaller, explicit shippable artifact. Consumers
get bit-exact reproducibility for free.

A SHA256 mismatch is a hard error (does NOT fall through), since it
indicates either tampering or a producer bug; both are situations the
user should be told about.

#### Example: consume a Gitea-hosted fork

```toml
[processkit]
source                     = "https://gitea.acme.com/platform/processkit-acme.git"
version                    = "v1.2.0"
release_asset_url_template = "https://gitea.acme.com/{org}/{name}/releases/download/{version}/{name}-{version}.tar.gz"
```

### [ai.mcp]

MCP server definitions and permission configuration. `aibox apply` merges servers from three sources and regenerates all MCP client config files:

1. **Built-in processkit servers** — either the processkit gateway or granular per-skill servers, depending on `[ai.mcp.gateway]`
2. **`aibox.toml [ai.mcp]`** — team-shared servers committed to version control
3. **`.aibox-local.toml [mcp]`** — personal servers, gitignored

Generated files (`.mcp.json`, `.cursor/mcp.json`, `.gemini/settings.json`, `.codex/config.toml`, `.codex/hooks.json`, `.continue/mcpServers/`) are **gitignored**. They are always reproducible from the config sources above and must not be committed — doing so would embed personal server definitions or credentials from `.aibox-local.toml`.

#### processkit Gateway: [ai.mcp.gateway]

When the selected processkit release provides `processkit-gateway`, its stdio
proxy can start the matching localhost daemon on demand. It can replace the
one-process-per-skill MCP topology with a single processkit MCP entry.

```toml
[ai.mcp.gateway]
mode = "auto"          # auto | granular | stdio | daemon-proxy
lazy_catalog = false
host = "127.0.0.1"
port = 8765
path = "/mcp"
```

| Mode | Behavior |
|------|----------|
| `auto` | Register a self-starting `processkit-gateway` stdio proxy when the installed processkit version ships it; otherwise fall back to granular per-skill servers |
| `granular` | Always register one MCP server per processkit skill |
| `stdio` | Register `processkit-gateway` directly as a stdio MCP server |
| `daemon-proxy` | Start a localhost gateway daemon from `devcontainer.json` and register a stdio proxy for harnesses |

The daemon-backed modes are localhost-only. Run `aibox apply` after changing
this section so generated harness configs stay in sync.

#### Server Definitions: [[ai.mcp.servers]]

Each `[[mcp.servers]]` entry has these fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | Yes | -- | Unique server name (used as the key in generated configs) |
| `command` | String | Yes | -- | Executable to run (e.g. `npx`, `/usr/local/bin/my-server`) |
| `args` | Array of strings | No | `[]` | Arguments passed to `command` |
| `env` | Map (String → String) | No | `{}` | Environment variables set when the server process starts |

Example:

```toml
[[ai.mcp.servers]]
name    = "github"
command = "npx"
args    = ["-y", "@modelcontextprotocol/server-github"]
[ai.mcp.servers.env]
GITHUB_TOKEN = "ghp_..."
```

#### Permission Configuration: [ai.mcp.permissions]

Controls which MCP servers harnesses are permitted to use, eliminating repetitive permission prompts. `aibox apply` expands glob patterns into concrete server names and regenerates harness-specific permission files for supported harnesses.

**Global defaults:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `default_mode` | String | No | `"ask"` | Default permission when no explicit pattern matches. Use `"allow"` only when every configured MCP server is trusted. |
| `allow_patterns` | Array of strings | No | `[]` | Glob patterns to auto-allow. Supports server patterns such as `"processkit-*"` and Claude-style aliases such as `"mcp__processkit-*"` or `"mcp__processkit-skill-gate__*"`. |
| `deny_patterns` | Array of strings | No | `[]` | Glob patterns to auto-deny (takes precedence over allow). Use for restricting specific tool families. |

**Per-harness overrides** (optional):

```toml
[ai.mcp.permissions.harness.claude-code]
mode = "allow"              # Override global default if needed
extra_patterns = []         # Add harness-specific patterns

[ai.mcp.permissions.harness.opencode]
mode = "allow"
deny_patterns = []          # Restrict specific tools per harness

[ai.mcp.permissions.harness.codex]
trust_level = "trusted"     # Codex uses project-level trust instead of per-tool lists
```

**Example:**

```toml
[ai.mcp.permissions]
default_mode    = "ask"
allow_patterns  = ["mcp__processkit-*"]
deny_patterns   = ["mcp__processkit-dangerous-admin"]  # Deny a specific pattern if needed

[ai.mcp.permissions.harness.claude-code]
# Use default settings; Claude Code will auto-allow all processkit tools

[ai.mcp.permissions.harness.continue]
# Continue defaults to "ask" for safety; override to "allow" to auto-approve
mode = "allow"
```

:::tip Personal MCP servers
Servers that require personal credentials or are not relevant to all team members belong in `[[mcp.servers]]` in `.aibox-local.toml`, not committed `[[ai.mcp.servers]]`. See [Local Config](./local-config.md).
:::

### [ai.agents]

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
overwrites). When the Claude harness is enabled, it also creates
`CLAUDE.md`, either as a thin pointer (default) or with the full rich
content (`provider_mode = "full"`). Other harnesses (Aider, Gemini,
Codex, Copilot, Continue) use config files rather than markdown
entries and are not affected by this section.

Existing files are never overwritten. If you already have a hand-written
`AGENTS.md` or `CLAUDE.md`, `aibox init` leaves it alone.

### [customization]

Visual and layout configuration. See [Themes](../customization/themes.md) and [Layouts](../customization/layouts.md) for details.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `theme` | String | No | `"gruvbox-dark"` | Color theme: `gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`, `tokyo-night`, `nord`, `projectious` |
| `mode` | String | No | `"auto"` | Global theme mode overlay: `auto`, `light`, `dark`. `auto` preserves the selected concrete theme. |
| `prompt` | String | No | `"default"` | Starship preset: `default`, `plain`, `arrow`, `minimal`, `nerd-font`, `pastel`, `bracketed` |
| `layout` | String | No | `"dev"` | tmux layout: `dev`, `focus`, `cowork`, `cowork-swap`, `browse`, `ai` |
| `tmux.status.mode` | String | No | `"powerline"` | tmux status presentation: `powerline` uses the themed aibox status bar, `plain` keeps minimal tmux text, `disabled` turns the status line off |

### [audio]

Audio bridging configuration.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | Boolean | No | `false` | Enable PulseAudio environment setup |
| `backend` | String | No | `"pulseaudio"` | Audio bridge backend. Only `pulseaudio` is currently supported. |
| `install` | Boolean | No | `true` | Select the internal `audio-voice` recipe when audio is enabled |
| `pulse_server` | String | No | `"tcp:host.docker.internal:4714"` | PulseAudio server address |

Legacy `[container.audio]` input is still accepted for compatibility. Fresh
scaffolding writes top-level `[audio]`.

## Environment Variable Overrides

Some settings can be overridden via environment variables:

| Variable | Overrides | Description |
|----------|-----------|-------------|
| `AIBOX_HOST_ROOT` | `.aibox-home/` path | Host directory for persistent config (default: `.aibox-home/`) |
| `AIBOX_WORKSPACE_DIR` | Workspace mount source | Host directory mounted as `/workspace` |
| `AIBOX_LOG_LEVEL` | `--log-level` | Log verbosity (`trace`, `debug`, `info`, `warn`, `error`) |

Example:

```bash
AIBOX_WORKSPACE_DIR=/home/user/projects/my-app aibox up
```
