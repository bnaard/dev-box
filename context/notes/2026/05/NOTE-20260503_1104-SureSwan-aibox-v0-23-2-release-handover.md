---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260503_1104-SureSwan-aibox-v0-23-2-release-handover
  created: '2026-05-03T11:04:41+00:00'
spec:
  title: Session handover after aibox v0.23.2 release
  body: |
    # Session Handover: aibox v0.23.2 Release

    ## Current State

    - Repository: `/workspace`, branch `main`.
    - Git state: clean; `main` is synced with `origin/main` at `8df8eae`.
    - Latest local commits:
      - `8df8eae fix: satisfy runtime cleanup clippy`
      - `1f4df91 chore: bump CLI version to 0.23.2`
      - `94a7061 chore: integrate processkit v0.25.1`
      - `908cf2f fix: improve runtime cleanup and zellij status controls`
    - GitHub release: `v0.23.2` at https://github.com/projectious-work/aibox/releases/tag/v0.23.2.
    - Release assets currently attached:
      - `aibox-v0.23.2-aarch64-apple-darwin.tar.gz`
      - `aibox-v0.23.2-aarch64-unknown-linux-gnu.tar.gz`
      - `aibox-v0.23.2-x86_64-apple-darwin.tar.gz`
      - `aibox-v0.23.2-x86_64-unknown-linux-gnu.tar.gz`
    - Docs were deployed to https://projectious-work.github.io/aibox/.
    - `dist/RELEASE-PROMPT.md` still contains the generated host-side command, but GitHub release assets show the macOS tarballs are already present.

    ## Completed This Session

    - Integrated processkit `v0.25.1`:
      - `cli/src/processkit_vocab.rs` default version now points at `v0.25.1`.
      - `aibox.toml` processkit pin now points at `v0.25.1`.
      - Compatibility docs and compatibility table were updated for `aibox 0.23.2` / `processkit v0.25.1`.
      - processkit `FORMAT.md` was unchanged, so no vocabulary surface update was required beyond the default version pin.
    - Applied generated migrations:
      - `MIG-20260503T104949` processkit `v0.25.0 -> v0.25.1`, applied.
      - `MIG-RUNTIME-20260503T104948` runtime home `0.23.0 -> 0.23.1`, applied.
      - The processkit conflict was only the scaffolded `AGENTS.md` version/footer delta; local project instructions were preserved.
    - Restored `gh` as a standard devcontainer/release tool:
      - The generated devcontainer includes `gh` through the tooling surface.
      - The repo-local `.devcontainer/Dockerfile.local` also includes `gh`.
    - Released `aibox v0.23.2` through `./scripts/maintain.sh release 0.23.2`:
      - fmt passed.
      - clippy passed after one follow-up style fix in `cli/src/container.rs`.
      - full Rust tests passed: unit, E2E, and integration suites.
      - cargo audit passed.
      - Linux release builds passed.
      - `aibox --version` reported `0.23.2`.
      - tag `v0.23.2` was pushed.
      - GitHub release was created.
      - docs were deployed.
      - `main` was pushed after the release script, because the script pushes the tag and docs branch but not `main`.

    ## Important Notes

    - The first release attempt stopped before tagging because clippy flagged a collapsible nested `if` in `cli/src/container.rs` around stale runtime cleanup. The fix is committed as `8df8eae`.
    - The release script wrote `dist/RELEASE-PROMPT.md` with `./scripts/maintain.sh release-host 0.23.2`. However, the GitHub release currently shows both macOS assets attached. Before asking the owner to run the host step again, verify whether GHCR images were also pushed.
    - `dist/` is ignored and contains release artifacts/notes generated during the release.

    ## Active Follow-Ups

    - `BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin`: build native Zellij plugin for aibox runtime status. Scope was expanded and approved to target two separately show/hidable rows: one key-hint bar and one runtime status bar.
    - `BACK-20260503_0148-CalmDew-release-script-notes-push-order`: improve aibox release script notes and push ordering. This remains relevant because the script currently creates the release before host-side completion and does not push `main` automatically after the version bump.
    - `BACK-20260429_1608-RoyalHawk-reduce-codex-startup-latency`: remains open for reducing Codex startup latency from eager processkit MCP server initialization.

    ## Recommended Next Checks

    1. Confirm whether the host-side `release-host 0.23.2` phase already completed fully, especially GHCR image push state.
    2. If host-side phase is fully done, optionally update release notes or close out any release checklist outside processkit.
    3. Pick up the native Zellij plugin/keybar backlog item when ready; target remains a two-line plugin surface with separate show/hide controls for key hints and status.
  type: reference
  state: captured
  review_due: '2026-05-10'
  tags:
  - session-handover
  - release
  - aibox-v0.23.2
  - processkit-v0.25.1
  source: pk-wrapup
