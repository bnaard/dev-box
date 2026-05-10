---
id: NOTE-20260411_0000-LoyalSpruce-aibox-release-process
title: "aibox Release Process"
type: reference
status: permanent
created: 2026-04-11T00:00:00Z
tags: [release, process, operations]
skill: release-semver
---

# aibox Release Process

When asked to release version X.Y.Z, follow ALL steps in order.
Full canonical source: `context/work-instructions/RELEASE-PROCESS.md` (archived).

## Phase 0 — Dependency and harness state check (Claude does this FIRST)

Before every release, check ALL upstream dependencies for updates and review
whether upstream changes alter generated runtime behavior. The scripted release
path runs all three Phase 0 steps automatically before the version bump:

```bash
./scripts/maintain.sh release-check-state   # 1. deps + harness drift → dist/RELEASE-STATE.md
./scripts/maintain.sh release-doctors       # 2. pk-doctor + aibox doctor → dist/RELEASE-DOCTORS.md
```

Or run `./scripts/maintain.sh release <version>` which calls both in sequence.

### Step 1 — release-check-state

`./scripts/maintain.sh release-check-state` writes `dist/RELEASE-STATE.md`, covering:

- processkit default version drift
- pinned base-image tool versions
- floating image inputs (`uv:latest`, Node.js major streams, Debian base tags)
- AI harness installer/package surfaces
- Rust advisory status and lockfile-resolvable crate updates
- addon inventory at release time

The Rust dependency section is mandatory. It must include both `cargo audit`
and `cargo update --dry-run`. `cargo audit` is the security gate; `cargo update
--dry-run` is the freshness gate that shows lockfile-resolvable crate updates
without mutating `Cargo.lock`.

The report is evidence, not a substitute for judgement. For every reported
update, inspect upstream release notes and decide whether to bump immediately,
defer explicitly, or file a follow-up issue. For every AI harness, verify:
install location, binary path, config path, command/skill projection path, auth
persistence, and generated devcontainer expectations.

If any release-check finding is deferred, create a processkit WorkItem in the
same turn before continuing the release. The WorkItem must name the deferred
dependency or harness surface, record the release where it was deferred, and
state what validation is required before it can be shipped later. Mention the
WorkItem ID in release notes or the release handover so the deferral is
traceable.

For `cargo update --dry-run` specifically: if it reports available crate
updates, either apply them immediately with a real `cargo update` plus the full
release validation suite, or create a processkit WorkItem that captures the
crate update set and the validation required. Do not silently treat a clean
`cargo audit` result as evidence that Rust dependencies are current.

### Step 2 — release-doctors (pk-doctor + aibox doctor)

```bash
./scripts/maintain.sh release-doctors
```

Invokes both health checks and writes `dist/RELEASE-DOCTORS.md`:

- **`pk-doctor`** — processkit health aggregator: schema_filename validation,
  sharding, pending migrations, src/context drift, commands_consistency, and
  additional checks (mcp_config_drift, preauth_applied, …). Exits 1 on any
  ERROR; exits 0 if only WARNs or INFO.
- **`aibox doctor`** — aibox runtime hygiene: config validation, template
  membership, container runtime, devcontainer files, legacy artifact scan, and
  MCP gateway checks. Reports a summary line `Diagnostics complete: N warning(s),
  M error(s)` on stderr; M > 0 is treated as ERROR.

**Gate semantics:**

| Doctor outcome | Effect |
|----------------|--------|
| Both exit clean (0 ERRORs) | Continue; `dist/RELEASE-DOCTORS.md` written for the record |
| Either reports ERRORs | Release halted with message: "Release blocked: doctor checks failed. See dist/RELEASE-DOCTORS.md for details." |
| WARNs only (no ERRORs) | `dist/RELEASE-DOCTORS.md` written; release continues |

When running `./scripts/maintain.sh release <version>` interactively, the
agent is prompted to review `dist/RELEASE-DOCTORS.md` before proceeding
even when there are no blocking ERRORs, so WARN-level findings are visible
to the release manager before any mutation (version bump, tags, push).

### processkit

```bash
./scripts/maintain.sh sync-processkit
```

Queries GitHub for the latest processkit tag, patches `PROCESSKIT_DEFAULT_VERSION` in
`cli/src/processkit_vocab.rs` if newer, shows FORMAT.md diff so you can spot vocabulary
changes. If `processkit_vocab.rs` was patched: review diff, make CLI changes, run
`cargo test`, then commit everything before continuing.

### Pinned tool versions (in `images/base-debian/Dockerfile` and `.devcontainer/Dockerfile`)

