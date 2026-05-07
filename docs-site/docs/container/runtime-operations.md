---
sidebar_position: 5
title: "Runtime Operations"
description: "How to start, inspect, rebuild, and troubleshoot a running aibox workspace."
---

# Runtime Operations

The generated devcontainer is standard Compose output, but the normal workflow
should go through aibox so generated files, runtime home state, and diagnostics
stay aligned.

## Start and Attach

```bash
aibox up
aibox up --layout focus
aibox up --apply
```

`aibox up` creates or starts the container and attaches through tmux. Use
`--layout` for a one-session layout override. Use `--apply` when you want aibox
to reconcile configuration before starting.

## Stop or Remove

```bash
aibox down
aibox delete runtime
```

`down` stops the Compose project. `delete runtime` removes the container while
preserving project files and `.aibox-home/`.

## Rebuild

```bash
aibox apply
aibox apply --no-cache
aibox apply --config-only
```

Use `--no-cache` after base-image, addon, or package-cache issues. `--rebuild`
is kept as a visible alias for the same behavior. Use `--config-only` when you
only want to regenerate files and refresh processkit content without building
the image.

## Inspect Runtime State

```bash
aibox get runtime
aibox get runtime --resources
aibox get runtime --resources -o json
aibox describe runtime
```

The resource snapshot is designed for low-dependency environments. It reads
cgroupfs and procfs directly instead of relying on tools such as `ps` or `free`
being installed.

Key fields:

| Field | What it tells you |
| --- | --- |
| `memory_current_bytes` | current cgroup memory usage |
| `memory_max` | memory limit or `unlimited` |
| `oom_kill_count` | whether the kernel has killed a process in the cgroup |
| `total_process_count` | total visible processes |
| `processkit_mcp_python_process_count` | live Python processkit MCP server processes |

An `oom_kill_count` above zero is strong evidence that a missing agent or
terminated tool was killed by the operating system rather than by the CLI.

With processkit v0.25.4, `[ai.mcp.gateway].mode = "auto"` registers a
self-starting `processkit-gateway` stdio proxy for MCP-capable harnesses. The
proxy starts the local daemon on demand when no listener exists, so generated
devcontainer startup no longer has to supervise the gateway in the default mode.
Use `granular` only when a harness needs the older one-server-per-skill layout.

## Resource Thresholds

Configure warning thresholds in `aibox.toml`:

```toml
[container.resource_thresholds]
memory_warn_percent = 80
process_count_warn = 400
processkit_mcp_python_warn = 50
```

`aibox doctor` uses these values when reporting runtime pressure.

## Compose Identity

Generated Compose output includes:

- a top-level project name derived from `[container].name`
- an explicit image name
- a main service named after the project
- `container_name = [container].name`
- `init: true` for PID 1 process reaping

This keeps Docker Desktop, OrbStack, and Compose UIs from grouping unrelated
aibox projects under a generic `devcontainer` identity.

## Existing Containers After Generator Changes

`aibox apply` rewrites generated files, but it does not magically replace a
running container that was created from older Compose output. Recreate the
runtime when a change affects process model, mounts, image labels, init
behavior, or service identity:

```bash
aibox down
aibox apply
aibox up
```

If a container has already accumulated zombie processes, the fix is still a
runtime recreate. A new generated `init: true` service can reap future orphaned
children, but it cannot change PID 1 in an already-created container.

## Common Symptoms

| Symptom | First checks |
| --- | --- |
| AI process disappears | `aibox get runtime --resources`, then inspect `oom_kill_count` |
| OrbStack groups projects oddly | rerun `aibox apply`, then recreate the container |
| Network drops after idle time | set `[container] keepalive = true` |
| Build keeps using stale layers | run `aibox apply --no-cache` |
| Runtime starts but tools are missing | check selected addons with `aibox describe workspace-manifest` |

## Podman Notes

Podman support depends on the Compose provider behind `podman compose`.
Generated files follow the Compose Specification, including `init: true`.
If an older provider rejects a spec key, upgrade the provider instead of
removing the generated setting.
