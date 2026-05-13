---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260513_0715-MerryOrchard-session-handover
  created: '2026-05-13T07:15:08+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-13T07:15:08+00:00'
  summary: 'Session handover: v0.25.13 completed end-to-end after fixing stale runtime
    projection and broad runtime-home mounts.'
  actor: codex
  subject: aibox v0.25.13 release and runtime projection fix
  subject_kind: release
  details:
    completed:
    - 'Diagnosed stale state root cause: managed .aibox-home files were preserved
      across apply/up and compose still had narrow Vim/Cargo mounts.'
    - Changed seed sync so aibox-managed runtime files are authoritative on apply
      while unknown runtime-home files remain untouched.
    - Changed aibox up for missing runtime to refresh managed runtime files before
      container creation.
    - Changed generated compose mount source of truth to broad .vim and .cargo directory
      mounts, alongside existing .config/.cache/.local/.tmux/provider directory mounts.
    - Updated .devcontainer/docker-compose.yml and container docs to reflect broad
      runtime-home mounts.
    - Added regression coverage for stale tmux/Yazi managed file refresh and broad
      Vim/Cargo mount generation.
    - 'Committed fix as 2174fdac fix: refresh managed runtime projection.'
    - 'Released v0.25.13 with full testing; release script committed f1184c65 chore:
      bump CLI version to 0.25.13.'
    - User completed host Phase 2; verified GitHub release has 8 assets, macOS and
      Linux archives plus checksums.
    - 'Verified post-host generated runtime commit 2fccaef5 chore: refresh generated
      runtime for v0.25.13 is on origin/main.'
    verification:
    - Working tree clean at 2fccaef577ee98f830b43f9beae57dcb25de6174.
    - origin/main matches HEAD.
    - v0.25.13 tag exists and release is not draft/prerelease.
    - 'GitHub release assets verified: aarch64/x86_64 Linux and Darwin tarballs plus
      sha256 files.'
    - Runtime smoke logs present under dist/release-smoke/v0.25.13/20260513-091105/.
    - 'Release gates passed: Phase 0 doctors, fmt, clippy -D warnings, full cargo
      test, Tier 2 SSH companion E2E, full visual E2E matrix, cargo audit, Linux release
      builds, version smoke, docs deploy.'
    - 'Host Phase 2 reported complete by user: macOS binaries uploaded, GHCR images
      pushed, runtime smoke passed, generated runtime refreshed.'
    open_items:
    - 'Pending migrations remain: MIG-DISABLED-HARNESS-STATE and MIG-RUNTIME-DRIFT-20260512T190200.
      The latter is now superseded in spirit by the v0.25.13 source fix but remains
      pending in processkit until explicitly resolved.'
    - Active interlocutor config points at missing TEAMMEMBER-20260508_2042-MigratedMember-avery;
      no TeamMember identity could be resolved.
    - Plain git fetch --tags still reports old divergent historical tags; fetch origin
      main --no-tags works and was used for current-state verification.
    next_actions:
    - Optionally resolve or reject the two pending migrations in a follow-up session.
    - 'After downstream host/runtime refresh, verify user-facing behavior with a fresh
      project using aibox v0.25.13: apply, delete runtime, up, then check first tmux
      status line and theme adherence.'
    - If package metadata verification is needed, verify GHCR tags from a host/token
      with suitable package-read permissions.
---
