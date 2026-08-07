# aibox v0.30.1 — 2026-08-07

**Summary:** This patch release makes container rebuilds honor Codex `latest`, hardens curated pnpm and Tau installations, and improves runtime and release validation reliability. Projects using `latest` or curated defaults should run `aibox apply` and rebuild their container.

## Added

- Add candidate-bound, independently retryable release validation evidence and sharded Tier-2 E2E execution.

## Changed

- Update the curated pnpm default to 11.20.0 and Tau default to 0.3.7 while retaining earlier versions as explicit pins.
- Refresh the E2E companion contract and validation for its current tmux, Yazi, Podman, and bubblewrap runtime.

## Fixed

- Resolve Codex `latest` to a concrete upstream version before Docker generation so rebuilds do not reuse a stale cached npm layer.
- Isolate Starship cache files per shell to prevent cache-directory collisions in generated and image runtime configurations.

## Removed

- No configuration or addon surface was removed.

## Upgrade notes

Run `aibox apply` and rebuild the container to receive newly resolved `latest` pins and curated addon defaults. Existing explicit addon pins remain unchanged.

[v0.30.1]: https://github.com/projectious-work/aibox/compare/v0.30.0...v0.30.1
