---
sidebar_position: 2
title: Maintenance
---

# Maintenance

Internal procedures for building, testing, documenting, and releasing aibox.
The canonical step-by-step release note remains in `context/notes/`; this page
is the public contributor summary.

## Development Checks

```bash
cd cli && cargo fmt -- --check
cd cli && cargo clippy --all-targets -- -D warnings
cd cli && cargo test
```

The helper script wraps the same checks:

```bash
./scripts/maintain.sh test
```

## Documentation Site

The public docs live in `docs-site/` and use Docusaurus.

```bash
cd docs-site
npm install
npm start
npm run build
```

The maintenance script also exposes:

```bash
./scripts/maintain.sh docs-serve
./scripts/maintain.sh docs-deploy --dry-run
./scripts/maintain.sh docs-deploy
```

`docs-deploy` builds the site and pushes the static output to the `gh-pages`
branch. The release script runs it as part of the container-side release phase.

## Published Image

aibox publishes the base Debian image used by generated downstream projects:

```bash
./scripts/maintain.sh build-images
./scripts/maintain.sh build-images --no-cache
./scripts/maintain.sh push-images X.Y.Z
```

Generated project images are built per project by `aibox apply`. They are not
published from this repository.

## Release Boundary

Releases are intentionally split:

| Phase | Where | Command | Purpose |
| --- | --- | --- | --- |
| Container side | aibox devcontainer | `./scripts/maintain.sh release X.Y.Z` | check dependency/harness state, sync processkit default, bump CLI version, test, audit, build Linux binaries, tag, create GitHub release, deploy docs |
| Host side | macOS host | `./scripts/maintain.sh release-host X.Y.Z` | build macOS binaries, upload them to the release, build and push GHCR images, run the generated-runtime smoke, then refresh repo-owned runtime surfaces |

Do not create GitHub releases by hand with `gh release create`. The release
script attaches binaries and writes the release notes expected by users.

## Container-Side Release

```bash
./scripts/maintain.sh release X.Y.Z
```

This command requires a clean working tree. It may stop after
`sync-processkit` if a newer processkit release changes the pinned default and
the CLI needs review before release.

The command performs:

- dependency, addon, image, and harness state report in `dist/RELEASE-STATE.md`
- processkit release sync check
- `cli/Cargo.toml` and `Cargo.lock` version bump when needed
- format, Clippy, and test checks
- Tier 2 SSH companion E2E tests, including generated runtime and visual
  asciinema probes
- `cargo audit`
- `cargo update --dry-run` review for lockfile-resolvable crate updates
- Linux release builds for `aarch64-unknown-linux-gnu` and
  `x86_64-unknown-linux-gnu`
- binary version smoke check
- annotated git tag push
- GitHub release creation with Linux binaries
- Docusaurus docs deployment
- `dist/RELEASE-PROMPT.md` for host-side completion

Run `./scripts/maintain.sh release-check-state` standalone when you want the
same pre-release report without bumping, tagging, or building.

If a report finding is deferred, create a processkit WorkItem before continuing
the release and mention that WorkItem ID in the release notes or handover.
For `cargo update --dry-run`, either apply available crate updates in the
release with full validation, or create a WorkItem for the deferred
crate-update pass.

## Host-Side Release

Run this on the macOS host after the container-side release succeeds:

```bash
./scripts/maintain.sh release-host X.Y.Z
```

This phase builds Darwin binaries, uploads them to the existing GitHub release,
pushes GHCR images, then runs a fresh downstream-style runtime smoke against
the pushed release tag. The smoke creates a temporary project, runs
`aibox init` and `aibox apply --no-cache --standardize-config`, starts the
generated container, probes Yazi, the aibox status helper, and Zellij
sidecar status-plugin logs, and writes a bundle to
`dist/release-smoke/vX.Y.Z/<timestamp>/`.
It is host-side because macOS binaries and host runtime access are not
available from the Linux devcontainer.

## Verification

After release:

- `gh release view vX.Y.Z` shows all expected binary assets.
- `curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | VERSION=X.Y.Z bash` installs the expected version.
- `aibox --version` reports `X.Y.Z`.
- `docker pull ghcr.io/projectious-work/aibox:base-debian-vX.Y.Z` or the matching Podman pull succeeds.
- The docs site at `https://projectious-work.github.io/aibox/` reflects the release.

## Project Devcontainer

The maintenance script can also operate this repository's own devcontainer:

```bash
./scripts/maintain.sh start
./scripts/maintain.sh status
./scripts/maintain.sh attach
./scripts/maintain.sh stop
```

Do not confuse `.devcontainer/` in this repository with `images/`. The former
is the environment used to develop aibox. The latter contains image recipes
published for downstream projects.
