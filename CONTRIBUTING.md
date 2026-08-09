# Contributing to aibox

## Prerequisites

- Rust stable toolchain (`rustup`)
- `cargo` on PATH
- The dev container (`.devcontainer/`) is the recommended environment — it includes
  the cross-compiler for x86_64 and all release tooling.

## Build

```bash
cd cli && cargo build           # debug build
cd cli && cargo build --release # release build
```

## Test

```bash
# All tests (unit + integration + E2E tier 1 — no container needed)
cd cli && cargo test

# Lint — zero warnings required
cd cli && cargo clippy --all-targets -- -D warnings

# Format check
cd cli && cargo fmt -- --check
```

### Local E2E contracts

```bash
# 1. Build the CLI binary
cd /workspace/cli && cargo build

# 2. Run local E2E contracts. No Docker, SSH, or companion is used.
cd /workspace/cli && cargo test --test e2e

# Run a specific E2E test
cd /workspace/cli && cargo test --test e2e local_lifecycle
```

Real candidate-image lifecycle evidence runs only through the owner-controlled
macOS release host gate. Its core candidate lifecycle, native Darwin smoke,
cleanup, SBOM, and vulnerability checks run for every release. Checksummed
changed-path provenance additionally selects affected download-based addon
builds, the LaTeX watcher/preview lifecycle, and the rootless Podman probe.

### Visual E2E tiers

The generated tmux/Yazi visual tests run real isolated tmux/asciinema sessions
inside the development container. They are intentionally opt-in because they are slower and
because release validation should choose the tier that matches the changed
surface:

```bash
./scripts/maintain.sh test-e2e-visual-status # layouts, themes, tmux status rows
./scripts/maintain.sh test-e2e-visual-tabs   # tab traversal, tools, harnesses
./scripts/maintain.sh test-e2e-visual-yazi   # Yazi previews, git symbols, plugins
./scripts/maintain.sh test-e2e-visual        # all visual tiers
```

To generate current-release source artifacts for documentation screenshots or
screencasts:

```bash
./scripts/maintain.sh test-e2e-doc-captures
```

Set `AIBOX_E2E_VISUAL_ARTIFACT_DIR` to override the default output directory
(`docs-site/static/img/e2e/`).

## Before committing

```bash
cd cli && cargo test && cargo clippy --all-targets -- -D warnings
```

Both must be clean. `cargo audit` must also be clean before tagging a release.

Use the repository's canonical Git identity for release and maintenance
commits:

```bash
git config user.name projectious
git config --get user.name
git config --get user.email
```

The reported email must match the canonical maintainer identity configured for
the repository. Do not substitute a harness-generated or container-local
identity in release commits.

## Commit message format

Conventional commits: `feat:`, `fix:`, `chore:`, `docs:`.
Always reference GitHub issue numbers: `fixes #N`, `refs #N`.
Include `Cargo.lock` in version bump commits.

## Release

See `context/notes/NOTE-20260411_0000-LoyalSpruce-aibox-release-process.md` for the full release process.
Quick summary: `./scripts/maintain.sh release X.Y.Z` in the container prepares
an immutable run directory. On macOS, run the single run-directory command
printed in `dist/RELEASE-PROMPT.md`.

Release validation and publication run locally. Do not introduce GitHub Actions
or another hosted CI release path. The local release tooling runs independent
gates concurrently, records exact-commit evidence under `dist/release-evidence/`,
and reuses that evidence only when its source, toolchain, environment scope,
and produced artifact checksums still match.

## Repository layout

| Path | Owns |
|------|------|
| `cli/` | The Rust CLI — the only shipped artifact besides addon YAMLs |
| `addons/` | YAML addon definitions (python, rust, node, latex, …) |
| `images/` | Container image build recipes published to GHCR |
| `docs-site/` | Hugo/Docsy documentation site |
| `scripts/` | Release and maintenance tooling |
| `context/` | This project's context (workitems, decisions, notes, …) |

**Key Rust modules:**

| Module | Responsibility |
|--------|---------------|
| `cli/src/main.rs` | Entry point, tracing setup, dispatch |
| `cli/src/cli.rs` | clap derive-based arg parsing |
| `cli/src/config.rs` | `aibox.toml` deserialization (serde + toml) |
| `cli/src/generate.rs` | Dockerfile / compose / devcontainer.json generation |
| `cli/src/container.rs` | `up` / `down` / runtime lifecycle helpers |
| `cli/src/content_source.rs` | processkit release-asset fetcher with fallback strategies |
| `cli/src/content_install.rs` | Install map — where each processkit file lands |
| `cli/src/content_init.rs` | `install_content_source` orchestration; templates mirror |
| `cli/src/content_diff.rs` | Three-way diff; migration document generation |
| `cli/src/mcp_registration.rs` | Per-harness MCP server registration |
| `cli/src/processkit_vocab.rs` | **Central constants module** — all processkit vocabulary |
| `cli/src/addon_loader.rs` | YAML addon loading and template context building |
| `cli/src/seed.rs` | `.aibox-home/` runtime config seed |
| `cli/src/doctor.rs` | Diagnostic checks |
| `cli/src/runtime_resources.rs` | cgroup/procfs runtime resource snapshots |
| `cli/src/context.rs` | Project skeleton scaffolding, gitignore, provider thin pointers |

**Rule:** Never hardcode processkit path strings, filenames, or vocabulary in production
Rust source — add constants to `processkit_vocab.rs` instead.

## Critical distinction

**We are in a dev-container building dev-containers.**

- **`.devcontainer/`** — THIS project's dev environment (Rust + Python/uv + Hugo/Docsy).
- **`images/`** — Published images for OTHER projects (pushed to GHCR).

Never confuse these two. Changes to `.devcontainer/` affect our development.
Changes to `images/` affect downstream projects.
