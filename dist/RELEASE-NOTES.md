# aibox v0.30.0 — 2026-08-05

**Summary:** This minor release gives Go teams production-ready quality, security, supply-chain, and release tooling through composable language addon groups. Existing flat addon configurations remain compatible; opt into nested groups and rebuild to install the new tools.

## Added

- Add `[addons.go.quality]`, `[addons.go.supply-chain]`, and `[addons.go.release]` configuration groups.
- Add pinned Go quality tools: goimports, Staticcheck, golangci-lint, govulncheck, and gosec.
- Add language-neutral Gitleaks, OSV-Scanner, Syft, Grype, Cosign, ShellCheck, and Hadolint bundles.
- Add pinned GoReleaser support and the native prerequisites for `go test -race`.

## Changed

- Language addon definitions now expose consistent infrastructure, supply-chain/security, and release group mappings.
- Nested group tool versions and enabled states merge into their target addons while preserving explicit flat-addon overrides.
- Downloaded release assets use published checksum files wherever upstream provides them.

## Fixed

- Map Linux ARM64 architecture names to upstream release asset names, including Hadolint's `arm64` archive.

## Removed

- No existing addon or flat configuration surface was removed; legacy flat addon configuration remains supported.

## Upgrade notes

Add the desired nested Go groups to `aibox.toml`, then run `aibox apply` and rebuild the container. Existing flat addon configuration requires no migration.

[v0.30.0]: https://github.com/projectious-work/aibox/compare/v0.29.0...v0.30.0
