# aibox v0.17.0 — generalised lock file absorbs .aibox-version (DEC-037)

`aibox.lock` is now the single source of truth for both the CLI version that
last touched a project **and** the pinned processkit content. The legacy
`.aibox-version` file is deleted on first `aibox sync` (hard-cut, idempotent).

## What changed

### Sectioned lock format (`[aibox]` + `[processkit]`)

`aibox.lock` gains a two-section TOML shape:

```toml
[aibox]
cli_version = "0.17.0"
synced_at   = "2026-04-08T16:42:00Z"

[processkit]
source               = "https://github.com/projectious-work/processkit.git"
version              = "v0.5.1"
src_path             = "src"
resolved_commit      = "abc123def456"
installed_at         = "2026-04-08T16:42:00Z"
```

The former flat shape (all processkit fields at top level, no `[aibox]`
section) is auto-upgraded on first read — no manual migration needed.

### One-time hard-cut migration

`aibox sync` now calls `migrate_legacy_lock_files` as its first step:

1. If `.aibox-version` is present, `aibox.lock` is read (and upgraded in
   memory if it is still in the flat shape), written back in the new
   sectioned format, and `.aibox-version` is deleted.
2. If `.aibox-version` is absent, the step is a no-op.

### `.aibox-version` removed from all aibox-owned surfaces

| Surface | Change |
|---|---|
| `aibox init` | No longer writes `.aibox-version` |
| `.gitignore` (generated) | Entry removed from aibox section |
| `aibox doctor` | Reads `aibox.lock [aibox].cli_version` instead |
| `aibox doctor expected_files` | Now checks `aibox.lock` (not `.aibox-version`) |
| `aibox reset` / backup | `aibox.lock` is now in managed items; `.aibox-version` kept as legacy cleanup |
| Sync perimeter | `.aibox-version` removed; `aibox.lock` was already present |
| Migration doc rollback snippet | Updated from `.aibox-version` to `aibox.lock` |

### Also in this release

Vim insert-mode word-movement bindings in the seeded vimrc:

- `Alt-Left` → move to previous word beginning (VSCode-style)
- `Alt-Right` → move to next word end
- `Home` → smart home (first non-blank vs column 0 toggle)

Zellij `ai-layout` horizontal split changed from 53/47 to **50/50**.

## Upgrading

Run `aibox sync` once. You will see:

```
✓ Migrated: .aibox-version absorbed into aibox.lock
```

After that `.aibox-version` is gone and `aibox.lock` holds everything.

## Breaking changes

None for existing projects — migration is automatic. Projects consuming aibox
as a library should update any code that read `.aibox-version` directly to
read `aibox.lock [aibox].cli_version` instead.
