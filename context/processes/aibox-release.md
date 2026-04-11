---
apiVersion: processkit.projectious.work/v1
kind: Process
metadata:
  id: PROC-20260411_0713-warmCliff-aibox-release
  version: "1.0.0"
  created: 2026-04-11T07:13:00Z
spec:
  name: aibox-release
  description: "Cut and publish a versioned aibox CLI release — two-phase: Linux build in container (Phase 1), macOS build + image push on host (Phase 2)."
  triggers:
    - release.requested
  roles:
    - agent         # runs Phase 1 inside the container
    - maintainer    # runs Phase 2 on the macOS host
  steps:

    # ── Phase 1 (agent, inside container) ─────────────────────────────────

    - name: preflight-build-check
      role: agent
      description: >
        Verify `cargo build --release --target aarch64-unknown-linux-gnu`
        succeeds. If it fails (e.g. missing linker after a container config
        change), stop immediately and tell the maintainer — do not proceed
        to create any release artefacts. A release without binaries is
        worse than no release.
      gates:
        - GATE-cargo-build-succeeds

    - name: commit-version-bump
      role: agent
      description: >
        Ensure all version strings are aligned: `cli/Cargo.toml`,
        `cli/Cargo.lock`, `aibox.lock` (`cli_version`), and a new
        `CompatEntry` in `cli/src/compat.rs`. Commit with
        `chore: bump version to vX.Y.Z`. Include `Cargo.lock` in the commit.
        Do NOT create the git tag yet.

    - name: run-maintain-release
      role: agent
      description: >
        From inside the container, run:

            ./scripts/maintain.sh release X.Y.Z

        This single command:
          1. Checks for a newer processkit release and patches constants
          2. Runs `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`
          3. Runs `cargo audit`
          4. Builds Linux binaries for `aarch64-unknown-linux-gnu` and
             `x86_64-unknown-linux-gnu`
          5. Creates and pushes the annotated git tag `vX.Y.Z`
          6. Creates the GitHub release with Linux binaries attached
          7. Deploys documentation to gh-pages
          8. Writes `dist/RELEASE-PROMPT.md` with the Phase 2 instructions

        NEVER replace this with a manual sequence of git, gh, and cargo
        commands — the result will be a release with no binary assets.
      gates:
        - GATE-tests-green
        - GATE-audit-clean
        - GATE-cargo-build-succeeds

    - name: hand-off-to-maintainer
      role: agent
      description: >
        Read `dist/RELEASE-PROMPT.md` and present its contents to the
        maintainer verbatim. The release is incomplete until Phase 2 is done.

    # ── Phase 2 (maintainer, macOS host) ──────────────────────────────────

    - name: build-and-upload-macos
      role: maintainer
      description: >
        On the macOS host, run:

            ./scripts/maintain.sh release-host X.Y.Z

        This builds macOS binaries (`aarch64-apple-darwin`,
        `x86_64-apple-darwin`), uploads them to the existing GitHub release,
        then builds and pushes container images to GHCR.

    - name: verify-release
      role: maintainer
      description: >
        Confirm the GitHub release at
        https://github.com/projectious-work/aibox/releases/tag/vX.Y.Z
        has all four binary archives attached:
          - `aibox-vX.Y.Z-aarch64-apple-darwin.tar.gz`
          - `aibox-vX.Y.Z-x86_64-apple-darwin.tar.gz`
          - `aibox-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz`
          - `aibox-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
      gates:
        - GATE-artifacts-verified

  definition_of_done: >
    Git tag pushed; GitHub release exists with all four binary archives
    attached; documentation deployed; GHCR images pushed and pullable.

  retryable: false

---

# aibox Release Process

## Overview

Releases are two-phase. The agent (Claude, running inside the aibox
devcontainer) owns Phase 1. The maintainer (on the macOS host) owns Phase 2.
Neither phase can be skipped or reordered.

## Why two phases?

Cross-compiling for Darwin (`apple-darwin`) targets from a Linux container is
not supported by the Rust toolchain. macOS binaries can only be built on a
macOS host. Container images are also built and pushed from the host because
the Docker daemon is not available inside the devcontainer.

## Retrofit recipe (for orphaned releases)

If a release was created manually (missing binaries), recover as follows:

**Linux binaries** — inside the container, after verifying `cargo build --release`
succeeds:

```bash
VERSION=X.Y.Z
mkdir -p dist
for target in aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu; do
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    cargo build --release --target "${target}" --manifest-path cli/Cargo.toml
  name="aibox-v${VERSION}-${target}"
  cp "cli/target/${target}/release/aibox" "dist/${name}"
  tar -czf "dist/${name}.tar.gz" -C dist "${name}"
  rm "dist/${name}"
  gh release upload "v${VERSION}" "dist/${name}.tar.gz" \
    --repo projectious-work/aibox
done
```

**macOS binaries** — on the macOS host:

```bash
./scripts/maintain.sh release-host X.Y.Z
```

## Precondition checklist

Before running `maintain.sh release`:

- [ ] `cargo build --release --target aarch64-unknown-linux-gnu` passes
- [ ] Working tree is clean (`git status --porcelain` is empty)
- [ ] Version strings aligned in `Cargo.toml`, `Cargo.lock`, `aibox.lock`, `compat.rs`
- [ ] Git tag `vX.Y.Z` does not already exist

## Known current state (as of 2026-04-11)

Releases v0.17.13 and v0.17.14 were created manually and have no binary
assets. Use the retrofit recipe above once the container is rebuilt with
the gcc/cross-compile toolchain (Rust addon v1.1.0 fix).
