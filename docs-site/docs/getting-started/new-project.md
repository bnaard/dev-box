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

aibox init --name my-app --process managed
```

The `init` command accepts these options:

| Option | Default | Description |
|--------|---------|-------------|
| `--name` | Current directory name | Container and hostname |
| `--base` | `debian` | Base image |
| `--process` | `managed` | processkit package(s): `minimal`, `managed`, `software`, `research`, `product` (can be repeated) |
| `--ai` | `claude` | AI providers (can be repeated): `claude`, `aider`, `gemini`, `mistral` |
| `--addons` | — | Addon names (can be repeated): `python`, `rust`, `node`, `go`, `latex`, etc. |
| `--theme` | `gruvbox-dark` | Color theme |

If you omit options, `aibox init` runs interactively and prompts for each value.

<div class="asciinema" data-cast="/aibox/screencasts/init-demo.cast" data-poster="npt:0" data-fit="width"></div>

## What Gets Created

`aibox init` lays down a **slim project skeleton** — devcontainer files,
config, and an empty `context/` directory. The actual content (skills,
processes, the canonical `AGENTS.md`) is then installed by **processkit** as
the last step of `init`.

```
my-app/
├── aibox.toml                  # Single source of truth (includes [processkit])
├── AGENTS.md                   # Canonical agent entry — rendered from processkit scaffolding
├── CLAUDE.md                   # Thin pointer to AGENTS.md (when [ai].providers includes "claude")
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
            └── v0.8.0/         # Immutable upstream snapshot, used by `aibox sync` for three-way diffs
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

By default, `aibox init` interactively picks the latest processkit version and pins it in `aibox.toml`. Use `--processkit-version` to pin a specific tag non-interactively:

```bash
aibox init --name my-app --processkit-version v0.8.0
```

:::

## The Generated aibox.toml

The scaffolded config file comes with commented documentation for every option:

```toml
# aibox.toml — project configuration for aibox.
# All generated files (.devcontainer/) derive from this file.
# Run `aibox sync` after editing to regenerate.
#
# Full documentation: https://projectious-work.github.io/aibox/docs/reference/configuration

[aibox]
version = "0.17.5"
base    = "debian"

[container]
name     = "my-app"
hostname = "my-app"
# user = "aibox"  # Container user (default: aibox)

[context]
schema_version = "1.0.0"
# processkit packages: minimal, managed (default), software, research, product
packages = ["managed"]

[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "v0.8.0"

# Addons install tool sets into the container.
# Run `aibox addon list` to see all available addons.
# [addons.python.tools]
# python = { version = "3.13" }
# uv     = { version = "0.7" }

# AI providers — controls which AI CLI tools are installed.
# Options: claude, aider, gemini, mistral
[ai]
providers = ["claude"]

[customization]
theme  = "gruvbox-dark"
prompt = "default"
layout = "dev"

# Audio support for PulseAudio bridging (e.g., Claude Code voice).
# Requires host-side PulseAudio setup: run `aibox audio setup`
[audio]
enabled = false
# pulse_server = "tcp:host.docker.internal:4714"
```

After editing, regenerate devcontainer files:

```bash
aibox sync
```

## Build and Start

```bash
aibox sync     # Reconcile config, regenerate files, build image
aibox start    # Start the container and attach via Zellij
```

You land in a Zellij session with the **dev** layout: Yazi file browser (40%) and Vim editor (60%) side by side, plus tabs for lazygit and shell.

Six layouts are available: **dev** (default), **focus** (one tool per tab, fullscreen), **cowork** (Yazi+Vim left, Claude right), **cowork-swap**, **browse**, and **ai**. See [Layouts](../container/base-image.md#layouts).

The project root is mounted at `/workspace`. Persistent configuration lives in `.aibox-home/` on the host, mounted into the container automatically.

## VS Code Integration

The generated `devcontainer.json` works with VS Code's Dev Containers extension:

1. Open the project folder in VS Code
2. When prompted, click "Reopen in Container"
3. VS Code builds and starts the container automatically

Both `aibox start` (terminal) and VS Code can use the same container simultaneously.

## Next Steps

- [Explore the base image](../container/base-image.md)
- [Choose the right image addon](../addons/overview.md)
- [Understand process packages](../context/process-packages.md)
- [Skills (via processkit)](../skills/index.md)
- [Full CLI reference](../reference/cli-commands.md)
