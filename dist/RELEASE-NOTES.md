# aibox v0.17.4

Patch release adding user-extensible bind mounts, personal credential overlays,
and automatic migration of old processkit runtime settings out of `aibox.toml`.

## New features

### User-extensible bind mounts (`[[container.extra_volumes]]`)

Projects can now declare additional bind mounts in `aibox.toml`:

```toml
[[container.extra_volumes]]
source = "/home/user/.local/share/fish"
target = "/home/aibox/.local/share/fish"

[[container.extra_volumes]]
source = "/data/models"
target = "/home/aibox/models"
read_only = true
```

Volumes are appended after the built-in aibox mounts in `docker-compose.yml`.
Path traversal (`..`) is rejected at load time.

### Personal credential overlay (`.aibox-local.toml`)

A gitignored `.aibox-local.toml` file can be placed alongside `aibox.toml` to
declare per-developer credentials and extra mounts without touching the shared
config:

```toml
[container.environment]
GITHUB_TOKEN = "ghp_..."
OPENAI_API_KEY = "sk-..."

[[container.extra_volumes]]
source = "/home/user/.config/gh"
target = "/home/aibox/.config/gh"
```

Environment variables from `.aibox-local.toml` win on conflict with `aibox.toml`.
Extra volumes are additive. `aibox context` now warns if `.aibox-local.toml` is
missing from `.gitignore`.

### Cargo registry mounts (rust addon)

When the `rust` addon is enabled, `aibox sync` now mounts the host cargo
registry into the container to avoid re-downloading crates on rebuild:

```
~/.cargo/registry  →  /home/aibox/.cargo/registry
~/.cargo/git       →  /home/aibox/.cargo/git
```

Only the registry cache is mounted — `~/.cargo/bin` is intentionally excluded
because host-compiled binaries won't run inside the container.

## Migration: processkit runtime settings (aibox#38)

`aibox sync` now detects and migrates old processkit runtime settings that were
previously written directly into `aibox.toml [context]`:

| Old key in `[context]`         | Migrated to                                              |
|-------------------------------|----------------------------------------------------------|
| `id_format`, `id_slug`        | `context/skills/id-management/config/settings.toml`     |
| `directories`, `sharding`, `index` | `context/skills/index-management/config/settings.toml` |

After migration the keys are removed from `aibox.toml` while preserving all
comments and formatting. The migration is idempotent — if a `settings.toml`
already exists (agent already set it up), the old keys are removed without
overwriting the file. Unknown keys (`budget`, `grooming`, …) are left in place
with a warning.
