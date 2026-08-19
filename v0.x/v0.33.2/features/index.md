# Features

> What aibox provides.

---

LLMS index: [llms.txt](/aibox/v0.x/v0.33.2/llms.txt)

---

# Features

## Single Project Contract

`aibox.toml` is the source of truth for the workspace: base image, container
identity, addons, AI harnesses, theme, layout, runtime thresholds, and
processkit source and version.

## Standard Devcontainer Output

aibox generates Dockerfile, Compose, override, and devcontainer JSON files.
The output remains readable and compatible with Docker, Podman, OrbStack, and
VS Code Dev Containers.

## Composable Addons

Select language runtimes, AI CLIs, git tools, preview utilities,
documentation frameworks, and infrastructure tools without forcing them into
every container.

## processkit Context Integration

processkit owns skills, processes, schemas, state machines, packages, and the
canonical `AGENTS.md` template. aibox pins, installs, and updates that content
under `context/`.

## Provider-Neutral AI Harnesses

AI harnesses and their MCP configuration are selected declaratively.
Provider-specific entry files stay thin while durable context remains local to
the project.

## Runtime Operations

`aibox get runtime --resources` and `aibox doctor` report memory pressure, OOM
kill counters, process counts, generated Compose posture, and selected runtime
settings.

## Migration System

When generated content changes, aibox preserves local edits, keeps upstream
snapshots, and emits migration documents for changes requiring review.
