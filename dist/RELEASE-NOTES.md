# aibox v0.14.0

## Breaking changes (backward-compatible)

Old config files continue to work via serde aliases — no manual migration required.

- **`[appearance]` renamed to `[customization]`** — new `layout` field sets default zellij layout (dev/focus/cowork/browse)
- **`[process]` merged into `[context]`** — `packages` now lives alongside `schema_version` in a single `[context]` section
- **`[container]` simplified** — removed `ports`, `extra_packages`, `extra_volumes`, `environment`, `vscode_extensions`. Use `Dockerfile.local` and `docker-compose.override.yml` for these customizations instead. Remaining fields: `name`, `hostname`, `user`, `keepalive`, `post_create_command`

## New features

- **Browse layout** — new yazi-focused layout with large file preview pane. Select via `aibox start --layout browse` or set as default in `[customization] layout = "browse"`
- **Configurable default layout** — `[customization] layout` in aibox.toml sets the default for `aibox start` (overridable with `--layout`)
- **`[skills]` in generated config** — `aibox init` now includes the `[skills]` section with include/exclude in the generated aibox.toml (was previously missing)
- **SVG preview on aarch64** — svg.yazi plugin now falls back to `rsvg-convert` when `resvg` is unavailable (aarch64). Base image adds `librsvg2-bin`

## Fixes

- **Prompt options comment** — generated aibox.toml now lists all 7 presets (was missing `plain` and `arrow`)
- **SSH key security** — removed e2e test SSH private keys from version control. Keys are now generated locally in `.aibox-e2e-runner-home/.ssh/` (gitignored) and mounted via docker-compose

## Documentation

- **AI Providers** — provider docs (Claude, Aider, Gemini, Mistral) moved to dedicated "AI Providers" sidebar chapter
- **Layouts page** — new `customization/layouts.md` documenting all 4 zellij layouts
- **Container config** — updated to explain `Dockerfile.local` and `docker-compose.override.yml` as the customization mechanism for packages, ports, volumes, and environment
- **Config reference** — fully updated for all section renames and field changes

## Research

Five new design documents in `context/research/`:
- Preview companion container (PROJ-004) — browser-based preview for images, PDFs, SVGs, Excalidraw
- Remote development patterns (BACK-011)
- Skill customization system (BACK-051)
- Kubernetes/Helm deployment (BACK-068)
- Event log design (BACK-073)

## Container Images

- `ghcr.io/projectious-work/aibox:base-debian-v0.14.0`
- `ghcr.io/projectious-work/aibox:base-debian-latest`

## CLI Binaries

- `aibox-v0.14.0-aarch64-unknown-linux-gnu.tar.gz`
- `aibox-v0.14.0-x86_64-unknown-linux-gnu.tar.gz`
- `aibox-v0.14.0-aarch64-apple-darwin.tar.gz` (added in Phase 2)
- `aibox-v0.14.0-x86_64-apple-darwin.tar.gz` (added in Phase 2)
