---
sidebar_position: 1
title: CLI Commands
---

# CLI Commands

aibox uses a small verb/resource grammar. `aibox.toml` is desired state, `aibox apply` reconciles generated files and images, and `aibox up` enters the workspace.

## Global Options

| Option | Environment Variable | Default | Description |
|--------|---------------------|---------|-------------|
| `--config <PATH>` | -- | `./aibox.toml` | Path to configuration file |
| `--log-level <LEVEL>` | `AIBOX_LOG_LEVEL` | `info` | Log verbosity |
| `-y`, `--yes` | -- | -- | Skip confirmation prompts |

## Core Workflow

```bash
aibox init my-app --context managed --harness claude --addon python
aibox apply
aibox up
aibox down
aibox doctor
```

## Command Grammar

```bash
aibox init [NAME] [OPTIONS]
aibox apply [RESOURCE] [NAME] [OPTIONS]
aibox up [OPTIONS]
aibox down
aibox get <RESOURCE> [OPTIONS]
aibox describe <RESOURCE> [NAME] [OPTIONS]
aibox set <TARGET> [VALUE] [EXTRA...] [OPTIONS]
aibox edit <RESOURCE>
aibox reset <RESOURCE> [OPTIONS]
aibox delete <RESOURCE> [NAME] [OPTIONS]
aibox create <RESOURCE> [NAME] [OPTIONS]
aibox self <ACTION> [OPTIONS]
```

## init

Create `aibox.toml`, generated devcontainer files, `.aibox-home/`, context scaffolding, and provider pointer files.

```bash
aibox init
aibox init my-app --context managed
aibox init my-app --addon python --addon infrastructure
aibox init my-app --harness claude --harness codex
aibox init my-app --theme catppuccin-mocha
```

| Option | Default | Description |
|--------|---------|-------------|
| `[NAME]` | Current directory | Project/container name |
| `--base <BASE>` | `debian` | Base image |
| `--context <PKG>` | `managed` | processkit package, repeatable |
| `--harness <NAME>` | `claude` | AI harness, repeatable |
| `--addon <NAME>` | -- | Addon name, repeatable |
| `--theme <THEME>` | `gruvbox-dark` | Runtime UI theme |
| `--processkit-version <TAG>` | latest prompt/default | Pin processkit |

## apply

Reconcile generated project state with `aibox.toml`.

```bash
aibox apply
aibox apply --rebuild
aibox apply --config-only
aibox apply migration MIG-20260430_1200
aibox apply audio
aibox apply env research
```

| Option | Description |
|--------|-------------|
| `--rebuild` | Force full image rebuild |
| `--config-only` | Regenerate files without building the image |
| `--fix-compliance-contract` | Rewrite the processkit compliance block in `AGENTS.md` |
| `--no-container` | Skip runtime probing and image build for CI/nested containers |

## Runtime

```bash
aibox up
aibox up --layout focus
aibox up --apply
aibox down
aibox get runtime
aibox describe runtime
aibox delete runtime
```

`up` starts or creates the workspace container and attaches through Zellij. `down` stops the compose project. `delete runtime` removes the container while preserving project files and `.aibox-home/`.

## Inspecting Resources

`get` is compact and scriptable. `describe` is detailed and human-readable. All list/detail commands support `-o, --format table|json|yaml`; `--output` is accepted as a visible alias.

```bash
aibox get addon
aibox describe addon python
aibox describe addon-catalog -o json
aibox describe image-provenance-policy -o json
aibox describe provider-backends -o json
aibox describe workspace-manifest -o json
aibox get skill
aibox get skill --all --category engineering
aibox describe skill model-recommender-route
aibox get process
aibox describe process release-semver
aibox get migration
aibox get env
aibox describe env
aibox get kit
```

`describe workspace-manifest` emits the read-only
`aibox.workspace-manifest.v0-preview` projection of the current `aibox.toml`.
It is intended for automation while the canonical processkit Artifact schema is
still upstream-owned.

`describe provider-backends` emits the local
`aibox.provider-backends.v0-preview` index of supported AI harness backends,
their addon availability, and MCP registration/permission targets.

`describe image-provenance-policy` emits
`aibox.image-provenance-policy.v0-preview`, including the configured GHCR image
tag or tag template, generated file paths, runtime version markers, selected
addons, and the host-side release phase command template.

## Mutating Resources

`set` changes config. Use `--apply` when you want to reconcile immediately.

```bash
aibox set theme.mode dark --apply
aibox set theme.name tokyo-night --apply --restart-session
aibox set addon python enabled
aibox set addon python disabled --apply
aibox set skill model-recommender-route enabled
aibox set skill pandas-polars disabled --apply
aibox set migration MIG-20260430_1200 in-progress
```

Delete explicit resources:

```bash
aibox delete addon python
aibox delete addon python --apply
aibox delete skill pandas-polars
aibox delete env research --yes
aibox delete migration MIG-20260430_1200 --reason "Not applicable"
```

## Reset And Backup

Project reset is intentionally scoped; there is no bare destructive reset.

```bash
aibox create backup
aibox create backup --dry-run
aibox create backup --output-dir /tmp/my-backup
aibox reset project
aibox reset project --dry-run
aibox reset project --no-backup --yes
```

`reset project` removes aibox-managed project files after stopping the runtime. By default it creates a backup first.

## Diagnostics

```bash
aibox doctor
aibox doctor --integrity
aibox doctor --integrity -o json
aibox doctor audio
aibox doctor security
```

`doctor audio` checks host PulseAudio readiness. `doctor security` runs available dependency/image scanners.

## Self Management

```bash
aibox self update --check
aibox self update --dry-run
aibox self update
aibox self completion bash
aibox self uninstall
aibox self uninstall --purge
```

## Removed Old Grammar

The hard-break CLI redesign removed the old top-level command taxonomy.

| Removed | Use |
|---|---|
| `aibox sync` | `aibox apply` |
| `aibox start` | `aibox up` |
| `aibox stop` | `aibox down` |
| `aibox status` | `aibox get runtime` |
| `aibox remove` / `aibox rm` | `aibox delete runtime` |
| `aibox theme ...` | `aibox set theme.mode ...` or `aibox set theme.name ...` |
| `aibox addon ...` | `aibox get/describe/set/delete addon ...` |
| `aibox kit ...` | `aibox get/describe/set/delete skill/process ...` |
| `aibox migrate ...` | `aibox get/set/apply/delete migration ...` |
| `aibox update` | `aibox self update` |
| `aibox completions` | `aibox self completion` |
| `aibox uninstall` | `aibox self uninstall` |
| `aibox audio check/setup` | `aibox doctor audio` / `aibox apply audio` |
