# Session handover

**Date:** 2026-07-31  
**Active line:** `v1.x-pre-release`  
**Recorded branch tip:** `5ae51ff5482af92a8c9c46d3a0a8bfa397941d4b`

## Completed

- Published and publicly verified `v1.0.0-alpha.1`.
- Completed the host-side release phase, including macOS artifacts, GHCR
  publication, generated-runtime smoke, and runtime finalization.
- Corrected prerelease SemVer handling across host release tooling.
- Corrected the v1 release smoke to use the explicit legacy attach lifecycle
  and to report early child-process exits accurately on macOS.
- Fixed release publication resumes by initializing the designated release
  branch before full or reduced-step publication.
- Reviewed and closed GitHub issues #179, #236, and #289.
- Merged the final generated `aibox.toml` catalog refresh through PR #300.
- Removed obsolete local branches, remote topic/recovery branches, and stale
  worktree metadata.

## Validation completed

- 1,162 unit tests passed.
- 90 Tier-1 E2E tests passed; one intentional test remained ignored.
- 45 integration tests passed.
- Exact processkit `v1.0.0-alpha.3` producer tests passed.
- Rust formatting, zero-warning Clippy, and RustSec audit passed.
- Release-harness regression tests passed.
- Hugo production build passed with 161 pages.
- The public release contains four native archives, checksums, and LICENSE.

## Repository state

- The primary checkout is clean.
- `/workspace` is the only registered worktree.
- Maintained branches are `main`, `gh-pages`, `v0.x-dev`, `v0.x-release`,
  `v1.x-dev`, and `v1.x-pre-release`.
- Maintained local branches were synchronized with their remote counterparts.

## Remaining work

GitHub issue [#299](https://github.com/projectious-work/aibox/issues/299)
tracks the non-fabricable stable-v1 evidence obligations:

1. Exercise release and exact-version rollback across Linux and macOS on
   x86_64 and aarch64, then record the evidence with
   `scripts/record-v1-platform-rehearsal.sh`.
2. Run and record the five reviewed external operator journeys: aibox
   self-host, representative v0 migration, clean Compose without processkit,
   an existing Kubernetes target, and direct processkit use.
3. Re-run `aibox config release-readiness --output json` against the final
   stable candidate and require every stable-v1 gate to pass.
4. Keep v0 retirement as a separate reviewed decision after the evidence is
   complete.

## Resume point

Start with issue #299. Do not add new v1 product scope until its platform and
operator evidence has been collected and the stable readiness result has been
reviewed.
