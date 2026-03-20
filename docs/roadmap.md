# Roadmap

This page outlines planned features and improvements for dev-box.

## Current — v0.3.8

The current release includes:

- Rust CLI with full container lifecycle (init, generate, build, start, stop, attach, status, doctor, update)
- 8 container image flavors (base, python, latex, typst, rust, python-latex, python-typst, rust-latex)
- `dev-box.toml` configuration system
- 4 work process flavors (minimal, managed, research, product)
- Context scaffolding with OWNER.md sharing
- Audio support (PulseAudio bridging) with `dev-box audio check/setup`
- Install script for pre-built binaries
- Shell completions for bash, zsh, fish, powershell, elvish
- Interactive init prompts when flags are omitted
- Registry-based update with upgrade (`dev-box update`, `--check`, `--dry-run`)
- Minijinja template engine for Dockerfile and docker-compose.yml generation
- Dockerfile.local support for project-specific build layers
- AI provider configuration (`[ai]` section with `providers` field)
- Non-root user support (`container.user` field)
- Yazi file manager (replaced Strider) with three IDE layouts (dev, focus, assist)
- Language-specific `.gitignore` blocks per image flavor
- `.dev-box-home/` for persistent config (with `.root/` backward compatibility)
- LaTeX Workshop settings in generated devcontainer.json
- Research/experiments folder scaffolding for research and product flavors
- `post_create_command` and `vscode_extensions` in devcontainer.json

## Recently Completed

### Yazi File Manager and IDE Layouts (v0.3.7)

Replaced Strider with Yazi as the default file manager in all layouts. Added three IDE layouts: **dev** (VS Code-like, default), **focus** (single-task, stacked panes), and **assist** (Claude-focused with center stage). LaTeX Workshop extension settings for LaTeX images.

### Update Upgrade Command (v0.3.6)

`dev-box update` now performs actual upgrades: fetches latest image version from GHCR, updates `dev-box.toml`, regenerates container files. Supports `--check` (read-only) and `--dry-run` (preview without writing).

### AI Config, User Support, .dev-box-home (v0.3.5)

`[ai]` config section with `providers` field. `container.user` for non-root containers. Renamed `.root/` to `.dev-box-home/`. Language-specific `.gitignore` blocks. Research/experiments scaffolding for research and product flavors.

### postCreateCommand and vscode_extensions (v0.3.1)

`post_create_command` and `vscode_extensions` fields in `[container]` config, generated into devcontainer.json. Pinned MkDocs dependency to `<2`.

### Shell Completions, Interactive Init, Update Checking, Template Engine (v0.3.0)

`dev-box completions <shell>` for all major shells. Interactive init prompts. Registry-based version checking. Minijinja template engine for file generation.

### Dockerfile.local (v0.2.3)

Project-specific Dockerfile layers appended to the generated Dockerfile, with `AS dev-box` stage alias for multi-stage builds.

## Planned

### Evaluate Zensical as MkDocs Successor

MkDocs 2.0 introduces breaking changes. Evaluate Zensical and other alternatives for documentation generation, or pin to a stable MkDocs version.

### Automated Context Migration

When upgrading between schema versions, `dev-box doctor` will generate migration artifacts. A future version may automate safe migrations (additive changes) while prompting for manual review on breaking changes.

### Additional Image Flavors

Potential new flavors based on demand:

- **node** — Node.js LTS via NodeSource
- **go** — Go toolchain
- **python-rust** — Python + Rust combined

### Plugin System

Extensibility mechanism for custom commands and image overlays without forking.
