---
name: runtime-prune
description: >
  Inspect, plan, and invoke safe cleanup for aibox-managed runtime state
  without binding the workflow to a specific agent provider.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v2
    id: SKILL-runtime-prune
    version: "1.0.0"
    created: 2026-05-13T00:00:00Z
    category: processkit
    layer: 2
    uses:
      - skill: skill-gate
        purpose: >
          Require explicit routing and confirmation before destructive
          cleanup.
    provides:
      primitives: []
      mcp_tools:
        - analyze_disk_usage
        - plan_prune
        - apply_prune
      assets: []
---

# Runtime Prune

## Intro

`runtime-prune` gives agents a provider-neutral cleanup workflow for
aibox-managed runtime state. It inventories selected cleanup scopes,
produces a dry-run plan, and invokes `aibox prune` for the destructive
path when that command is available.

## Overview

Use this skill when a project has accumulated runtime-home caches,
build caches, agent worktree state, container storage, or E2E companion
state and the user wants to inspect or reclaim disk space safely.

The workflow is:

1. Call `analyze_disk_usage(project_root, scopes)` to inventory
   configured cleanup scopes without modifying the filesystem.
2. Call `plan_prune(project_root, scopes)` to produce a dry-run plan,
   expected reclaimed bytes, risk labels, and the exact confirmation
   token needed for apply.
3. Call `apply_prune(project_root, scopes, confirmation)` only after
   the user has explicitly approved the plan. The apply tool delegates
   to `aibox prune`; it does not duplicate destructive provider cleanup
   logic inside processkit.

### MCP Tools

| Tool | Purpose |
|---|---|
| `analyze_disk_usage(project_root, scopes)` | Return structured size and risk data for selected cleanup scopes. |
| `plan_prune(project_root, scopes)` | Return a dry-run prune plan and required confirmation token. |
| `apply_prune(project_root, scopes, confirmation)` | Invoke `aibox prune` with approved scopes and return command evidence. |

### Scopes

| Scope | What it covers | Apply policy |
|---|---|---|
| `runtime-home` | Managed `.aibox-home` and bounded `.aibox` runtime cache or diagnostics paths. | Delegated to `aibox prune`. |
| `build-cache` | Explicit build-cache paths such as Rust incremental and build subdirectories. | Delegated to `aibox prune`. |
| `agent-worktrees` | Provider-created or aibox-managed Git worktrees discovered from Git metadata. | Delegated to `aibox prune` after explicit confirmation. |
| `containers` | aibox-owned container cleanup. | Delegated to `aibox prune`; no direct Docker or Podman calls. |
| `e2e-companion` | Nested companion environment cleanup. | Delegated to `aibox prune`; companion access must be via SSH reachability. |

## Gotchas

- **Do not treat analysis as approval.** `analyze_disk_usage()` and
  `plan_prune()` are read-only. Apply still needs the exact confirmation
  token returned by the plan.
- **Do not hand-roll destructive provider cleanup.** If `aibox prune`
  is unavailable, `apply_prune()` must return an error instead of
  deleting worktrees, containers, or caches itself.
- **Do not inspect local Docker or Podman for E2E companion cleanup.**
  Companion state is remote from the devcontainer's perspective; use
  the aibox workflow that checks SSH reachability.
- **Do not count arbitrary project files as reclaimable.** Only
  explicitly scoped runtime and cache paths contribute to expected
  reclaimed bytes.
- **Do not delete provider worktrees by path heuristics.** Worktrees
  are inventoried for visibility, but deletion is delegated to aibox
  because it owns provider-specific safety checks.
- **Keep paths provider-neutral in user-facing output.** Report scopes
  like `agent-worktrees` and `runtime-home`, not Claude-, Codex-, or
  shell-specific state unless a discovered path itself contains that
  name.

## Full reference

`analyze_disk_usage()` resolves `project_root` to an existing directory
and defaults to all scopes when none are supplied. It never follows
symlinks during size walks.

`plan_prune()` returns:

- `required_confirmation`, formatted as
  `apply-prune:<comma-separated-scopes>`
- per-scope actions with `risk`, `expected_reclaim_bytes`, `targets`,
  `dry_run_command`, and `apply_command`
- whether `aibox prune` is currently available on `PATH`

`apply_prune()` accepts only the exact `required_confirmation` value
from the matching plan. If the token matches and `aibox prune` is on
`PATH`, it runs:

```sh
aibox prune --scope <scope> ... --yes --json
```

The tool records stdout, stderr, exit code, and the command arguments in
the returned evidence. A non-zero `aibox prune` exit code is returned to
the caller as structured evidence instead of being hidden.
