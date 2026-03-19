# Development Instructions

## Project Structure

```
cli/                    Rust CLI source (the dev-box binary)
  src/
    main.rs             Entry point, tracing setup, dispatch
    cli.rs              clap derive-based arg parsing
    config.rs           dev-box.toml deserialization (serde + toml)
    generate.rs         Dockerfile / compose / devcontainer.json generation
    runtime.rs          podman / docker abstraction
    container.rs        build / start / stop / attach / status / init
    context.rs          context scaffolding + gitignore + doctor helpers
    doctor.rs           diagnostic checks + migration artifacts
    update.rs           registry version checking (GHCR + GitHub releases)
    seed.rs             .dev-box-home/ directory seeding
    audio.rs            host-side PulseAudio diagnostics and setup
    output.rs           ANSI-colored terminal output
    templates/          embedded Jinja2 templates (Dockerfile.j2, docker-compose.yml.j2)
images/                 Published images for downstream projects (8 flavors)
templates/              Work process flavor templates (4 flavors, 21 files)
schemas/                Context schema documents (versioned)
docs/                   MkDocs documentation source
scripts/                Build/release/maintenance scripts
```

## Building

```bash
cd cli && cargo build           # Debug build
cd cli && cargo build --release # Release build
```

## Testing

```bash
cd cli && cargo test                          # All tests (unit + integration)
cd cli && cargo clippy -- -D warnings         # Lint check
cd cli && cargo fmt -- --check                # Format check
```

## Config Spec — dev-box.toml

`dev-box.toml` is the single source of truth. All generated files derive from it.

Key sections:
- `[dev-box]` — version, image flavor, process flavor
- `[container]` — name, hostname, user, ports, extra_packages, extra_volumes, environment
- `[context]` — schema_version
- `[ai]` — providers (claude, etc.)
- `[audio]` — enabled, pulse_server

## Docker Image Architecture

8 images built from `images/`:
- **base** — debian:trixie-slim + zellij + vim + git + lazygit + gh + claude CLI + audio + unzip
- **python** — FROM base + python 3.13 + uv + mkdocs-material
- **latex** — FROM base + TeX Live (multi-stage CTAN install)
- **typst** — FROM base + Typst (static musl binary)
- **rust** — FROM base + rustup + cargo + clippy + rustfmt
- **python-latex**, **python-typst**, **rust-latex** — combinations

TeX Live uses multi-stage build: builder installs from CTAN (~2GB), runtime copies the tree.

## Container User Support

The `container.user` config controls:
- Mount paths inside container (e.g., `/root/.vim` vs `/home/user/.vim`)
- `remoteUser` in devcontainer.json
- `GIT_CONFIG_GLOBAL` env var path
