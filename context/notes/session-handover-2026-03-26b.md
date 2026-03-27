# Session Handover — 2026-03-26 (evening)

## Summary

Fixed critical `aibox update` bug, shipped v0.13.1 with compose override support and browse layout, then dogfooded the project's own devcontainer to use `aibox sync` instead of hand-maintained files.

## What was done

### Bug fixes
- **GHCR tag prefix mismatch** — `update.rs` searched for `debian-v*` tags but GHCR has `base-debian-v*` (broken since BACK-022 Phase 2 refactor). Fixed prefix to `format!("base-{}-v", flavor)`.
- **E2E companion not starting** — Added `depends_on` in docker-compose.yml so VS Code starts both containers.
- **E2E tests not validating registry** — Tests only asserted graceful 401 handling (images were accidentally private before). Rewrote to assert actual successful GHCR fetch.

### New features (v0.13.1)
- **BACK-066: docker-compose.override.yml support** — Scaffold during `aibox init`, auto-detect in `aibox sync`, wire into `devcontainer.json` as array. Uses Docker Compose standard strategic merge.
- **BACK-067: "browse" zellij layout** — Tab 1: yazi (60%) + AI pane (40%). Tab 2: vim. Tab 3: lazygit. Tab 4: bash. Uses `AIBOX_EDITOR_DIR=tab`.

### Release v0.13.1
- Full Phase 1 completed: tests, audit, Linux binaries (aarch64 + x86_64), GitHub release, docs deployed.
- Phase 2 completed by owner on macOS host (macOS binaries + container images pushed to GHCR).

### Dogfooding
- Replaced hand-maintained `.devcontainer/` with `aibox sync`-generated files.
- `Dockerfile.local` handles project-specific layers: cross-compilation toolchain, Rust (as USER aibox), bun, asciinema.
- `docker-compose.override.yml` contains the e2e test companion service with security options.
- Removed rust addon due to HOME path bug (BACK-074), installed Rust directly in Dockerfile.local.
- Added `USER root` at end of Dockerfile.local to work around template bug (BACK-075).

### Backlog
- Filed BACK-068 through BACK-075 (Kubernetes Helm charts, skill versioning, scheduled tasks, code audit, code walkthrough docs, event log, two bugs).
- Archived BACK-010, BACK-012, BACK-052, BACK-066 to archive.

## Open threads

1. **Container rebuild needed** — Owner needs to rebuild the VS Code devcontainer to validate the dogfooding changes. After rebuild, the e2e companion should be on the same Docker network, enabling the e2e update tests to actually run.
2. **BACK-074 (Rust addon HOME bug)** — Must-fix before other projects can use the rust addon. Fix is straightforward: add `ENV CARGO_HOME=/home/aibox/.cargo RUSTUP_HOME=/home/aibox/.rustup` to the builder stage in `addons/languages/rust.yaml`.
3. **BACK-075 (Dockerfile template /etc write)** — Must-fix: add `USER root` before the version stamp in `Dockerfile.j2`. Currently worked around in this project's Dockerfile.local.

## Decisions made

- **Compose override over custom multi-service** — Use Docker Compose standard `docker-compose.override.yml` rather than inventing custom mechanisms. Resolves BACK-010.
- **No circular dependency** — Dogfooding is safe because the CLI is host-side only and pulls released images from GHCR.
- **Rust in Dockerfile.local, not addon** — Due to BACK-074, Rust is installed in Dockerfile.local as USER aibox. Once the addon bug is fixed, this can migrate back to `[addons.rust]`.

## Blockers

None — waiting on container rebuild to validate.

## Next steps

1. Rebuild container, verify e2e companion is reachable, run `cargo test --features e2e --test e2e update::` to validate the registry fetch tests.
2. Fix BACK-074 (Rust addon builder HOME) and BACK-075 (template USER root) — these are quick fixes, should be in the next patch release.
3. Consider: once bugs are fixed, remove Rust from Dockerfile.local and switch to `[addons.rust]` in aibox.toml.

## Context for next agent

- The `.devcontainer/config/` directory still exists and is needed — it's used by the base image build (`images/base-debian/Dockerfile`), not the devcontainer itself.
- E2E SSH keys are in `.aibox-e2e-runner-home/.ssh/` (gitignored, mounted into companion container). Previously in `.devcontainer/ssh-e2e/` which was a security issue (private key in VCS).
- `aibox sync --no-build` was used because no container runtime is available inside this Claude Code session.
- The `docs-docusaurus` addon requires the `node` addon explicitly (`requires: [node]` in YAML).
