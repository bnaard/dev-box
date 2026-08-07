# New Project

LLMS index: [llms.txt](/aibox/v0.x/v0.31.0/llms.txt)

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
| `--context-mode` | `processkit` | Context layer: `processkit` or `harness-only` |
| `--processkit-version` | latest stable tag | processkit content release to pin; explicit prerelease pins are supported |

If you omit options, `aibox init` runs interactively and prompts for each value.

<div class="asciinema" data-cast="/aibox/screencasts/init-demo.cast" data-poster="npt:0" data-fit="width"></div>

## What Gets Created

By default, `aibox init` lays down a **processkit-backed project skeleton**:
devcontainer files, config, an empty `context/` directory, and processkit
content (skills, processes, and the canonical `AGENTS.md`).

```
my-app/
├── aibox.toml                  # Single source of truth (includes [processkit])
├── AGENTS.md                   # Canonical agent entry — rendered from processkit scaffolding
├── CLAUDE.md                   # Thin pointer to AGENTS.md (when Claude is enabled in [ai].harnesses)
├── .gitignore                  # Generated with language-specific blocks
├── aibox.lock                  # Records resolved CLI, image, addon, and processkit state
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
            └── v0.27.4/        # Immutable upstream snapshot, used by `aibox apply` for three-way diffs
```

For projects that only want the generated devcontainer and AI harness setup,
use harness-only mode:

```bash
aibox init my-app --context-mode harness-only --harness claude
```

Harness-only projects still get `aibox.toml`, `.devcontainer/`, `.aibox-home/`,
selected harness config, `AGENTS.md`, and provider pointer files such as
`CLAUDE.md`. They do **not** get processkit content, `context/skills/`,
`context/templates/processkit/`, processkit MCP gateway config, processkit
hooks/preauth, processkit command adapters, or processkit Migration entities.
The minimal generated `AGENTS.md` contains no processkit references.

<div class="alert alert-success" role="alert"><div class="h4 alert-heading" role="heading">.aibox-local.toml — secrets and per-developer overrides</div>



`.aibox-local.toml` is added to `.gitignore` by `aibox init`. Use it for API keys and host-specific bind mounts that should not be committed:

```toml
[container.environment]
ANTHROPIC_API_KEY = "sk-ant-..."
GH_TOKEN = "github_pat_project_scoped_token"
```

Shared settings stay in `aibox.toml`; personal secrets go here.
Use narrowly scoped PATs by default. For the alternative persistent
`gh auth login` flow and multi-token commands, see
[GitHub authentication](../reference/local-config.md#github-authentication).

</div>


<div class="alert alert-success" role="alert"><div class="h4 alert-heading" role="heading">processkit version</div>



By default, the interactive `aibox init` picker offers `latest` first, then the
10 newest stable processkit tags. Choosing `latest` writes
`version = "latest"` to `aibox.toml` so `aibox apply` tracks the newest
compatible content. Use `--processkit-version` to pin a specific tag
non-interactively:

```bash
aibox init my-app --processkit-version v0.27.4
```

To evaluate prerelease processkit content without changing the stable default,
pin the prerelease explicitly or opt into prerelease selection:

```sh
aibox init my-app --processkit-version v1.0.0-alpha.1
aibox init my-app --include-prerelease
```

This picker is skipped when `--context-mode harness-only` is selected.

</div>


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

[context]
mode = "processkit"
packages = ["product"]

[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "latest"

[processkit.context]
schema_version = "1.0.0"

# Addons install tool sets into the container.
# Run `aibox get addon` to see all available addons.
# [addons.python.tools]
# python = { version = "3.14" }
# uv     = { version = "0.12.0" }

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
