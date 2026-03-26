# Session Handover — 2026-03-26

## What Was Done

### BACK-060: Version Upgrade Flows

Implemented full version tracking across the CLI, config, and container image.

- `Dockerfile.j2` now embeds `LABEL aibox.version=<version>` and writes `/etc/aibox-version` in the image
- `runtime.rs`: added `get_container_image_version()` — reads the `aibox.version` Docker label via `docker inspect`
- `container.rs` (`cmd_start`): hard-errors with a clear message if running container image version mismatches `aibox.toml`; suggests `aibox sync`
- `update.rs`: added `--yes`/`-y` flag; update flow now prompts for confirmation and rebuild; `sync_config_files()` helper; removed stale `.aibox-version` write
- `doctor.rs`: two new checks — container image label mismatch warning, CLI version file (`.aibox-version`) mismatch warning
- `tests/e2e/infra/mock-docker.sh` + `mock-podman.sh`: fixed `--format` argument parsing bug; added `MOCK_CONTAINER_VERSION` env var
- `tests/e2e/version_upgrade.rs`: 8 new Tier 1 tests covering all flows

### BACK-052: Addon Dependency Tree

- Audited circular dep detection — Kahn's algorithm in `addons.rs` is correct; added 2 explicit cycle tests (self-loop, 3-node)
- `addon info` now prints `Requires:` line for addons with dependencies
- Research doc: `context/research/addon-dependency-design-2026-03.md`
- Finding: max dep depth in current corpus is 1; topological sort is sufficient; no conflict detection needed

### BACK-062: New Starship Prompt Preset

- New `arrow` preset: powerline/chevron-style with git status (ahead/behind, staged/modified/untracked), command duration; requires Nerd Font
- `plain` preset was already fully implemented
- Docs (`docs-site/docs/customization/prompts.md`) rewritten with all 7 presets, font requirements table
- Asciinema recording TODOs documented in `scripts/record-asciinema.sh` (requires running container, deferred)

### BACK-043: AI Provider Research

- Research report: `context/research/ai-provider-integrations-2026-03.md`
- Recommendations: add OpenAI Codex CLI, GitHub Copilot CLI, Continue CLI (→ BACK-064); re-evaluate ai-mistral SDK addon (→ BACK-065)

### Docs & E2E Catalogue

- `docs-site/docs/contributing/e2e-tests.md`: verbal "if X then Y" descriptions of all ~61 E2E tests across 11 modules, with test references
- `docs-site/sidebars.js`: wired new e2e-tests page

### Release v0.13.0

- Version bumped in `cli/Cargo.toml`
- `cargo-audit` added to `.devcontainer/Dockerfile` and `addons/languages/rust.yaml`
- Fixed `resvg` fetch stage: aarch64 now produces a stub shell script so `COPY --from=fetch-resvg` succeeds on Apple Silicon
- Linux binaries built and uploaded; macOS binaries built and uploaded by host script
- GitHub release: https://github.com/projectious-work/aibox/releases/tag/v0.13.0
- Docs deployed to https://projectious-work.github.io/aibox/

### BACK-063: Image Size Quick Wins Noted

Base image ships at >900 MB — significantly above the ~465 MB estimate in the Dockerfile comment.
Root cause: `ffmpeg` alone installs ~450 MB of codec libraries.
Quick wins documented in BACK-063 for next session (see below).

---

## What Needs to Happen Next

### Immediate: Rebuild Dev-Container

The `.devcontainer/Dockerfile` changed this session (added `cargo-audit` install to the Rust toolchain block). Rebuild is required for `cargo audit` to be available without `cargo install`.

After rebuild, validate:
```bash
cargo audit   # should run without needing to install first
```

### Next Session: BACK-063 Image Size Quick Wins

Target: get base image from >900 MB to under 500 MB. Do these in order:

1. **Remove `ffmpeg`** from `images/base-debian/Dockerfile` — saves ~450 MB. Video thumbnail support in yazi is not a priority. Move to a new `preview-enhanced` addon.
2. **Remove `ghostscript`** — `mupdf-tools` covers all PDF rendering; ghostscript is redundant. Saves ~70 MB.
3. **Review `poppler-utils`** — `mutool draw` can replace `pdftoppm`; evaluate removing it. Saves ~50 MB.
4. **Review `imagemagick`** — check what formats `chafa` already handles natively (AVIF/WebP/HEIC via librsvg/libheif); assess whether imagemagick is still needed.
5. **Research Rust-native alternatives** — look for musl-static Rust tools that replace heavy C packages (imagemagick, poppler, ghostscript). Candidates: `oxipng`, image-rs, `svg2png` crates, `pdf-rs`/`lopdf`. Research with web search before deciding.
6. **Create `preview-enhanced` addon** — move ffmpeg (and optionally timg for video thumbnails) there for users who need video support.
7. **Update Dockerfile comment** with accurate size estimates after changes.

### Backlog Items Created This Session

| ID | Title | Priority |
|---|---|---|
| BACK-064 | Add AI provider addons: OpenAI Codex CLI, GitHub Copilot CLI, Continue CLI | must |
| BACK-065 | Re-evaluate `ai-mistral` addon (SDK-only, not a CLI) | must |

Next backlog ID: **BACK-066**

---

## Current State

- Branch: `main`
- Version: `v0.13.0` — fully released (Linux + macOS binaries, GHCR images, docs)
- All tests passing (300 unit + E2E Tier 1)
- Dev-container rebuild required (Dockerfile changed for cargo-audit)
- No uncommitted changes
