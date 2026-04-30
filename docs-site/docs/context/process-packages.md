---
sidebar_position: 2
title: "Process Packages"
---

# Process Packages

Process packages are **upstream-defined skill bundles** owned by
[processkit](https://github.com/projectious-work/processkit). They live as YAML
files under `packages/` in the processkit repository and compose via an `extends:`
field, so each package can build on the one below it without restating its contents.

aibox itself does not define packages — it only consumes them. The package you
pick in `aibox.toml`'s `[context].packages` controls which skills are installed
in your project.

```toml
[context]
packages = ["managed"]   # or "minimal", "software", "research", "product"
```

## Packages

processkit ships five packages. For the exact skill composition of each, see:

- `context/templates/processkit/<version>/.processkit/packages/` in your project (after `aibox apply`)
- https://github.com/projectious-work/processkit/releases

> **processkit documentation** is not yet deployed as a standalone site.
> Until then, the package YAML files are the canonical source of truth.

The five package names are:

| Package | Best for |
|---------|----------|
| `minimal` | Scripts, experiments, small utilities |
| `managed` | Recommended default — backlog, decisions, standups, session management |
| `software` | Software projects with a recurring build/test/review cycle |
| `research` | Learning, documentation, academic work |
| `product` | Full product development with security, ops, design, planning |

## Where the Content Lands

After `aibox init` and `aibox apply` with `[processkit].version` pinned:

```
context/
├── skills/                              # Editable copies of installed skills
├── processes/                           # release, code-review, feature-development, bug-fix
├── schemas/                             # primitive schemas
├── state-machines/                      # state machine definitions
└── templates/
    └── processkit/
        └── v0.8.0/
            ├── context/
            │   ├── skills/
            │   └── schemas/
            ├── .processkit/
            │   └── packages/            # The package YAMLs themselves
            └── AGENTS.md
```

The version path (`v0.8.0` above) is whatever `[processkit].version` is pinned
to in `aibox.toml`.

## Changing Packages

Edit `aibox.toml` then run `aibox apply`:

```toml
[context]
packages = ["product"]   # was ["managed"]
```

## Upstream Source

The package YAMLs and the skills they reference are owned by processkit:

- Repository: https://github.com/projectious-work/processkit
- Releases: https://github.com/projectious-work/processkit/releases
- Local copy in your project: `context/templates/processkit/<version>/.processkit/packages/`

To consume a fork or a private mirror, point `[processkit].source` at it (see
[`[processkit]` configuration](../reference/configuration.md#processkit)).
