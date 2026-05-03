---
sidebar_position: 1
title: "Context Overview"
---

# Context System Overview

The aibox context system gives AI agents the structured, file-based information
they need to work effectively on your project — backlog, decisions, standups,
processes, primitives, and skills.

As of **v0.16.0**, the system is split across two cleanly separated projects:

- **aibox** owns the **container** — devcontainers, addons, the CLI, the
  install/apply/migrate machinery, and the slim project skeleton (`.aibox-version`,
  `.gitignore`, an empty `context/`, and a thin `CLAUDE.md` pointer).
- **[processkit](https://github.com/projectious-work/processkit)** owns the
  **content** — every skill, every primitive schema, every state machine, the
  canonical `AGENTS.md` template, the processes, and the package YAMLs that
  compose them.
- **The user-side `context/` directory is shared territory.** aibox creates it,
  processkit fills it, and the user edits in place. An immutable upstream
  snapshot is kept under `context/templates/processkit/<version>/` for the
  three-way diff that `aibox apply` uses to detect upstream changes versus
  local edits.

## The Problem

AI coding agents like Claude operate best when they understand not just the
code, but the project's goals, decisions, and current state. Without structure,
this information ends up scattered across chat histories, stale comments, and
the developer's memory.

A single root-level instructions file is not enough for non-trivial projects.
It works well for instructions and preferences, but it does not provide a
standard place for decisions, backlog, progress tracking, or team conventions.

## How Context Files Work

After `aibox init` and `aibox apply` (with a real `[processkit].version` pinned),
your project looks something like this:

```
my-project/
├── AGENTS.md                       # Canonical agent entry — rendered from processkit scaffolding
├── CLAUDE.md                       # Thin pointer to AGENTS.md (provider entry file)
├── aibox.toml
├── .devcontainer/
└── context/
    ├── skills/                     # Editable skill copies
    ├── processes/                  # release, code-review, feature-development, bug-fix
    ├── schemas/                    # primitive schemas
    ├── state-machines/             # state machine definitions
    └── templates/
        └── processkit/
            └── v0.25.0/            # Immutable upstream snapshot, base of three-way diffs
```

### AGENTS.md, CLAUDE.md, and provider files

`AGENTS.md` at the project root is the **canonical** agent entry document. It
is rendered from the processkit template (`AGENTS.md` at the processkit tarball root)
during `aibox init` (write-if-missing — never overwrites). The
[agents.md](https://agents.md/) ecosystem convention is to read this file from
any AI harness.

When `[ai].providers` includes `claude`, aibox also writes a thin `CLAUDE.md`
at the project root that just points at `AGENTS.md`. **No content is written
under `.claude/skills/` or any other provider-specific directory** as of
v0.16.0. Other providers (Aider, Gemini, Mistral) use config files
(`.aider.conf.yml`, `.gemini/settings.json`, `.mistral/config.json`) which are
scaffolded by the addon system.

## OWNER.md — Developer Identity

`OWNER.md` captures the developer's identity and preferences. It is created
during `aibox init` (or by the `owner-profile` skill the first time the agent
asks), with fields that help AI agents understand who they are working with:

- **Name** — how the developer prefers to be addressed
- **Domain expertise** — areas of knowledge and experience
- **Primary languages** — programming languages used most often
- **Communication language** — natural language for responses (e.g., English, German)
- **Timezone** — for scheduling and availability context
- **Working hours** — typical availability window
- **Current focus** — what the developer is currently working on or learning
- **Communication preferences** — style and conventions for AI interactions

## Skills and Processes

Skills and processes are owned by processkit. For full documentation on what's
available, how skills are organised, and which packages to use, see:

- [Skills](../skills/index.md) — how skills install and how to browse them
- [Process Packages](process-packages.md) — package tiers and selection
- [processkit on GitHub](https://github.com/projectious-work/processkit)

## Version Tracking

Two pieces track the version:

```toml
[aibox]
version = "0.23.0"

[context]
schema_version = "1.0.0"

[processkit]
version = "v0.25.0"
```

When the schema evolves, `aibox doctor` flags version mismatches and `aibox apply`
runs the relevant migrations. See [Migration](migration.md) for details.

## Relationship to aibox.toml

The `[context]` section in `aibox.toml` declares which processkit packages are
in scope. The `[processkit]` section pins which version of the content
repository this project consumes:

```toml
[context]
packages = ["managed"]

[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "v0.25.0"
```

Run `aibox apply` after editing `[processkit].version` to pull a new release.

## Design Principles

**Convention over configuration.** File names and locations are standardised
so AI agents can find them without special instructions.

**Human-readable first.** All context files are Markdown. They are useful
without any tooling.

**Editable in place.** Everything under `context/skills/`, `context/processes/`,
`context/schemas/`, and `context/state-machines/` is yours to edit. The
immutable snapshot under `context/templates/processkit/<version>/` exists only
as the base of `aibox apply`'s three-way diff.

**No lock-in.** Context files are plain Markdown and YAML in a `context/`
directory. Stop using aibox and the files remain useful.

**Clean boundary between container and content.** aibox owns the box;
processkit owns what goes in it. Each ships on its own cadence.
