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
aibox init runner --profile headless-runner
aibox init my-app --addon python --addon infrastructure
aibox init my-app --harness claude --harness codex
aibox init my-app --theme catppuccin-mocha
```

| Option | Default | Description |
|--------|---------|-------------|
| `[NAME]` | Current directory | Project/container name |
| `--base <BASE>` | `debian` | Base image |
| `--profile <PROFILE>` | `human-dev` | Usage profile: `human-dev` or warning-mode `headless-runner` |
| `--context <PKG>` | `managed` | processkit package, repeatable |
| `--harness <NAME>` | `claude` | AI harness, repeatable |
| `--addon <NAME>` | -- | Addon name, repeatable |
| `--theme <THEME>` | `gruvbox-dark` | Runtime UI theme |
| `--processkit-version <TAG>` | latest prompt/default | Pin processkit |

## apply

Reconcile generated project state with `aibox.toml`.

```bash
aibox apply
aibox apply --no-cache
aibox apply --rebuild
aibox apply --config-only
aibox apply migration MIG-20260430_1200
aibox apply audio
aibox apply env research
```

| Option | Description |
|--------|-------------|
| `--no-cache` | Force a full image rebuild without using cached layers |
| `--rebuild` | Visible alias for `--no-cache` |
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
aibox get runtime --resources
aibox get runtime --resources -o json
aibox describe runtime
aibox delete runtime
```

`up` starts or creates the workspace container and attaches through Zellij. `down` stops the compose project. `delete runtime` removes the container while preserving project files and `.aibox-home/`.

`get runtime` reports the configured container name and detected state
(`running`, `stopped`, or `missing`). With `--resources`, it reports a
best-effort resource pressure snapshot from the current Linux runtime by
reading cgroupfs and procfs directly rather than shelling out to `ps` or
`free`.

`aibox get runtime --resources` includes:

| Field | Source | Meaning |
|-------|--------|---------|
| `memory_current_bytes` | `/sys/fs/cgroup/memory.current` | Current cgroup memory usage in bytes |
| `memory_max` | `/sys/fs/cgroup/memory.max` | Cgroup memory limit, or `unlimited` when the kernel reports `max` |
| `oom_kill_count` | `/sys/fs/cgroup/memory.events` | Cumulative `oom_kill` counter for the cgroup |
| `total_process_count` | numeric entries under `/proc` | Total visible process count |
| `processkit_mcp_python_process_count` | `/proc/*/cmdline` | Python processes whose command line looks like a processkit MCP server |

The default table output is compact and human-readable. `-o json` and
`-o yaml` emit the same field names for automation; unavailable cgroup values
are `null`, while process counts fall back to `0` if `/proc` cannot be read.

## Inspecting Resources

`get` is compact and scriptable. `describe` is detailed and human-readable. All list/detail commands support `-o, --format table|json|yaml`; `--output` is accepted as a visible alias.

```bash
aibox get addon
aibox describe addon python
aibox describe addon-catalog -o json
aibox describe image-provenance-policy -o json
aibox get runtime --resources -o json
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

### Preview projections

These `describe` resources are stable enough for local automation. The
workspace manifest has been promoted to `aibox.workspace-manifest.v0` because
processkit now recognizes `Artifact{kind=workspace-manifest}`; the other
environment-contract projections remain `aibox.*.v0-preview` until processkit
publishes more detailed canonical Artifact schemas.

| Command | Schema | Contents |
|---------|--------|----------|
| `describe addon-catalog` | `aibox.addon-catalog.v0` | Built-in addon metadata, profile intent, automation usage class, exported surfaces, dependencies, and tool versions |
| `describe workspace-manifest` | `aibox.workspace-manifest.v0` | Sorted projection of `aibox.toml`: project, processkit source, context packages, AI harnesses, addons, and MCP server keys |
| `describe provider-backends` | `aibox.provider-backends.v0-preview` | Supported AI harness backends, selected status, addon availability, MCP config targets, and permission targets |
| `describe image-provenance-policy` | `aibox.image-provenance-policy.v0-preview` | GHCR image tag or tag template, generated file paths, runtime version/profile markers, selected addons, and release phase commands |

The preview projections do not create processkit entities and should not be
treated as canonical processkit Artifacts yet.

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
aibox reset context --from-processkit v0.25.0 --dry-run
```

`reset project` removes aibox-managed project files after stopping the runtime. By default it creates a backup first.

`reset context` is plan-only in this release. It reports the processkit-owned
context paths that a hard reset would replace from a selected processkit
baseline and the project-owned context paths that must be preserved or reviewed.
Use normal migrations unless the owner explicitly approves a hard context reset.

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
