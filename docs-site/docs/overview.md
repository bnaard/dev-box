---
sidebar_position: 1
title: "Overview"
description: "What aibox does, what it owns, and when to use it."
---

# Overview

aibox creates reproducible, AI-ready development workspaces from one project
configuration file. It is not a new container runtime and it is not a process
framework. It is the glue that turns a declared project shape into a standard
devcontainer, selected tool bundles, AI harness configuration, and a pinned
processkit context layer.

## The Short Version

```bash
aibox init my-app --context managed --harness claude --addon python
aibox apply
aibox up
```

`aibox init` writes the initial project contract. `aibox apply` reconciles that
contract into generated files and content. `aibox up` starts or attaches to the
workspace.

## What aibox Owns

| Area | Output |
| --- | --- |
| Project contract | `aibox.toml`, `aibox.lock`, `.aibox-version` |
| Devcontainer | `.devcontainer/Dockerfile`, Compose files, `devcontainer.json` |
| Runtime home | `.aibox-home/` with Zellij, shell, prompt, theme, and tool config |
| Addons | tool and runtime selection from `addons/` YAML definitions |
| Harness wiring | provider entry files, MCP registration, permissions, and runtime tabs |
| Diagnostics | `aibox doctor`, `aibox get runtime`, migration and integrity checks |

The generated files use common formats on purpose. You can inspect them, run
Docker or Podman commands against them, and use the same project in VS Code Dev
Containers when that is useful.

## What processkit Owns

processkit owns the project-process content:

- skills and `SKILL.md` files
- schemas and state machines
- work processes
- package definitions such as `managed`, `software`, `research`, and `product`
- the canonical `AGENTS.md` template

aibox installs processkit content into `context/`, keeps an immutable upstream
snapshot under `context/templates/processkit/<version>/`, and uses that snapshot
for three-way diff and migration workflows. The content itself remains
processkit-owned.

## Why This Split Matters

The split keeps the system forkable and maintainable:

- aibox can improve containers, addons, and runtime operations without changing
  process semantics.
- processkit can improve skills, primitives, and workflows without shipping a
  new container tool.
- projects can pin or fork processkit independently through `[processkit]` in
  `aibox.toml`.

## When To Use aibox

Use aibox when you want:

- a reproducible terminal-first workspace for AI-assisted development
- selected AI harnesses and tool bundles declared in one file
- project context on disk instead of only in chat history
- consistent Zellij layouts, themes, shell tooling, and runtime diagnostics
- a clean handoff path between different agents and human contributors

Do not use aibox as a general infrastructure deployer. It manages development
workspaces. Production deployment, remote host provisioning, and service
orchestration belong in dedicated infrastructure tooling.

## Daily Mental Model

`aibox.toml` is desired state.

Run `aibox apply` after changing desired state. It regenerates managed files,
refreshes processkit content, updates the lock file, and builds the image unless
you ask it not to.

Run `aibox up` to enter the workspace. It starts the Compose project and
attaches through Zellij.

Run `aibox doctor` when the environment looks wrong. Run
`aibox get runtime --resources` when the workspace feels slow or agents exit
without a clean error.
