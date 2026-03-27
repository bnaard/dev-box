# Version Upgrade Flows Design

**Date:** 2026-03-26
**Status:** Draft
**Scope:** CLI binary upgrades, container image upgrades, config migration

## Overview

aibox has three independently versioned components:

| Component | Version source | Current | Example value |
|-----------|---------------|---------|---------------|
| CLI binary | `cli/Cargo.toml` (`CARGO_PKG_VERSION`) | 0.14.1 | `0.14.1` |
| Container base image | `aibox.toml` `[aibox] version` field | per-project | `0.13.2` |
| Context schema | `aibox.toml` `[context] schema_version` | `1.0.0` | `1.0.0` |

A version mismatch can occur between any pair. The CLI version governs which
config fields are recognized; the image version governs what is inside the
container; the schema version governs the context directory structure. All three
must be considered during upgrades.

### Current State (as of v0.14.1)

What already exists:

- **`aibox update --check`** -- queries GHCR tags + GitHub releases, reports
  available upgrades for both CLI and image.
- **`aibox update`** -- fetches latest image tag for the configured base flavor,
  bumps `aibox.toml` version, regenerates `.devcontainer/`, offers to rebuild.
- **`aibox sync`** -- calls `migration::check_and_generate_migration()` which
  compares `.aibox-version` to the running CLI version and generates a
  `context/migrations/{from}-to-{to}.md` document if they differ.
- **`aibox doctor`** -- checks CLI version file, container image label match,
  schema version match, and context structure completeness.
- **`aibox start`** -- hard-errors if the running container's image label
  disagrees with `aibox.toml` version (skips check for pre-v0.13 images
  without the label).

What is missing:

- No automatic config migration (e.g., section renames).
- No version compatibility matrix (CLI v0.14 vs image v0.12).
- No validation when the user manually edits `aibox.toml` version.
- No `aibox migrate` command.
- No GHCR tag existence check before build.
- Migration doc is generated but never enforced (user can ignore it).

---

## Comparable Tool Analysis

### Docker Desktop

- Auto-downloads updates in background; prompts to restart.
- Compose file format version is validated at parse time -- unknown fields
  produce warnings, removed fields produce errors.
- No automatic config migration; breaking changes require manual edits.
- Rollback: user can install a previous `.dmg` but Docker does not provide a
  built-in downgrade path.

### Homebrew

- `brew upgrade` upgrades all formulae. Individual pins (`brew pin`) prevent
  upgrades.
- No config migration -- formulae are self-contained.
- Rollback via `brew switch` (deprecated) or manual `git checkout` of the tap.
- Key lesson: **upgrades are opt-in per package, not forced globally**.

### rustup

- `rustup update` updates all installed toolchains.
- `rust-toolchain.toml` pins the project-level version; `rustup` respects it
  automatically when entering the directory.
- If the pinned toolchain is not installed, `rustup` downloads it
  transparently.
- No config migration needed -- the toolchain file format is stable.
- Rollback: change the pin back and `rustup` switches.
- Key lesson: **project-level version pinning with automatic acquisition**.

### nvm

- `.nvmrc` pins the Node version per project.
- `nvm use` reads the file and switches; if the version is not installed,
  it tells the user to run `nvm install`.
- No auto-install by default (shell hook `nvm use --install` exists).
- No config migration.
- Key lesson: **explicit user action to switch; clear error on missing version**.

### Takeaways for aibox

| Principle | Source |
|-----------|--------|
| Project-level version pin is authoritative | rustup, nvm |
| Auto-acquire missing versions when possible | rustup |
| Never silently break -- warn or error, never auto-migrate destructively | Docker, Homebrew |
| Provide explicit rollback path | all |
| Separate "check for updates" from "apply updates" | Homebrew, rustup |

---

## Flow Definitions

### Flow A: CLI Upgrade (user installs newer CLI binary)