---

# Session Handover: aibox v0.23.2 Release

## Current State

- Repository: `/workspace`, branch `main`.
- Git state: clean; `main` is synced with `origin/main` at `8df8eae`.
- Latest local commits:
  - `8df8eae fix: satisfy runtime cleanup clippy`
  - `1f4df91 chore: bump CLI version to 0.23.2`
  - `94a7061 chore: integrate processkit v0.25.1`
  - `908cf2f fix: improve runtime cleanup and zellij status controls`
- GitHub release: `v0.23.2` at https://github.com/projectious-work/aibox/releases/tag/v0.23.2.
- Release assets currently attached:
  - `aibox-v0.23.2-aarch64-apple-darwin.tar.gz`
  - `aibox-v0.23.2-aarch64-unknown-linux-gnu.tar.gz`
  - `aibox-v0.23.2-x86_64-apple-darwin.tar.gz`
  - `aibox-v0.23.2-x86_64-unknown-linux-gnu.tar.gz`
- Docs were deployed to https://projectious-work.github.io/aibox/.
- `dist/RELEASE-PROMPT.md` still contains the generated host-side command, but GitHub release assets show the macOS tarballs are already present.

## Completed This Session

- Integrated processkit `v0.25.1`:
  - `cli/src/processkit_vocab.rs` default version now points at `v0.25.1`.
  - `aibox.toml` processkit pin now points at `v0.25.1`.
  - Compatibility docs and compatibility table were updated for `aibox 0.23.2` / `processkit v0.25.1`.
  - processkit `FORMAT.md` was unchanged, so no vocabulary surface update was required beyond the default version pin.
- Applied generated migrations:
  - `MIG-20260503T104949` processkit `v0.25.0 -> v0.25.1`, applied.
  - `MIG-RUNTIME-20260503T104948` runtime home `0.23.0 -> 0.23.1`, applied.
  - The processkit conflict was only the scaffolded `AGENTS.md` version/footer delta; local project instructions were preserved.
- Restored `gh` as a standard devcontainer/release tool:
  - The generated devcontainer includes `gh` through the tooling surface.
  - The repo-local `.devcontainer/Dockerfile.local` also includes `gh`.
- Released `aibox v0.23.2` through `./scripts/maintain.sh release 0.23.2`:
  - fmt passed.
  - clippy passed after one follow-up style fix in `cli/src/container.rs`.
  - full Rust tests passed: unit, E2E, and integration suites.
  - cargo audit passed.
  - Linux release builds passed.
  - `aibox --version` reported `0.23.2`.
  - tag `v0.23.2` was pushed.
  - GitHub release was created.
  - docs were deployed.
  - `main` was pushed after the release script, because the script pushes the tag and docs branch but not `main`.

## Important Notes

- The first release attempt stopped before tagging because clippy flagged a collapsible nested `if` in `cli/src/container.rs` around stale runtime cleanup. The fix is committed as `8df8eae`.
- The release script wrote `dist/RELEASE-PROMPT.md` with `./scripts/maintain.sh release-host 0.23.2`. However, the GitHub release currently shows both macOS assets attached. Before asking the owner to run the host step again, verify whether GHCR images were also pushed.
- `dist/` is ignored and contains release artifacts/notes generated during the release.

## Active Follow-Ups

- `BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin`: build native Zellij plugin for aibox runtime status. Scope was expanded and approved to target two separately show/hidable rows: one key-hint bar and one runtime status bar.
- `BACK-20260503_0148-CalmDew-release-script-notes-push-order`: improve aibox release script notes and push ordering. This remains relevant because the script currently creates the release before host-side completion and does not push `main` automatically after the version bump.
- `BACK-20260429_1608-RoyalHawk-reduce-codex-startup-latency`: remains open for reducing Codex startup latency from eager processkit MCP server initialization.

## Recommended Next Checks

1. Confirm whether the host-side `release-host 0.23.2` phase already completed fully, especially GHCR image push state.
2. If host-side phase is fully done, optionally update release notes or close out any release checklist outside processkit.
3. Pick up the native Zellij plugin/keybar backlog item when ready; target remains a two-line plugin surface with separate show/hide controls for key hints and status.
