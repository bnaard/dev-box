---
weight: 2
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

The public docs live in `docs-site/` and use Hugo with the Docsy theme.

```bash
git submodule update --init --recursive docs-site/themes/docsy
npm --prefix docs-site ci
./scripts/maintain.sh docs-serve
./scripts/build-docs.sh
```

`./scripts/build-docs.sh` is the local verification build. It writes the static
site to `docs-site/public/` and prints Hugo render warnings.

The maintenance script also exposes:

```bash
./scripts/maintain.sh docs-serve
./scripts/maintain.sh docs-deploy --dry-run
./scripts/maintain.sh docs-deploy
```

`docs-deploy` builds the site and pushes the static output to the `gh-pages`
branch from the local checkout. It does not use GitHub Actions. Use
`--dry-run` to validate the production build without pushing; use the command
without that flag only when the current checkout is the source that should be
published. The release script runs the same deployment as part of the
container-side release phase.

Use the repository maintenance command for publication rather than
`npm run deploy`. The maintenance command preserves the project's local-only
release flow, publishes the already configured `/aibox/` site, and ensures
GitHub Pages serves the `gh-pages` branch.

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

Both phases run locally. The project deliberately does not use GitHub Actions
for release validation, artifact builds, image publication, or deployment.
Release speed comes from bounded local concurrency, persistent caches, and
reuse of evidence for the exact release commit rather than from moving gates to
a hosted runner.

## Version-line branches

Long-lived branches are protected: direct pushes, force-pushes, and deletion
are disabled; changes arrive through pull requests with resolved conversations.
No GitHub Actions or required hosted checks are used.

| Line | Development | Release authority | Purpose |
| --- | --- | --- | --- |
| v0 maintenance | `v0.x-dev` | `v0.x-release` | Stable v0 releases and hotfixes |
| v1 prerelease | `v1.x-dev` | `v1.x-pre-release` | Alpha, beta, and release-candidate tags |
| v1 GA | `v1.x-dev` | `v1.x-release` (created at GA) | Stable v1 releases |

`main` is the published-history branch. After a tag is cut on its designated
release branch, merge that branch into `main` through a pull request. Apply or
verify the policy from an administrator checkout with:

```bash
./scripts/configure-branch-protection.sh
```

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
- processkit and host-context aibox doctor runs in `dist/RELEASE-DOCTORS.md`;
  doctor errors block the release and warnings remain visible for review
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
- Hugo/Docsy docs deployment
- `dist/RELEASE-PROMPT.md` for host-side completion

Independent validation gates run concurrently. The default worker limit is
two; set `AIBOX_RELEASE_PARALLELISM` to a positive integer that fits the local
machine. Linux release targets build concurrently inside the build gate, and
the version smoke reuses the matching release artifact instead of compiling a
third binary.

Successful gates write local evidence under
`dist/release-evidence/vX.Y.Z/<commit>/`. Evidence is bound to the exact commit,
Rust toolchain, clean-tree state, release phase, and gate-specific environment.
Companion evidence includes the companion fingerprint, audit evidence expires
daily, and binary evidence rechecks archive checksums. Set
`AIBOX_RELEASE_REUSE_EVIDENCE=0` to force every selected gate to run again.
Container-side timings are written to `dist/RELEASE-TIMINGS.md`.

Run `./scripts/maintain.sh release-check-state` standalone when you want the
dependency and tool-state report without bumping, tagging, or building. Run
`./scripts/maintain.sh release-doctors` for the matching diagnostic report.

If a report finding is deferred, create a processkit WorkItem before continuing
the release and mention that WorkItem ID in the release notes or handover.
For `cargo update --dry-run`, either apply available crate updates in the
release with full validation, or create a WorkItem for the deferred
crate-update pass.

## Host-Side Release

Run this on the macOS host after the container-side release succeeds. Sync the
matching version-line release branch first; the container-side release may have
pushed tag-prep commits from another clone. For a v0 release:

```bash
git fetch origin v0.x-release
git switch v0.x-release
git reset --keep origin/v0.x-release
./scripts/maintain.sh release-host X.Y.Z
```

`release-host` derives the protected release branch from the version:
`v0.x-release` for v0, `v1.x-pre-release` for v1 prereleases, and
`v1.x-release` for v1 GA. It fetches only that branch and the requested tag,
then verifies that the tag is reachable from the branch before building.

This phase builds Darwin binaries, uploads them to the existing GitHub release,
pushes GHCR images, then runs a fresh downstream-style runtime smoke against
the pushed release tag. The smoke creates a temporary project, runs
`aibox init` and `aibox apply --no-cache --standardize-config`, starts the
generated container, probes Yazi, the aibox status helper, tmux state, and the
diagnostics sidecar, and writes a bundle to
`dist/release-smoke/vX.Y.Z/<timestamp>/`.
By default, this smoke runs with `AIBOX_RELEASE_SMOKE_TIER=addons`, so `git-ui`
(`lazygit`) startup is exercised in addition to the core runtime contract.
It is host-side because macOS binaries and host runtime access are not
available from the Linux devcontainer.

The two macOS targets build concurrently. The host release also overlaps that
build lane with source-hash-aware image reuse or publication, then joins both
lanes before uploading binaries and starting the runtime smoke. Healthy tmux
smoke probes advance on observed session, window, pane, and status readiness;
their timeouts are failure ceilings rather than fixed delays. Host timings are
written to `dist/RELEASE-HOST-TIMINGS.md`.

The Linux-side Tier 2 E2E companion is separate from this host phase. From the
devcontainer, verify that companion over SSH/SCP; do not use local
Docker/Podman availability in the main devcontainer as the reachability check.

`release-doctors` is an aibox CLI development exception to the normal
host/container diagnostic split. Inside the workspace container, ordinary
dogfood diagnostics use `pk-doctor`; `aibox doctor` is host-side. During release
Phase 0, however, `./scripts/maintain.sh release-doctors` runs `aibox doctor`
as a host-context simulation so the CLI's host diagnostic behavior remains
gated.

## Verification

After release:

- `gh release view vX.Y.Z` shows all expected binary assets.
- `curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | VERSION=X.Y.Z bash` installs the expected version.
- `aibox --version` reports `X.Y.Z`.
- For `v0.26.x`, `docker pull ghcr.io/projectious-work/aibox:base-debian-vX.Y.Z` or the matching Podman pull succeeds.
- For `v0.27.0+`, `docker pull ghcr.io/projectious-work/aibox:base-debian-runtime-vX.Y.Z` and `base-debian-runtime-latest` succeed. Foundation images are published as `base-debian-foundation-vX.Y.Z`; source-hash marker tags are not published.
- To remove historical source-hash marker tags from GHCR, first run `./scripts/maintain.sh ghcr-prune-source-tags --repair-mixed` and review the mixed-version repair plan. Then run `./scripts/maintain.sh ghcr-prune-source-tags --repair-mixed --execute` with `read:packages`, `delete:packages`, and Docker Buildx available on the host.
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