```
User installs CLI v0.14 while project has aibox.toml version = "0.13.2"

  CLI v0.14 binary
       |
       v
  aibox start / sync / doctor
       |
       v
  Load aibox.toml -----> Parse config
       |                     |
       |           +--[ config parse fails? ]--+
       |           |                           |
       |        [yes]                        [no]
       |           |                           |
       |    Emit structured error         Continue
       |    with migration hint               |
       |    "Run aibox migrate"               v
       |                            Compare .aibox-version
       |                            to CARGO_PKG_VERSION
       |                                      |
       |                     +---[ mismatch? ]---+
       |                     |                   |
       |                   [yes]               [no]
       |                     |                   |
       |              Generate migration     Proceed
       |              doc (existing behavior)  normally
       |              + emit warning
       |                     |
       v                     v
                      Execute command
```

**Detection mechanism:**

1. **Config compatibility** -- During `AiboxConfig::from_cli_option()`, if the
   TOML contains removed/renamed sections (e.g., `[process]` after the v0.14
   rename to `[context]`), deserialization fails. Currently this is a raw
   parse error. Proposal: catch known legacy section names and emit a
   structured diagnostic with the exact required changes.

2. **CLI version vs `.aibox-version`** -- Already implemented in
   `migration::check_and_generate_migration()` (runs during `sync`). Proposal:
   also run this check during `start` and `doctor`.

**UX:**

| Situation | Behavior |
|-----------|----------|
| Config parses successfully, `.aibox-version` matches | Silent. Proceed. |
| Config parses successfully, `.aibox-version` older | Warning + generate migration doc. Command proceeds. |
| Config fails to parse due to renamed sections | Error with actionable message: "Section `[process]` was renamed to `[context]` in v0.14. Run `aibox migrate` to update your config." |
| Config fails to parse for other reasons | Standard error (unchanged). |

**Migration strategy:**

Introduce `aibox migrate` command:

```
aibox migrate [--dry-run]
```

Behavior:

1. Read `aibox.toml` as raw TOML text (not typed deserialization).
2. Walk the **migration registry** -- an ordered list of `(from_version,
   to_version, transform_fn)` entries compiled into the CLI.
3. Apply each applicable transform in order (e.g., rename `[process]` to
   `[context]`, move `packages` field).
4. Write the updated file, preserving comments (use `toml_edit`).
5. Update `.aibox-version`.
6. Print a summary of changes.
7. With `--dry-run`, print the diff without writing.

The migration registry is a compile-time table analogous to database
migrations. Each entry has:

```rust
struct ConfigMigration {
    from: &'static str,       // semver range, e.g., ">=0.13.0, <0.14.0"
    to: &'static str,         // target version, e.g., "0.14.0"
    description: &'static str,
    apply: fn(&mut toml_edit::DocumentMut) -> Result<()>,
}
```

This extends the existing `KNOWN_MIGRATIONS` array in `migration.rs` with
executable transforms instead of just documentation.

**Rollback:**

- `aibox migrate` creates a backup at `aibox.toml.bak` before writing.
- The generated migration doc already includes `git checkout HEAD --
  .aibox-version context/ .devcontainer/` as a rollback command.
- For git-tracked projects, `git diff` shows the exact changes; `git checkout`
  reverts.

---

### Flow B: Image Upgrade (`aibox update`)

```
User runs: aibox update

  aibox update
       |
       v
  Load current config (aibox.toml)
       |
       v
  Query GHCR tags/list for base flavor
       |
       v
  Parse all tags as semver, find latest
       |
       +---[ latest <= current? ]---+
       |                            |
     [yes]                        [no]
       |                            |
  "Already up to date"         Show: current -> latest
       |                            |
       v                            v
     Exit                   +--[ breaking? ]--+
                            |                 |
                          [yes]             [no]
                            |                 |
                     Show breaking       Prompt:
                     change summary      "Upgrade X -> Y?"
                     + extra warning          |
                            |            +--[yes]--+
                            |            |         |
                            v            |       [no]
                     Prompt with         |     "Cancelled"
                     "migration          |
                      required" note     |
                            |            |
                            v            v
                     Update aibox.toml version
                            |
                            v
                     Regenerate .devcontainer/
                            |
                            v
                     Run config migration if needed
                     (auto-apply non-breaking transforms)
                            |
                            v
                     Prompt: "Rebuild now?"
                            |
                      +--[yes]--+--[no]--+
                      |                  |
                 docker compose       "Run aibox sync
                 build                 when ready"
                      |
                      v
                 "Upgrade complete"
```

**Detection mechanism:**

