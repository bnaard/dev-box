# Existing Project

LLMS index: [llms.txt](/aibox/v0.x/v0.31.1/llms.txt)

---

# Existing Project

This guide covers adding aibox to a project that already exists.

## Create aibox.toml

If your project does not yet have a `aibox.toml`, create one manually or use `init`:

```bash
cd my-existing-project
aibox init my-existing-project --harness claude
```

<div class="alert alert-warning" role="alert"><div class="h4 alert-heading" role="heading">init will not overwrite</div>



If `aibox.toml` already exists, `init` will refuse to run. Either delete the existing file or edit it directly.

</div>


If you prefer to write it by hand:

```toml
[aibox]
project_name = "my-existing-project"

[container]
name     = "my-existing-project"
hostname = "my-existing-project"

[container.image]
release_version = "latest"
base = "debian"

[context]
mode = "processkit"
packages = ["product"]

[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "latest"

[processkit.context]
schema_version = "1.0.0"

[ai]
harnesses = [
  { harness = "claude", enable = true, install = true },
]

[audio]
enabled = false
```

For a container-and-harness-only adoption with no processkit content, use:

```toml
[context]
mode = "harness-only"

[ai]
harnesses = [
  { harness = "claude", enable = true, install = true },
]
```

In harness-only mode, omit `[processkit]`, `[processkit.context]`, and
`[skills]`; `aibox apply` will not create processkit skill mirrors or
processkit Migration entities.

<div class="alert alert-success" role="alert"><div class="h4 alert-heading" role="heading">Pin processkit before apply</div>



Set `[processkit].version` to a real tag (e.g. `v0.27.4`) before the first
`aibox apply`. The default sentinel `unset` skips processkit content
installation entirely — you will get devcontainer files but no skills,
processes, or processkit-rendered `AGENTS.md` template.

This tip only applies when `[context].mode = "processkit"`.

</div>


## Apply Devcontainer Files

Run `apply` to create the `.devcontainer/` directory from your config:

```bash
aibox apply
```

This creates:

- `.devcontainer/Dockerfile`
- `.devcontainer/docker-compose.yml`
- `.devcontainer/devcontainer.json`

## Replacing Hand-Written Devcontainer Files

If your project already has a `.devcontainer/` directory with hand-written files, you have two options:

### Option A: Let aibox take over

1. Back up your existing files:
   ```bash
   cp -r .devcontainer .devcontainer.bak
   ```
2. Run `aibox apply` -- it will overwrite the existing files
3. Move any custom configuration into `aibox.toml`:
   - Environment variables go in `[container.environment]`
   - Bind mounts go in `[[container.extra_volumes]]` (or `.aibox-local.toml` for secrets and per-developer paths)
4. Rebuild without cached image layers: `aibox apply --no-cache` (`--rebuild` is an alias)

### Option B: Keep hand-written files

If your devcontainer setup is heavily customized, you can still use aibox for
context or harness configuration and skip the container lifecycle commands. Use
`aibox.toml` for the `[aibox]`, `[context]`, `[ai]`, and, when applicable,
`[processkit]` sections, and manage `.devcontainer/` yourself.

## Running Diagnostics

Use `doctor` to validate your project structure:

```bash
aibox doctor
```

This checks:

- Config file validity and version
- Container runtime availability (podman or docker)
- `.aibox-home/` directory existence
- `.devcontainer/` directory existence
- Image and process settings

Example output:

```
==> Running diagnostics...
 ✓ Config version is compatible
 ✓ Container runtime detected
 ✓ .aibox-home/ directory exists
 ✓ .devcontainer/ directory exists
 ✓ Generated compose enables an init reaper
 ✓ Runtime resource pressure is below configured thresholds
 ✓ Diagnostics complete
```

## Migrating from `.root/` to `.aibox-home/`

If you are upgrading from aibox ≤ v0.3.4, the persisted config directory was renamed from `.root/` to `.aibox-home/`. This directory is **gitignored and not tracked**, so use a plain filesystem rename:

```bash
mv .root .aibox-home
```

<div class="alert alert-warning" role="alert"><div class="h4 alert-heading" role="heading">Do not use `git mv`</div>



`git mv .root .aibox-home` will fail because `.root/` is listed in `.gitignore` and was never committed. Use a regular `mv` command.

</div>


aibox will fall back to `.root/` automatically if `.aibox-home/` does not exist, so this migration is optional but recommended.

## Migrating Context Structure

If your project already has context files (like `DECISIONS.md` or `BACKLOG.md`) that predate aibox, `doctor` can help identify what needs to change. See [Migration](../context/migration.md) for the full guide.

## Common gaps to watch for

- **Node.js version pinning** -- use the `node` addon and `--addon-tool` or
  `[addons.node.tools]` when you need a specific supported version.
- **postCreateCommand** -- use `post_create_command` in `[container]` config.
  For git identity, prefer `.aibox-home/.config/git/config`.
- **VS Code extensions/settings** -- the generated `devcontainer.json` works with VS Code Dev Containers. For project-specific settings, keep a `.vscode/settings.json`.
- **Third-party CLI tools** (Gemini, Jules) -- mount from host via `[[container.extra_volumes]]`, or add installation to `post_create_command`.
- **Existing generated containers** -- after adopting aibox, recreate the
  runtime so Compose identity, image name, mounts, and init-reaper settings take
  effect.

## Build and Start

Once `aibox.toml` and `.devcontainer/` are in place:

```bash
aibox apply    # Regenerate files and build image
aibox up       # Start and attach
```

The workflow is identical to a [new project](new-project.md#build-and-start) from this point forward.

## Next Steps

- [Configuration reference](../reference/configuration.md)
- [CLI commands](../reference/cli-commands.md)
- [Runtime operations](../container/runtime-operations.md)
- [Context migration guide](../context/migration.md)
