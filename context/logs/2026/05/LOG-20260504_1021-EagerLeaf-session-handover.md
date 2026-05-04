---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260504_1021-EagerLeaf-session-handover
  created: '2026-05-04T10:21:17+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-04T10:21:17+00:00'
  summary: 'Session handover: aibox v0.23.6 release completed, including host Phase
    2 macOS assets.'
  actor: TEAMMEMBER-cora
  subject: v0.23.6 release wrapup
  subject_kind: release
  details:
    release: v0.23.6
    status: complete
    repository: projectious-work/aibox
    github_release: https://github.com/projectious-work/aibox/releases/tag/v0.23.6
    main_head: 1c8d1de
    commits:
    - '5adc3e7 fix(v0.23.6): integrate processkit 0.25.5 and harden runtime diagnostics'
    - '1c8d1de chore: bump CLI version to 0.23.6'
    processkit_version: v0.25.5
    phase_1_linux_side: complete
    phase_2_host_side: complete, confirmed by owner and verified on GitHub release
      assets
    assets_verified:
    - aibox-v0.23.6-aarch64-apple-darwin.tar.gz
    - aibox-v0.23.6-x86_64-apple-darwin.tar.gz
    - aibox-v0.23.6-aarch64-unknown-linux-gnu.tar.gz
    - aibox-v0.23.6-x86_64-unknown-linux-gnu.tar.gz
    docs: deployed to gh-pages by release script
    git_status: clean at handover
    validation:
    - cargo fmt --check passed
    - cargo clippy -- -D warnings passed
    - 'full cargo test passed: 742 unit, 64 E2E, 26 integration'
    - cargo audit clean
    - release builds for both Linux targets passed
    - aibox --version verified as 0.23.6
    implemented_scope:
    - integrated processkit v0.25.5 and refreshed processkit mirror
    - fixed Codex MCP script paths for subagents
    - added addon dependency fallback migration guidance
    - added doctor aibox.toml schema and runtime theme/template checks
    - removed stale lazygit runtime files when lazygit is disabled
    - fixed native Zellij status plugin visibility
    - strengthened no-container and asciinema E2E coverage
    next_actions:
    - No release action remains for v0.23.6.
    - For the next session, start with pk-resume and verify whether a downstream project
      rebuild should consume v0.23.6.
---