Already implemented: `fetch_latest_image_version()` queries GHCR OCI tags API,
filters by flavor prefix, parses semver, returns highest.

Addition needed: **breaking change detection**. Embed a compatibility manifest
in the CLI:

```rust
struct ImageCompatibility {
    version: &'static str,
    min_cli_version: &'static str,   // minimum CLI version that supports this image
    breaking_from: Option<&'static str>, // if set, upgrading from this version requires migration
    notes: &'static [&'static str],
}
```

When `aibox update` detects a jump across a breaking boundary, it shows
additional warnings and triggers `aibox migrate` automatically after updating
the version field.

**UX:**

| Situation | Behavior |
|-----------|----------|
| Already at latest | Info message, exit 0. |
| Minor bump (no breaking changes) | Prompt to confirm, update, rebuild. |
| Major/breaking bump | Extra warning banner, list breaking changes, prompt, update, auto-run migrate, rebuild. |
| Network unreachable | Warning with suggestion to check connectivity. Exit 0. |
| `--dry-run` flag | Show what would change, exit. |
| `-y` flag | Skip all prompts, proceed. |

**Migration strategy:**

1. `update_toml_version()` already handles the version field update.
2. After updating, reload config. If reload fails (renamed sections), run
   `aibox migrate` automatically.
3. Then call `sync_config_files()` (already exists).

**Rollback:**

- `aibox update` already preserves the old version string. Add: write
  `aibox.toml.bak` before modification.
- If rebuild fails, print: "Rollback: restore aibox.toml.bak and run
  `aibox sync`."
- For image-level rollback, the old image layers remain in the local Docker
  cache until pruned.

---

### Flow C: Manual Edit (`aibox sync` after user edits aibox.toml)

```
User edits aibox.toml: version = "0.15.0"
User runs: aibox sync

  aibox sync
       |
       v
  migration::check_and_generate_migration()
  (compare .aibox-version to CLI version)
       |
       v
  AiboxConfig::from_cli_option()
       |
       +---[ parse fails? ]---+
       |                      |
     [no]                   [yes]
       |                      |
       v                  Error + hint
  Validate version            |
  field against            "Run aibox migrate"
  known constraints
       |
       +---[ version exists in GHCR? ]---+
       |                                 |
     [yes]                             [no]
       |                                 |
       v                           Warning:
  Check CLI compatibility       "Version 0.15.0 not found
       |                         in registry. Build will
       +---[ CLI too old? ]--+   use the tag as-is."
       |                     |        |
     [no]                 [yes]       v
       |                     |   Continue (user
       v                Error:   may have a local
  Proceed with          "aibox.toml pins   image)
  sync normally          v0.15.0 but this
       |                 CLI (v0.14.1)
       |                 does not support
       |                 images above
       |                 v0.14.x. Upgrade
       |                 the CLI first."
       v
  seed + generate + reconcile + build
```

**Detection mechanism:**

1. **GHCR tag existence check** -- New. Before generating the Dockerfile,
   query GHCR to verify the tag `base-{flavor}-v{version}` exists. This is a
   non-blocking warning (the user might have a locally built image or a
   private registry).

2. **CLI compatibility check** -- New. The CLI embeds a `MAX_SUPPORTED_IMAGE`
   constant. If `aibox.toml` version exceeds it, emit a hard error directing
   the user to upgrade the CLI.

3. **Config parse validation** -- Existing. If the user introduces fields from
   a newer schema, serde will reject unknown fields (with `#[serde(deny_unknown_fields)]`
   if we add it) or silently ignore them (current behavior). Proposal: add
   `deny_unknown_fields` to catch forward-incompatible edits.

**UX:**

| Situation | Behavior |
|-----------|----------|
| Version exists, CLI compatible | Proceed normally. |
| Version not found in GHCR | Warning (non-blocking). Build may fail at pull time. |
| Version requires newer CLI | Hard error with upgrade instructions. |
| Unknown config fields | Hard error: "Unknown field `foo` in `[aibox]`. Check that your CLI version supports this config." |

**Migration strategy:**

No automatic migration -- the user made a deliberate edit. If the config
fails to parse, direct them to `aibox migrate` or manual correction.

**Rollback:**

