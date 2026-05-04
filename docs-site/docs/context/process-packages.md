---
sidebar_position: 2
title: "Skill Selection"
---

# Skill Selection

New aibox projects install the full processkit product skill set by default.
This keeps the project-local `context/skills/` directory complete and makes
skill selection a direct comment/uncomment workflow in `aibox.toml`.

Use `[skills].enabled` for explicit additions and `[skills].disabled` for
explicit removals:

```toml
[skills]
enabled = [
  # "extra-skill",
]
disabled = [
  # "skill-to-omit",
]
```

Legacy package tiers (`minimal`, `managed`, `software`, `research`, `product`)
are still accepted for compatibility when present under
`[processkit.context].packages` or old `[context].packages`, but new generated
configs treat them as deprecated.

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
        └── v0.25.7/
            ├── context/
            │   ├── skills/
            │   └── schemas/
            ├── .processkit/
            │   └── packages/            # The package YAMLs themselves
            └── AGENTS.md
```

The version path (`v0.25.7` above) is whatever `[processkit].version` is pinned
to in `aibox.toml`.

## Upstream Source

The skills are owned by processkit:

- Repository: https://github.com/projectious-work/processkit
- Releases: https://github.com/projectious-work/processkit/releases
- Local copy in your project: `context/templates/processkit/<version>/.processkit/packages/`

To consume a fork or a private mirror, point `[processkit].source` at it (see
[`[processkit]` configuration](../reference/configuration.md#processkit)).
