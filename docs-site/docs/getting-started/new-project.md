---
sidebar_position: 2
title: "New Project"
---

# New Project

This guide walks through creating a new project from scratch with aibox.

## Initialize the Project

```bash
mkdir my-app && cd my-app
git init

aibox init my-app --harness claude --addon python
```

The `init` command accepts these options:

| Option | Default | Description |
|--------|---------|-------------|
| `--base` | `debian` | Base image |
| `<NAME>` | Current directory name | Container and hostname |
| `--profile` | `human-dev` | Usage profile: `human-dev` or warning-mode `headless-runner` |
| `--harness` | `claude` | AI harnesses (can be repeated): `claude`, `codex`, `gemini`, `aider`, etc. |
| `--addon` | — | Addon names (can be repeated): `python`, `rust`, `node`, `go`, `latex`, etc. |
| `--theme` | `gruvbox` | Theme family |
| `--processkit-version` | latest available tag | processkit content release to pin |

If you omit options, `aibox init` runs interactively and prompts for each value.

<div class="asciinema" data-cast="/aibox/screencasts/init-demo.cast" data-poster="npt:0" data-fit="width"></div>

## What Gets Created

`aibox init` lays down a **slim project skeleton**: devcontainer files,
config, and an empty `context/` directory. The actual content (skills,
processes, the canonical `AGENTS.md`) is then installed by **processkit** as
the last step of `init`.

```
my-app/
├── aibox.toml                  # Single source of truth (includes [processkit])
├── AGENTS.md                   # Canonical agent entry — rendered from processkit scaffolding
├── CLAUDE.md                   # Thin pointer to AGENTS.md (when [ai.harness.claude] is enabled)
├── .gitignore                  # Generated with language-specific blocks
├── .aibox-version              # Tracks installed CLI version
├── .aibox-home/                # Persistent config (git-ignored)
├── .devcontainer/
│   ├── Dockerfile              # Generated from aibox.toml
│   ├── docker-compose.yml      # Generated — volume mounts, env vars
│   └── devcontainer.json       # Generated — VS Code integration
└── context/
    ├── skills/                 # Editable skill copies — installed by processkit
    ├── processes/              # release, code-review, feature-development, bug-fix
    ├── schemas/                # primitive schemas
    ├── state-machines/         # state machine definitions
    └── templates/
        └── processkit/
            └── v0.26.15/       # Immutable upstream snapshot, used by `aibox apply` for three-way diffs
```

:::tip .aibox-local.toml — secrets and per-developer overrides

`.aibox-local.toml` is added to `.gitignore` by `aibox init`. Use it for API keys and host-specific bind mounts that should not be committed:

```toml
[container.environment]
ANTHROPIC_API_KEY = "sk-ant-..."

[[container.extra_volumes]]
source = "~/.config/gh"
target = "/home/aibox/.config/gh"
```

Shared settings stay in `aibox.toml`; personal secrets go here.

:::

:::tip processkit version

By default, the interactive `aibox init` picker offers `latest` first, then the
10 newest concrete processkit tags. Choosing `latest` writes
`version = "latest"` to `aibox.toml` so `aibox apply` tracks the newest
compatible content. Use `--processkit-version` to pin a specific tag
non-interactively:

```bash
aibox init my-app --processkit-version v0.26.15
```

:::

## The Generated aibox.toml

The scaffolded config file comes with commented documentation for every option:

```toml
# aibox.toml — project configuration for aibox.
# All generated files (.devcontainer/) derive from this file.
# Run `aibox apply` after editing to regenerate.
#
# Full documentation: https://projectious-work.github.io/aibox/docs/reference/configuration

[aibox]
project_name = "my-app"
profile      = "human-dev"

[container]
name     = "my-app"
hostname = "my-app"
# user = "aibox"  # Container user (default: aibox)

[container.image]
release_version = "latest"
base = "debian"

[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "latest"

[processkit.context]
schema_version = "1.0.0"

# Addons install tool sets into the container.
# Run `aibox get addon` to see all available addons.
# [addons.python.tools]
# python = { version = "3.14" }
# uv     = { version = "0.11.19" }

# AI harnesses — controls which AI CLIs/configs are enabled.
[ai]
harnesses = [
  { harness = "claude", enable = true, install = true },
]

[customization]
theme  = "gruvbox"
mode   = "auto"
prompt = "default"
layout = "dev"

# Audio support for PulseAudio bridging (e.g., Claude Code voice).
# Requires host-side PulseAudio setup: run `aibox apply audio`
[audio]
enabled = false
# pulse_server = "tcp:host.docker.internal:4714"
```

After editing, regenerate devcontainer files:

```bash
aibox apply
```

## Build and Start

```bash
aibox apply    # Reconcile config, regenerate files, build image
aibox up       # Start the container and attach via tmux
```

You land in a tmux session with the **dev** layout: a work window with Yazi,
the 1st harness, and a shell, plus optional lazygit, further harness, and shell
windows.

Four layouts are available: **dev** (default), **focus**, **cowork**, and **ai**.
See [Layouts](../customization/layouts.md).

The project root is mounted at `/workspace`. Persistent configuration lives in `.aibox-home/` on the host, mounted into the container automatically.

## VS Code Integration

The generated `devcontainer.json` works with VS Code's Dev Containers extension:

1. Open the project folder in VS Code
2. When prompted, click "Reopen in Container"
3. VS Code builds and starts the container automatically

Both `aibox up` (terminal) and VS Code can use the same container simultaneously.

## Next Steps

- [Explore the base image](../container/base-image.md)
- [Choose the right image addon](../addons/overview.md)
- [Operate the runtime](../container/runtime-operations.md)
- [Understand process packages](../context/process-packages.md)
- [Skills (via processkit)](../skills/index.md)
- [Full CLI reference](../reference/cli-commands.md)
