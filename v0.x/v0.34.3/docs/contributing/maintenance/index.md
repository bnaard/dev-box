# Maintenance


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

The public docs live in `docs-site/` and use Hugo with the pinned
`brand-theme-hugo-vanilla` module. Hugo Extended, Go, and Node.js are required;
the project devcontainer enables all three through aibox addons.

```bash
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
| Host side | macOS host | run-directory command from `dist/RELEASE-PROMPT.md` | validate immutable inputs, build/smoke Darwin and candidate images without credentials, emit evidence, then publish the fixed manifest |

Both phases run locally. The project deliberately does not use GitHub Actions
for release validation, artifact builds, image publication, or deployment.
Release speed comes from bounded local concurrency, persistent caches, and
reuse of evidence for the exact release commit rather than from moving gates to
a hosted runner.

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
- local temporary-workspace E2E and isolated tmux/asciinema probes
- `cargo audit`
- `cargo update --dry-run` review for lockfile-resolvable crate updates
- Linux release builds for `aarch64-unknown-linux-gnu` and
  `x86_64-unknown-linux-gnu`
- binary version smoke check
- annotated git tag push
- GitHub release creation with Linux binaries
- Hugo/projectious.work brand-theme docs deployment
- `dist/RELEASE-PROMPT.md` for host-side completion

Independent validation gates run concurrently. The default worker limit is
two; set `AIBOX_RELEASE_PARALLELISM` to a positive integer that fits the local
machine. Linux release targets build concurrently inside the build gate, and
the version smoke reuses the matching release artifact instead of compiling a
third binary.

Successful gates write local evidence under
`dist/release-evidence/vX.Y.Z/<commit>/`. Evidence is bound to the exact commit,
Rust toolchain, clean-tree state, release phase, and gate-specific environment.
Audit evidence expires daily, and binary evidence rechecks archive checksums. Set
`AIBOX_RELEASE_REUSE_EVIDENCE=0` to force every selected gate to run again.
Container-side timings are summarized in `dist/RELEASE-TIMINGS.md`; host-side
timings are summarized in `dist/RELEASE-HOST-TIMINGS.md`. Each summary is a
cumulative view of the append-only `timing-events.tsv` file beside the
candidate evidence, so retries and resumed release commands retain failed,
passed, and reused gate attempts instead of replacing the earlier timings.

Run `./scripts/maintain.sh release-check-state` standalone when you want the
dependency and tool-state report without bumping, tagging, or building. Run
`./scripts/maintain.sh release-doctors` for the matching diagnostic report.

If a report finding is deferred, create a processkit WorkItem before continuing
the release and mention that WorkItem ID in the release notes or handover.
For `cargo update --dry-run`, either apply available crate updates in the
release with full validation, or create a WorkItem for the deferred
crate-update pass.

## Host-Side Release

The container-side release prepares a checksummed source archive and provenance
record under `tmp/host-gates/aibox-release/<run-id>/input/`. It writes the
single owner command to `dist/RELEASE-PROMPT.md`:

```bash
./scripts/maintain.sh release-host tmp/host-gates/aibox-release/<run-id>
```

For an evidence-only rehearsal, append `--dry-run`. Validation still performs
all candidate builds, probes, cleanup, SBOM generation, vulnerability scanning,
and manifest hashing, but it does not invoke GitHub or GHCR publication. The
validator prints the separate publisher command that can consume the verified
run directory later.

Interactive terminals use the Textual dashboard by default (`--ui=auto`). Use
`--ui=textual` to fail if the dashboard cannot start, or `--ui=plain` for
redirected and captured output. The UI presents a high-level progress bar, a
persistent task list, and a selectable task-filtered log. Space toggles follow,
`w` toggles wrapping, Ctrl+A/C selects and copies, `y` copies the selected task
log, End resumes the live tail, and `p` displays the authoritative evidence
path. The UI is presentation rather than evidence; full output is retained in
`evidence/command-results.log`.

Content-addressed container layers are reused by default. The Rust registry is
shared in a dedicated credential-free host-gate cache, while compiled artifacts
are isolated by candidate commit so a failed candidate can be retried without a
full two-target rebuild. Use `--cold-cache` only when deliberately investigating
cache behavior. Cold mode retains downloaded Rust packages but forces downstream
container layers to rebuild.

To retry after a conditional addon or lifecycle check fails, prepare a new run
and name the failed run as its checkpoint source:

```bash
NEW_RUN="$(./scripts/maintain.sh release-host-prepare X.Y.Z)"
./scripts/maintain.sh release-host \
  --retry-from=tmp/host-gates/aibox-release/<failed-run-id> "${NEW_RUN}"
