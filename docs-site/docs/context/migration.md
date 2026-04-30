---
sidebar_position: 3
title: "Migration"
---

# Migration

When the aibox context schema evolves between versions, existing projects may need to update their context files. The `aibox doctor` command helps identify gaps and produces migration artifacts.

:::warning v0.16.0 — `context/AIBOX.md` is gone

Pre-v0.16 releases generated a `context/AIBOX.md` "universal baseline" file
on every `aibox apply`. That file has been **removed** as part of the
aibox⇄processkit split. The canonical agent entry document is now `AGENTS.md`
at the project root, owned by processkit and rendered at `aibox init` time
(write-if-missing — never overwritten).

Existing projects upgrading to v0.16.0 can safely delete `context/AIBOX.md`.
Anything you wrote into it by hand should be moved into `AGENTS.md`,
`context/DECISIONS.md`, or one of the work-instructions files, depending on
its nature.

:::

## How Version Tracking Works

Two pieces track the version:

1. **`aibox.toml`** contains the target schema version:
   ```toml
   [context]
   schema_version = "1.0.0"
   ```

2. **`.aibox-version`** in the project root records the version that was last applied. This file is created during `aibox init` and updated after successful migrations.

When `aibox doctor` detects a mismatch between these two values, it flags the project as needing migration.

## Running Doctor

```bash
aibox doctor
```

Doctor performs the following checks:

- Validates `aibox.toml` syntax and field values
- Detects the container runtime (podman or docker)
- Checks for `.aibox-home/` and `.devcontainer/` directories
- Compares `.aibox-version` against `context.schema_version`
- Validates that expected context files exist for the chosen process flavor

Example output when migration is needed:

```
==> Running diagnostics...
 ✓ Config version: 0.1.0
 ✓ Image: python
 ✓ Process: product
 ✓ Container name: my-app
 ✓ Container runtime: podman
 ✓ .aibox-home/ directory exists at .aibox-home
 ✓ .devcontainer/ directory exists
 ! Schema version mismatch: .aibox-version says 0.9.0, config says 1.0.0
 ✓ Diagnostics complete
```

## Migration Artifacts

When a version mismatch is detected, `doctor` generates migration artifacts in
`context/migrations/`:

```
context/
└── migrations/
    ├── pending/       # Migrations queued but not yet started
    ├── in-progress/   # Migration currently being applied
    └── applied/       # Completed migrations (archived for reference)
```

Each migration is identified by a MIG-ID and lives as a versioned document in
the appropriate subdirectory. Migrations are managed through the normal
resource grammar:

```bash
aibox get migration                       # show pending/in-progress migrations
aibox set migration <id> in-progress      # begin a pending migration
aibox apply migration <id>                # mark a migration as applied
aibox delete migration <id> --reason "…"  # reject and archive without applying
```

## Applying a Migration

### With an AI agent (recommended)

1. Run `aibox doctor` to identify gaps and queue migration artifacts
2. Run `aibox set migration <id> in-progress` to begin the next pending migration
3. Open the migration document from `context/migrations/in-progress/`
4. Paste its contents into a Claude Code session (or let the agent find it via `AGENTS.md`)
5. Review the changes the agent makes
6. Run `aibox apply migration <id>` to mark the migration complete

### Manually

1. Run `aibox doctor` to generate migration artifacts
2. Run `aibox set migration <id> in-progress` to move the migration to `context/migrations/in-progress/`
3. Follow the migration document's checklist
4. Run `aibox apply migration <id>` to archive the migration to `context/migrations/applied/`

:::warning Review before applying

Migration artifacts describe structural changes. They do not migrate content. If a file is renamed, the artifact tells you to create the new file -- but you need to move the content yourself (or have an AI agent do it thoughtfully).

:::

## Best Practices

**Never auto-migrate content.** Structural changes (new files, renames) can be automated. Content changes (rewriting sections, reformatting entries) should always be reviewed by a human or guided AI session.

**Commit before migrating.** Always commit your current state before applying migration changes. This gives you a clean rollback point.

**Run doctor after migrating.** After applying changes, run `aibox doctor` again to confirm everything is clean.

**Keep .aibox-version in version control.** This file should be committed so all team members know which schema version the project uses.

## Schema Document Format

Schema documents in the `schemas/` directory define the expected structure for each version. They specify:

- Which files each process flavor should contain
- Required sections within each file
- File naming conventions
- Directory structure requirements

These schemas are used by `doctor` to validate the project and by migration tooling to compute diffs between versions.
