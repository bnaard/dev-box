# aibox v0.31.0 — 2026-08-07

**Summary:** This minor release lets derived projects opt into rootless Podman and Podman Compose inside their development container, alongside the existing curated Go supply-chain and release bundles. Projects that need an internal container runtime can enable the new infrastructure tool and rebuild.

## Added

- Add an optional `podman` infrastructure tool with Podman Compose and rootless runtime prerequisites.
- Add companion-backed E2E coverage that starts a generated container and verifies nested rootless Podman operation.

## Changed

- Document the existing Go `supply-chain` bundle (`gitleaks`, `osv-scanner`, `syft`, `grype`, and `cosign`) and `release` bundle (`goreleaser`, `shellcheck`, and `hadolint`).
- Refresh curated fzf, uv, and Zensical versions and apply compatible Rust dependency updates after a clean security audit.

## Fixed

- Render a valid infrastructure builder stage when OpenTofu and Packer are both disabled.

## Removed

- No configuration or addon surface was removed.

## Upgrade notes

Enable the runtime with `[addons.go.infrastructure.tools] podman = {}` (or the equivalent language group), then run `aibox apply` and rebuild the container. Existing projects remain unchanged because Podman is disabled by default.

[v0.31.0]: https://github.com/projectious-work/aibox/compare/v0.30.1...v0.31.0