| Tool | Pin location | How to check |
|------|-------------|-------------|
| tmux | apt Debian package (Trixie) | `Debian security tracker` (no ARG pin; managed through base image rebuilds) |
| Yazi | `ARG YAZI_VERSION` | `gh api repos/sxyazi/yazi/releases/latest --jq .tag_name` |
| ripgrep | `ARG RIPGREP_VERSION` | `gh api repos/BurntSushi/ripgrep/releases/latest --jq .tag_name` |
| fd | `ARG FD_VERSION` | `gh api repos/sharkdp/fd/releases/latest --jq .tag_name` |
| bat | `ARG BAT_VERSION` | `gh api repos/sharkdp/bat/releases/latest --jq .tag_name` |
| eza | `ARG EZA_VERSION` | `gh api repos/eza-community/eza/releases/latest --jq .tag_name` |
| fzf | `ARG FZF_VERSION` | `gh api repos/junegunn/fzf/releases/latest --jq .tag_name` |
| delta | `ARG DELTA_VERSION` | `gh api repos/dandavison/delta/releases/latest --jq .tag_name` |
| ouch | `ARG OUCH_VERSION` | `gh api repos/ouch-org/ouch/releases/latest --jq .tag_name` |
| starship | `ARG STARSHIP_VERSION` | `gh api repos/starship/starship/releases/latest --jq .tag_name` |
| zoxide | `ARG ZOXIDE_VERSION` | `gh api repos/ajeetdsouza/zoxide/releases/latest --jq .tag_name` |
| python3 | apt `python3` (Debian Trixie, ~3.13.x) | Check Trixie default |
| uv | `COPY --from=ghcr.io/astral-sh/uv:latest` (unpinned) | `gh api repos/astral-sh/uv/releases/latest --jq .tag_name` |
| Node.js | `COPY --from=node:22-slim` in .devcontainer | Check LTS status |

If a pinned version has an update, propose a bump. Report all findings before proceeding.
If a harness changed its install layout, command location, skill location, auth
state, or config files, treat that as an aibox or processkit compatibility bug
and patch/file it before publishing.

## Phase 1 — In container (Claude does this)

1. **Version bump**: `cli/Cargo.toml`
2. **Update documentation** for new features
3. **Run tests and clippy**:
   ```bash
   cd cli && cargo test && cargo clippy -- -D warnings
   ```
   The scripted release path also runs the Tier 2 SSH companion tests:
   ```bash
   ./scripts/maintain.sh test-e2e
   ```

   **Codex harness note:** Codex shell-tool calls can run inside an additional
   network sandbox that is separate from the devcontainer. That sandbox can
   block Docker/Compose service-name DNS such as `aibox-e2e-testrunner`, causing
   Tier 2 companion tests to fail with `Temporary failure in name resolution`
   even when the companion container is healthy. This is Codex-specific; do not
   generalize it to Claude or other harnesses. When running the release from
   Codex, execute the release command with elevated shell/network permissions
   so Docker's embedded DNS at `127.0.0.11` can resolve the companion hostname.
   If the failure appears, verify with:
   ```bash
   ssh -i /workspace/.aibox-e2e-runner-home/.ssh/id_ed25519 \
     -o BatchMode=yes -o StrictHostKeyChecking=no \
     -o UserKnownHostsFile=/dev/null \
     testuser@aibox-e2e-testrunner 'echo ok'
   ```
   A successful elevated check means the fix is to rerun the same release
   command with elevated permissions, not to change the release script, compose
   file, or E2E runner defaults.

   Visual E2E is release-gated because it records real tmux/Yazi/asciinema
   sessions and is intentionally slower. Before every release, the agent must
   decide and record in the release notes or handover which visual tier was run
   or why it was skipped:

   - Run `./scripts/maintain.sh test-e2e-visual-status` when layouts, themes,
     the tmux PowerKit status line, key bindings, generated `.aibox-home`,
     tmux version, or terminal color behavior changed.
   - Run `./scripts/maintain.sh test-e2e-visual-tabs` when tool panes, tab
     wiring, harness selection, Vim, shell, lazygit, or generated layout command
     wiring changed.
   - Run `./scripts/maintain.sh test-e2e-visual-yazi` when Yazi config,
     previews, git symbols, preview addons, or optional preview dependencies
     changed.
   - Run `./scripts/maintain.sh test-e2e-visual` for a full visual sweep when a
     release touches broad runtime surfaces or when confidence is otherwise low.
   - Run the full visual sweep at least every fifth release even if the current
     diff looks unrelated, so slow drift across themes, layouts, and tools is
     still caught periodically.

   `./scripts/maintain.sh release <version>` accepts
   `AIBOX_RELEASE_VISUAL_E2E=status|tabs|yazi|full|docs` to run a visual tier
   inside the release command. The default is `skip`, which prints a warning;
   only use the default when the release notes or handover explicitly justify
   the skip.

   For documentation assets, run:
   ```bash
   ./scripts/maintain.sh test-e2e-doc-captures
   ```
   This writes current-release asciinema casts, screen dumps, tmux capture-pane
   logs, and metadata under `docs-site/static/img/e2e/` by default. These
   artifacts are the source material for documentation screenshots or
   screencasts, so docs visuals stay tied to the same generated runtime that
   release validation used.