- `git checkout -- aibox.toml` (for git-tracked projects).
- `aibox sync` is idempotent; re-running after fixing the version is safe.

---

## `aibox doctor` Enhancements

Current checks (v0.14.1):

1. Config validity
2. Container runtime presence
3. `.aibox-home/` directory and subdirectories
4. `.devcontainer/` files
5. Context structure vs packages
6. `.gitignore` entries
7. Security audit tools
8. Schema version match
9. Container image version label match
10. CLI version file (`.aibox-version`) match

Proposed additions:

| Check | Severity | Description |
|-------|----------|-------------|
| CLI-image compatibility | Error | `aibox.toml` version vs `MAX_SUPPORTED_IMAGE` |
| Image tag exists in GHCR | Warning | Query registry, warn if tag not found |
| Pending migration docs | Warning | Scan `context/migrations/*.md` for `Status: pending` |
| Config field deprecation | Warning | Detect legacy fields that parse via compat shims (e.g., `[process]` vs `[context]`) |
| CLI update available | Info | Compare `CARGO_PKG_VERSION` to latest GitHub release (already in `update --check`, surface in doctor too) |
| `.aibox-version` freshness | Warning | If `.aibox-version` is more than one major version behind CLI |

### Proposed `doctor` output grouping:

```
$ aibox doctor

  Config .................................................. OK
  Container runtime ...................................... OK (podman 5.4.0)
  CLI version ............................................ OK (v0.14.1)
  Image version .......................................... OK (v0.13.2, matches container)

  VERSION CHECKS
  CLI <-> config compatibility ........................... OK
  Image tag in registry .................................. OK (base-debian-v0.13.2 exists)
  Pending migrations ..................................... WARN (1 pending)
    context/migrations/0.13.0-to-0.14.1.md: Status: pending

  STRUCTURE CHECKS
  .aibox-home/ ........................................... OK
  .devcontainer/ ......................................... OK
  context/ ............................................... OK (12 files, 0 extra)

  SUMMARY: 1 warning, 0 errors
```

---

## `aibox migrate` Command Design

```
aibox migrate [--dry-run] [--from <version>] [--to <version>]
```

### Behavior

1. **Detect current version** from `.aibox-version` (or `--from` override).
2. **Detect target version** from `CARGO_PKG_VERSION` (or `--to` override).
3. **Load `aibox.toml` as raw `toml_edit::DocumentMut`** to preserve
   formatting, comments, and ordering.
4. **Walk the migration registry** and collect applicable transforms.
5. **Show the migration plan**:
   ```
   Migration plan: v0.13.2 -> v0.14.1

   1. Rename section [process] -> [context]           (required)
   2. Move field process.packages -> context.packages  (required)
   3. Add field context.schema_version = "1.0.0"       (required)

   3 changes will be applied.
   ```
6. **With `--dry-run`**: print the unified diff of `aibox.toml` and exit.
7. **Without `--dry-run`**: prompt for confirmation, write `aibox.toml.bak`,
   apply transforms, update `.aibox-version`, print summary.
8. **Run `aibox sync --no-build`** after successful migration to regenerate
   `.devcontainer/` files.

### Migration Registry Architecture

```rust
/// A single config migration step.
struct ConfigMigration {
    /// Semver range this migration applies FROM.
    from_range: &'static str,  // e.g., ">=0.12.0, <0.14.0"
    /// Version this migration brings the config TO.
    to_version: &'static str,  // e.g., "0.14.0"
    /// Human-readable description of the change.
    description: &'static str,
    /// The transform function. Receives a mutable TOML document.
    apply: fn(&mut toml_edit::DocumentMut) -> Result<bool>,
    /// Is this migration required (error if skipped) or recommended (warning)?
    required: bool,
}

/// Ordered list of all known config migrations.
static CONFIG_MIGRATIONS: &[ConfigMigration] = &[
    ConfigMigration {
        from_range: ">=0.11.0, <0.14.0",
        to_version: "0.14.0",
        description: "Rename [process] to [context], move packages field",
        apply: migrate_process_to_context,
        required: true,
    },
    // Future migrations added here in order.
];
```

Migrations compose: upgrading from v0.11 to v0.15 applies all intermediate
transforms in sequence. Each transform is idempotent (checks whether the
change has already been applied before modifying).

