# Roadmap

This page outlines planned features and improvements for dev-box.

## Current — v0.1.0

The initial release includes:

- Rust CLI with full container lifecycle (init, generate, build, start, stop, attach, status, doctor, update)
- 8 container image flavors (base, python, latex, typst, rust, python-latex, python-typst, rust-latex)
- `dev-box.toml` configuration system
- 4 work process flavors (minimal, managed, research, product)
- Context scaffolding with OWNER.md sharing
- Audio support (PulseAudio bridging)
- Install script for pre-built binaries

## Planned

### Registry-Based Update Checking

`dev-box update` will query GHCR for newer image tags and compare against the version in `dev-box.toml`, enabling one-command upgrades.

### Automated Context Migration

When upgrading between schema versions, `dev-box doctor` will generate migration artifacts. A future version may automate safe migrations (additive changes) while prompting for manual review on breaking changes.

### Template Engine Migration

Replace string formatting in `generate.rs` with minijinja templates for better maintainability and extensibility.

### Post-Create Script Support

Support for `postCreateCommand` in `dev-box.toml` to handle setup steps that run after container creation (e.g., installing project-specific tools, setting git identity).

### Additional Image Flavors

Potential new flavors based on demand:

- **node** — Node.js LTS via NodeSource
- **go** — Go toolchain
- **python-rust** — Python + Rust combined

### Linux x86_64 Binary

Add `x86_64-unknown-linux-gnu` target to CI builds and release artifacts.

### Plugin System

Extensibility mechanism for custom commands and image overlays without forking.
