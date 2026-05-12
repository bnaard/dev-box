---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260506_2032-NeatCrow-session-handover
  created: '2026-05-06T20:32:47+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-06T20:32:47+00:00'
  summary: Completed aibox v0.23.21 patch release wrapup after host Phase 2 confirmation.
  actor: TEAMMEMBER-20260422_0832-MigratedMember-cora
  subject: v0.23.21
  subject_kind: release
  details:
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.23.21
    repo_state:
      branch: main
      head: 'e7450f4 chore: refresh generated runtime for v0.23.21'
      origin_main: e7450f44ce4a9428fa25bbd1dc0c6f5bc93a419c
      tag: v0.23.21
      tag_commit: 'cbf7bcc fix: expose yazi companion entrypoint in final images'
      working_tree: clean
    release_assets:
    - aibox-v0.23.21-aarch64-apple-darwin.tar.gz
    - aibox-v0.23.21-x86_64-apple-darwin.tar.gz
    - aibox-v0.23.21-aarch64-unknown-linux-gnu.tar.gz
    - aibox-v0.23.21-x86_64-unknown-linux-gnu.tar.gz
    linux_phase_validation:
    - fmt
    - clippy
    - full Rust tests
    - 69 no-container E2E
    - 26 integration tests
    - Tier 2 SSH companion E2E 114 passed
    - visual status/theme matrix 12/12 passed
    - cargo audit
    - Linux release builds and version smoke
    host_phase_2: User confirmed host phase 2 done; release assets verified include macOS binaries and main includes generated runtime refresh commit.
    ghcr_verification: Container package verification from this environment failed with HTTP 403 because the gh token lacks read:packages scope.
    notable_fixes:
    - Zellij native status permission-cache projection guard and doctor/E2E coverage
    - Codex-specific DNS sandbox release note
    - Yazi ya companion entrypoint copied into final images
---