---

## Version Compatibility Matrix

Embed a compatibility table in the CLI binary:

```rust
struct VersionCompat {
    cli_version: &'static str,
    min_image: &'static str,
    max_image: &'static str,
    min_schema: &'static str,
    max_schema: &'static str,
}

static COMPAT: VersionCompat = VersionCompat {
    cli_version: env!("CARGO_PKG_VERSION"),
    min_image: "0.11.0",   // oldest image this CLI can generate files for
    max_image: "0.14.1",   // newest image this CLI knows about
    min_schema: "1.0.0",
    max_schema: "1.0.0",
};
```

Used in:
- `cmd_sync` -- validate `aibox.toml` version against `COMPAT.max_image`.
- `cmd_start` -- same check before attempting container operations.
- `cmd_doctor` -- report compatibility status.

---

## Rollback Strategy Summary

| Scenario | Rollback method |
|----------|----------------|
| Bad config after `aibox migrate` | Restore `aibox.toml.bak` |
| Bad container after image upgrade | `git checkout HEAD -- .devcontainer/ aibox.toml .aibox-version` then `aibox sync` |
| Bad context after schema migration | `git checkout HEAD -- context/` |
| CLI binary downgrade needed | Reinstall previous version from GitHub releases |
| Everything broken | `aibox reset --backup` (existing command) creates a timestamped backup, then `aibox init` from scratch |

---

## Decision Matrix

| Design question | Decision | Rationale |
|----------------|----------|-----------|
| Should `aibox start` auto-migrate config? | No | Destructive changes require explicit user action. Error with instructions instead. |
| Should `aibox sync` auto-migrate config? | No | Same principle. Generate migration doc, warn, but do not modify `aibox.toml` automatically. |
| Should `aibox update` auto-migrate after version bump? | Yes, with prompt | The user has already opted into upgrading. Migrating config is the natural next step. Still prompt unless `-y`. |
| Should `aibox migrate` exist as a separate command? | Yes | Explicit, discoverable, testable. Mirrors `rustup`, `diesel migration run`, `alembic upgrade`. |
| Should unknown config fields be rejected? | Yes (`deny_unknown_fields`) | Catches forward-incompatible manual edits and typos. |
| Should GHCR tag existence be validated? | Warning only | The user may have a local image or private mirror. Non-blocking. |
| Should CLI refuse to work with images newer than itself? | Yes (hard error) | Prevents undefined behavior from unrecognized Dockerfile directives or addon APIs. |
| Should CLI refuse to work with images much older than itself? | Warning only | Old images still work; user may have reasons to stay pinned. |
| Should `aibox doctor` check for CLI updates? | Yes (info level) | Low-cost network call, high value. Already in `update --check`. |

---

## Implementation Priority

| Phase | Items | Effort |
|-------|-------|--------|
| P0 (next release) | `aibox migrate` command with registry, `deny_unknown_fields`, CLI-image compat check in `sync`/`start` | Medium |
| P1 (following release) | GHCR tag validation in `sync`, breaking change detection in `update`, `doctor` enhancements | Medium |
| P2 (later) | Auto-run `migrate` from within `update`, migration plan preview in `doctor`, per-project CLI version pinning (like `rust-toolchain.toml`) | Low-Medium |

---

## Open Questions

1. **Per-project CLI pinning** -- Should `aibox.toml` specify a minimum CLI
   version (like `rust-toolchain.toml`)? This would let the CLI self-validate
   on every invocation. Downside: yet another version to maintain.

2. **Schema version vs CLI version vs image version** -- Currently schema
   version is independent (`1.0.0` while CLI is `0.14.1`). Should schema
   version track CLI major/minor instead? Separate versioning adds cognitive
   overhead but allows independent evolution.

3. **Offline mode** -- GHCR checks require network. Should there be an
   `--offline` flag that skips all registry queries? Or should timeouts be
   short enough (2s) that network failures are non-blocking?

4. **Multi-step upgrades** -- If a user jumps from v0.11 to v0.16, should
   `aibox migrate` apply all intermediate transforms, or should it require
   stepping through one minor version at a time? Recommendation: apply all
   in sequence (composable migrations), similar to database migration tools.
