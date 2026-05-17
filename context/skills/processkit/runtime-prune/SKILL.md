---
name: runtime-prune
description: >
  Inspect, plan, and invoke safe cleanup for runtime-manager-owned state
  without binding the workflow to a specific host orchestrator or agent
  provider.
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
runtime-manager-owned state. It inventories selected cleanup scopes,
produces a dry-run plan, applies low-risk processkit-owned cleanup from
an explicit allowlist, and returns structured host-action evidence for
high-risk runtime-owner cleanup. It does not invoke host orchestrator
commands from inside the processkit runtime.

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
   the user has explicitly approved the plan. The apply tool may remove
   only explicit processkit-owned allowlist targets; provider/runtime
   owner cleanup returns an external host-action result for the owner.

### MCP Tools

| Tool | Purpose |
|---|---|
| `analyze_disk_usage(project_root, scopes)` | Return structured size and risk data for selected cleanup scopes. |
| `plan_prune(project_root, scopes)` | Return a dry-run prune plan and required confirmation token. |
| `apply_prune(project_root, scopes, confirmation)` | Apply processkit allowlist cleanup and return external host-action evidence for unsupported scopes. |

### Scopes

| Scope | What it covers | Apply policy |
|---|---|---|
| `runtime-home` | Bounded runtime-home cache, diagnostics, tmp, and runtime paths. | Processkit allowlist cleanup. |
| `build-cache` | Explicit build-cache paths such as Rust incremental and build subdirectories. | Processkit allowlist cleanup. |
| `agent-worktrees` | Provider-created or runtime-manager-owned Git worktrees discovered from Git metadata. | External host action after explicit confirmation. |
| `containers` | Host runtime container cleanup. | External host action; no direct Docker or Podman calls. |
| `e2e-companion` | Nested companion environment cleanup. | External host action; companion access must use owner-approved reachability checks. |

## Gotchas

- **Do not treat analysis as approval.** `analyze_disk_usage()` and
  `plan_prune()` are read-only. Apply still needs the exact confirmation
  token returned by the plan.
- **Do not hand-roll destructive provider cleanup.** `apply_prune()` may
  clean processkit-owned allowlist targets, but worktrees, containers,
  and companion state must return structured external host-action
  evidence instead of invoking a host orchestrator command.
- **Do not remove the whole runtime home.** `runtime-home` targets only
  narrow cache, diagnostics, tmp, and runtime subpaths, not the whole
  `.aibox-home` tree.
- **Do not inspect local Docker or Podman for E2E companion cleanup.**
  Companion state is remote from the devcontainer's perspective; use
  the owner-approved host workflow that checks reachability.
- **Do not count arbitrary project files as reclaimable.** Only
  explicitly scoped runtime and cache paths contribute to expected
  reclaimed bytes.
- **Do not delete provider worktrees by path heuristics.** Worktrees
  are inventoried for visibility, but deletion is delegated to the host
  runtime manager because it owns provider-specific safety checks.
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
- per-scope actions with `risk`, `apply_owner`,
  `expected_reclaim_bytes`, `targets`, `dry_run_command`, and
  `apply_command`

`apply_prune()` accepts only the exact `required_confirmation` value
from the matching plan. For `runtime-home` and `build-cache`, it removes
only the allowlisted paths from the plan. For external scopes, it records
per-scope host-action evidence and leaves execution to the owner outside
the container.
