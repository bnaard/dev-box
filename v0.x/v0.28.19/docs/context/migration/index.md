# Migration

LLMS index: [llms.txt](/aibox/v0.x/v0.28.19/llms.txt)

---

# Migration

When the aibox context schema evolves between versions, existing projects may
need to update their context files. The `aibox doctor` command helps identify
schema gaps and produces review artifacts under `.aibox/migration/`.

Separate processkit content and generated-runtime changes are surfaced as
Migration entities under `context/migrations/` in processkit mode.

<div class="alert alert-warning" role="alert"><div class="h4 alert-heading" role="heading">v0.16.0 — `context/AIBOX.md` is gone</div>



Pre-v0.16 releases generated a `context/AIBOX.md` "universal baseline" file
on every `aibox apply`. That file has been **removed** as part of the
aibox⇄processkit split. The canonical agent entry document is now `AGENTS.md`
at the project root. In processkit mode it is rendered from processkit
scaffolding; in harness-only mode it is a minimal aibox-owned file. In both
modes `aibox init` writes it only when missing.

Existing projects upgrading to v0.16.0 can safely delete `context/AIBOX.md`.
Anything you wrote into it by hand should be moved into `AGENTS.md`,
`context/DECISIONS.md`, or one of the work-instructions files, depending on
its nature.

</div>


## How Version Tracking Works

Two pieces track the version:

1. **`aibox.toml`** contains the target context schema version. Current
   canonical processkit-mode configs render this under `[processkit.context]`;
   `[context].schema_version` is still accepted for compatibility:
   ```toml
   [processkit.context]
   schema_version = "1.0.0"
   ```

2. **`aibox.lock`** records the aibox CLI/runtime state last applied to the
   project. Legacy `.aibox-version` files from older projects are absorbed into
   `aibox.lock` and removed by the migration path.

When `aibox doctor` detects a schema mismatch, it flags the project as needing
migration and writes schema review artifacts.

## Running Doctor

```bash
aibox doctor
```

Doctor performs the following checks:

- Validates `aibox.toml` syntax and field values
- Detects the container runtime (podman or docker)
- Checks for `.aibox-home/` and `.devcontainer/` directories
- Compares the current embedded context schema against the configured target
  schema version
- Validates expected context/processkit files for the chosen context mode

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
 ! Context schema: current 1.0.0, target 2.0.0 (migration needed)
 ✓ Diagnostics complete
```

## Migration Artifacts

When a schema mismatch is detected, `doctor` generates review artifacts in
`.aibox/migration/`:

```
.aibox/
└── migration/
    ├── schema-current.md
    ├── schema-target.md
    ├── diff.md
    └── migration-prompt.md
```

When processkit content, runtime-home drift, model/provider changes, or similar
processkit-mode updates need human review, aibox emits Migration entities in
`context/migrations/`:

```
context/
└── migrations/
    ├── pending/       # Migrations queued but not yet started
    ├── in-progress/   # Migration currently being applied
    └── applied/       # Completed migrations (archived for reference)
```

Each processkit Migration is identified by a MIG-ID and lives as a versioned
document in the appropriate subdirectory. These Migration entities are managed
through the normal resource grammar:

```bash
aibox get migration                       # show pending/in-progress migrations
aibox set migration <id> in-progress      # begin a pending migration
aibox apply migration <id>                # mark a migration as applied
aibox delete migration <id> --reason "…"  # reject and archive without applying
```

## Applying a Migration

## Strict Schema And Storage Policy

Migrations are the durable fix for schema, vocabulary, filename, ID, and
directory-layout drift. A clean project should satisfy the current schemas and
storage policy directly, not by carrying project-local compatibility allowlists.

Do not resolve doctor findings by adding `legacy_known_*` schema entries,
doctor suppressions, mixed-layout exceptions, or local notes that accept legacy
event names, IDs, filenames, or directory shapes as the steady state. If a
schema genuinely needs a new value or layout, introduce that as an explicit
schema migration and then migrate existing entities and references to the new
standard.

Compatibility shims are acceptable only as short-lived migration aids. Before a
migration is marked applied, the repository should contain canonical values,
canonical filenames, canonical directory placement, and updated references.

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

<div class="alert alert-warning" role="alert"><div class="h4 alert-heading" role="heading">Review before applying</div>



Migration artifacts describe structural changes. They do not migrate content. If a file is renamed, the artifact tells you to create the new file -- but you need to move the content yourself (or have an AI agent do it thoughtfully).

</div>


## Best Practices

**Never auto-migrate content.** Structural changes (new files, renames) can be automated. Content changes (rewriting sections, reformatting entries) should always be reviewed by a human or guided AI session.

**Migrate forward, do not grandfather.** Fix schema and storage drift by moving
entities to the current vocabulary, filenames, IDs, and directory layout.
Project-local allowlists and doctor suppressions are not acceptable terminal
states.

**Commit before migrating.** Always commit your current state before applying migration changes. This gives you a clean rollback point.

**Run doctor after migrating.** After applying changes, run `aibox doctor` again to confirm everything is clean.

**Keep `aibox.lock` in version control.** It records the resolved CLI,
processkit, addon, and managed runtime state shared by the project. A legacy
`.aibox-version` file is migration input only and is removed by `aibox apply`.

## Schema Document Format

Schema documents in the `schemas/` directory define the expected structure for each version. They specify:

- Which files each process flavor should contain
- Required sections within each file
- File naming conventions
- Directory structure requirements

These schemas are used by `doctor` to validate the project and by migration tooling to compute diffs between versions.
