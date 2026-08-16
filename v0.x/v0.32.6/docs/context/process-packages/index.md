# Skill Selection

LLMS index: [llms.txt](/aibox/v0.x/v0.32.6/llms.txt)

---

# Skill Selection

New aibox projects list the standard processkit operating skills explicitly in
`[skills].include`. This makes skill selection a direct comment/uncomment
workflow in `aibox.toml` without relying on legacy package tiers.

Use `[skills].include` for explicit additions and `[skills].exclude` for
explicit removals:

```toml
[skills]
include = [
  "pk-doctor",
  "status-briefing",
]
exclude = [
  # "skill-to-omit",
]
```

Legacy package tiers (`minimal`, `managed`, `software`, `research`, `product`)
are still accepted for compatibility under `[context].packages` when
`[context].mode = "processkit"`, but explicit `[skills]` selection is the
preferred control surface.

## Where the Content Lands

After `aibox init` and `aibox apply` with `[context].mode = "processkit"` and
`[processkit].version` pinned:

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

Harness-only projects do not install this content and do not create
`context/templates/processkit/`.

## Upstream Source

The skills are owned by processkit:

- Repository: https://github.com/projectious-work/processkit
- Releases: https://github.com/projectious-work/processkit/releases
- Local copy in your project: `context/templates/processkit/<version>/.processkit/packages/`

To consume a fork or a private mirror, point `[processkit].source` at it (see
[`[processkit]` configuration](../reference/configuration.md#processkit)).
