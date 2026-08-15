# Skills (via processkit)

LLMS index: [llms.txt](/aibox/v0.x/v0.32.5/llms.txt)

---

# Skills

As of aibox **v0.16.0**, skills are no longer bundled with aibox. They live in
**[processkit](https://github.com/projectious-work/processkit)** — a separate,
versioned content repository that ships skills, primitives, processes, packages,
and the canonical `AGENTS.md` template.

aibox owns the **container** (devcontainers, addons, the CLI, the install/apply
machinery). processkit owns the **content** (skills, packages, processes,
state machines). The boundary is deliberate and load-bearing: it lets the two
projects move at their own velocity without dragging each other through breaking
changes.

## Where skills land in your project

After running `aibox init` and `aibox apply` in your project, processkit content
is materialised under your `context/` directory when
`[context].mode = "processkit"`:

```
context/
├── skills/                          # Active, editable skill copies
└── templates/
    └── processkit/
        └── v0.27.4/                 # Immutable upstream snapshot, git-tracked
            ├── context/
            │   └── skills/
            ├── .processkit/
            └── AGENTS.md
```

The version in the path (`v0.27.4`) is whatever you pinned in `aibox.toml`:

```toml
[context]
mode = "processkit"

[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "v0.27.4"
```

When `[context].mode = "harness-only"`, aibox does not install processkit
skills, does not create `context/templates/processkit/`, and does not project
processkit command adapters into harness-specific surfaces. The generated
`AGENTS.md` is a minimal aibox-owned file with no processkit references.

The `context/skills/` copies are yours to edit. The `context/templates/processkit/<version>/`
copies are the immutable upstream snapshot — `aibox apply` uses them as the base
side of a three-way diff to detect upstream changes that should be pulled into
your local edits.

## Skill catalogue and documentation

> **processkit documentation** is not yet deployed as a standalone site.
> Until then, browse the upstream source directly:
>
> - **GitHub:** https://github.com/projectious-work/processkit/tree/main/src/skills
> - **Releases:** https://github.com/projectious-work/processkit/releases
> - **In your project, after `aibox apply`:** `context/skills/` and
>   `context/templates/processkit/<version>/context/skills/`

Every skill is a directory with at least a `SKILL.md` (the agent-readable
instructions) and may include `references/`, `mcp/`, `assets/`, and `scripts/`
siblings. Skills follow the open [Agent Skills specification](https://agentskills.io/specification).

## Browsing installed skills

```bash
aibox get skill                    # list installed skills, grouped by category
aibox get skill --all              # include available-but-not-installed skills
aibox get skill --category ai      # filter by category
aibox describe skill <name>        # frontmatter + description for one skill
```

## Skill Selection

New projects list the standard processkit operating skills explicitly in
`[skills].include`. Use `[skills].include` and `[skills].exclude` for
skill-level overrides in processkit mode:

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

Legacy package selections are still accepted for compatibility, but new
`aibox.toml` files use explicit skill selection as the primary control surface.
`enabled` and `disabled` are accepted as aliases for older configs. `[skills]`
is omitted and ignored in harness-only mode.

## Custom skills

To add a project-specific skill, drop a directory under `context/skills/`:

```
context/skills/my-custom-skill/
└── SKILL.md
```

Local skills are not touched by `aibox apply`. They are also not part of any
processkit package — they exist purely for the local project.

## Core skills

Some processkit skills carry `metadata.processkit.core: true` in their
frontmatter (e.g. `skill-finder`). Core skills are installed regardless of any
`[skills].include` / `[skills].exclude` configuration. `aibox doctor` warns if
you attempt to exclude a core skill.

## Why this split?

- **Independent release cadence.** processkit can ship a new skill or fix a
  prompt without forcing an aibox CLI release.
- **Reusable content.** Other tools can consume processkit directly without
  taking a dependency on aibox or its container stack.
- **Forkable content.** A team can fork processkit, point `[processkit].source`
  at the fork, and ship a private skill catalogue without forking aibox itself.
- **Smaller aibox.** The aibox binary stays focused on container lifecycle and
  the install/diff/migrate machinery.

See [`[processkit]` configuration](../reference/configuration.md#processkit) for
the full set of fields, including release-asset URL templates and SHA256
verification.