```

The retry source must contain byte-identical immutable candidate inputs.
Completed conditional checks are reused only when their candidate-bound
checkpoint checksum is valid. The candidate lifecycle, SBOM, vulnerability
scan, cleanup, manifest assembly, and publication verification remain fresh.

The reviewed entry point accepts only that run-directory path. It rejects
traversal, symlinks, special files, hardlinks, unexpected inputs, unsafe
permissions, checksum drift, and tag/commit mismatches. A previous partial run
is never resumed in place: `runtime/` and `evidence/` must not exist in the new
run when it starts. Retry imports only validated conditional checkpoints.
The host checkout's `HEAD` must match the attested candidate commit, but
unrelated tracked worktree edits do not invalidate the gate because all builds
and probes use the immutable checksummed source archive rather than worktree
contents.

Validation and publication are separate security stages within the one owner
invocation. Candidate-controlled native builds, build scripts, CLI commands,
and the generated runtime smoke receive a fixed environment and run under a
macOS sandbox that denies GitHub configuration, Docker configuration, SSH
material, and Keychain services. Docker uses an empty per-run configuration;
no publication credential, secret, broad mount, or runtime socket is exposed
to the development container or candidate container.

The entry point accepts owner-installed uv from the official standalone path
`~/.local/bin/uv` or the architecture-native Homebrew prefix. It verifies
ownership and rejects group/world-writable executables instead of searching an
inherited `PATH`. It lets uv resolve, install when necessary, and run exact
Python `3.14.6` under `--no-project`, fixed owner cache/managed-Python roots,
and a fully rebuilt environment. The gate invokes `python` as uv's command
rather than handing the script to uv, so candidate inline script metadata is
not processed. Candidate project metadata and inherited `UV_*` settings cannot
select the interpreter or dependencies.

The validation stage builds both Darwin targets, natively smokes the current
architecture, builds the actual candidate foundation/runtime images, exercises
the generated Compose lifecycle and `--forget-tmux-state`, requires cleanup,
generates a CycloneDX SBOM, and applies the reviewed Grype policy. High or
Critical findings with a listed fixed version block the release; findings with
no listed fix remain explicit non-blocking warnings in the evidence. It also
verifies the checksummed comparison tag, commit, and changed-path list, then
runs affected addon build groups, the LaTeX watcher/preview lifecycle, and the
rootless Podman readiness probe when relevant inputs changed. The readiness
probe verifies the unprivileged binaries, subordinate ID ranges, namespace
helpers, storage/network helpers, and container configuration without granting
the outer development container privileges for nested user-namespace execution. No comparison tag selects
all three surfaces. Every command, selection reason, skip reason, and result is
retained beneath `evidence/` with toolchain metadata, image inspection, runtime
logs, hashes, and a release manifest.

The terminal interface streams subprocess output as it is produced. Each
high-level operation reports running, passed, or failed state with elapsed
time, and quiet commands emit a heartbeat every ten seconds. The same
transitions are retained in `evidence/steps.log`; full argv and output remain
in `commands.log` and `command-results.log`, so interactive progress does not
replace auditable evidence.
The terminal groups High/Critical package matches by unique advisory and prints
a bounded summary with severity, affected package names, and disposition. The
complete scanner report remains at `evidence/security/vulnerability-scan.json`;
counts, grouped advisories, package versions, fix versions, and the blocking or
warning classification are publication-required evidence in
`evidence/security/vulnerability-policy.json`.

The gate selects the first responsive runtime in this order: the `docker` CLI
contract exposed by Docker Desktop or OrbStack, then Podman. All image builds,
Compose lifecycles, exec probes, inspection, cleanup, scanning, and publication
use that selected runtime. Docker-compatible builds explicitly enable BuildKit
for features such as `COPY --chmod`; OrbStack does not need Docker Desktop's
separate Buildx component.
When Compose and Buildx are installed as Docker CLI plugins, the gate copies
only those owner/root-owned, non-group-writable executables into its empty
per-run `DOCKER_CONFIG`. It does not copy the owner's Docker configuration,
registry credentials, contexts, or unrelated plugins.

Before the Darwin build, the gate fetches the exact locked Cargo dependency
graph into a per-run credential-free Cargo home. The actual compilation remains
offline, so a newly locked crate does not require a pre-warmed owner cache and
candidate build scripts do not receive network access.

Only after every gate succeeds does the separate publisher receive normal host
GitHub/GHCR authority. It revalidates the immutable manifest and can upload
only the two Darwin archives plus checksums and push only the fixed aibox
foundation-version, runtime-version, and runtime-latest tags. It cannot build,
run tests, execute candidate code, commit, merge, or accept extra arguments.
Remote asset and image inspection is mandatory.

On failure, keep the run directory as diagnostic evidence. Correct the source,
create a new candidate commit/tag, and prepare a new run ID; do not edit the
old input or selectively reuse its evidence. The owner must review changes to
the gate and publisher before using a changed version.

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


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.3/docs/contributing/maintenance/index.md