4. **Audit dependencies**:
   ```bash
   cd cli && cargo audit
   ```
   `./scripts/maintain.sh release` installs `cargo-audit` if missing, runs it,
   and aborts the release on any advisory. Phase 0 also records audit output in
   `dist/RELEASE-STATE.md` when `cargo-audit` is already available.
5. **Review crate freshness**:
   ```bash
   cd cli && cargo update --dry-run
   ```
   This is run by `./scripts/maintain.sh release-check-state`, which is run
   automatically by `./scripts/maintain.sh release` before the version bump. If
   updates are available, either apply them in this release and rerun validation,
   or create a processkit WorkItem for the deferred crate update pass.
6. **Build Linux release binaries — both architectures**:
   ```bash
   cd /workspace/cli
   # Native aarch64 build
   cargo build --release
   # Cross-compile for x86_64
   CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
     cargo build --release --target x86_64-unknown-linux-gnu
   ```
7. **Package Linux binaries**:
   ```bash
   cd /workspace && mkdir -p dist && VERSION=X.Y.Z
   cp cli/target/release/aibox dist/aibox-v${VERSION}-aarch64-unknown-linux-gnu
   tar -czf dist/aibox-v${VERSION}-aarch64-unknown-linux-gnu.tar.gz \
     -C dist aibox-v${VERSION}-aarch64-unknown-linux-gnu
   rm dist/aibox-v${VERSION}-aarch64-unknown-linux-gnu
   cp cli/target/x86_64-unknown-linux-gnu/release/aibox dist/aibox-v${VERSION}-x86_64-unknown-linux-gnu
   tar -czf dist/aibox-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz \
     -C dist aibox-v${VERSION}-x86_64-unknown-linux-gnu
   rm dist/aibox-v${VERSION}-x86_64-unknown-linux-gnu
   ls -lh dist/aibox-v${VERSION}-*-linux-*.tar.gz   # verify both tarballs exist
   ```
8. **Write release notes** to `dist/RELEASE-NOTES.md`
9. **Commit, tag, push**:
   ```bash
   git add cli/Cargo.toml cli/Cargo.lock
   git commit -m "chore: bump version to vX.Y.Z"
   git tag vX.Y.Z
   git push origin main && git push origin vX.Y.Z
   ```
10. **Create GitHub release with Linux binaries attached**:
   ```bash
   gh release create vX.Y.Z --repo projectious-work/aibox \
     --title "aibox vX.Y.Z" --notes-file dist/RELEASE-NOTES.md \
     dist/aibox-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz \
     dist/aibox-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
   ```
   Always use `--notes-file`, never `--generate-notes`. macOS binaries added in Phase 2.
11. **Deploy documentation**:
    ```bash
    ./scripts/maintain.sh docs-deploy
    ```

## Phase 2 — On macOS host (user runs one command)

```bash
cd /path/to/aibox
./scripts/maintain.sh release-host X.Y.Z
```

Builds macOS binaries (arm64 + x86_64), uploads to the existing GitHub release, builds
container images, pushes to GHCR.

**Prerequisites:** Rust toolchain on macOS, `gh` authenticated with `write:packages` scope,
Docker/OrbStack running.

**Critical gotcha:** A fresh `gh auth login` only grants the default `repo` scope — GHCR
push fails with `denied: permission_denied`. Fix:
```bash
gh auth refresh -s read:packages,write:packages,delete:packages
```
The script is idempotent for the binary upload step — safe to retry after a partial run.

## Commit message convention for releases

- **Version bump only:** `chore: bump version to vX.Y.Z`
- **With real changes:** `fix(vX.Y.Z): <one-line summary>` + section-by-section body with
  `Refs: DEC-NNN, BACK-NNN` and `Co-Authored-By:` trailer.
