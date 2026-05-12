---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260507_1426-CalmFinch-session-handover
  created: '2026-05-07T14:26:02+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-07T14:26:02+00:00'
  summary: 'pk-wrapup: aibox v0.24.3 Linux-side release completed; host-side release remains; live container still shows hot Zellij server CPU.'
  actor: TEAMMEMBER-20260422_0832-MigratedMember-cora
  subject: v0.24.3 Linux-side release and runtime CPU follow-up
  subject_kind: release
  details:
    completed:
    - 'Implemented and pushed runtime containment: generated Zellij layouts default to shell status, sidecar remains opt-in, and aibox-status reads current-container metrics directly.'
    - 'Implemented and pushed Claude Code derived runtime drift fix: stale granular processkit MCP permissions are pruned when processkit-gateway is active, the stable .aibox-home/.local/bin/claude shim is seeded to /usr/local/bin/claude, and aibox doctor reports Claude drift.'
    - Recorded related processkit decisions/workitems and archived BACK-20260507_1341-SoundSky-claude-code-derived-runtime-drift as done.
    - Committed and pushed main through 9e5438a.
    - Ran ./scripts/maintain.sh release 0.24.3 with AIBOX_RELEASE_SKIP_COMPANION_E2E=1 and AIBOX_RELEASE_VISUAL_E2E=skip.
    - GitHub release v0.24.3 was created with Linux binaries for aarch64-unknown-linux-gnu and x86_64-unknown-linux-gnu.
    - Documentation was built and deployed to gh-pages.
    validation:
    - Release-state report completed; processkit current=v0.25.8 latest=v0.25.8.
    - cargo fmt check passed.
    - cargo clippy --all-targets -- -D warnings passed.
    - 'cargo test passed: 822 unit tests, 71 Tier 1 E2E tests, 28 integration tests.'
    - cargo audit clean.
    - Release builds completed for both Linux targets; aibox --version matched 0.24.3.
    - Verified gh release view v0.24.3 shows uploaded Linux assets and non-draft/non-prerelease release.
    - Verified git status is clean and main/origin/main/tag v0.24.3 all point to 9e5438a.
    skipped:
    - Tier 2 SSH companion E2E skipped per owner request using AIBOX_RELEASE_SKIP_COMPANION_E2E=1.
    - 'Opt-in visual E2E skipped for speed using AIBOX_RELEASE_VISUAL_E2E=skip; justification: owner explicitly requested quick release and no companion E2E while release contained a containment patch already covered by unit/Tier 1 checks.'
    remaining:
    - 'Owner/host Phase 2: run ./scripts/maintain.sh release-host 0.24.3 on the macOS host.'
    - Host Phase 2 will build macOS binaries, upload them to the existing GitHub release, build/push GHCR images, refresh repo-owned generated runtime surfaces after image tags exist, and commit/push generated runtime drift if any.
    runtime_cpu_evidence:
      loadavg: 9.88 8.34 6.69
      pids_current: '118'
      memory_events: low 0; high 0; max 0; oom 0; oom_kill 0; oom_group_kill 0; sock_throttled 0
      cpu_stat: nr_throttled 0; throttled_usec 0
      top_process: PID 51 /usr/local/bin/zellij --server /tmp/zellij-1000/contract_version_1/aibox around 278% CPU
      note: Did not kill PID 51 because the active Codex session is running under the Zellij process tree. The release contains downstream containment; this existing live session likely needs a Zellij/runtime restart to clear the current spin.
    git:
      head: 9e5438a
      tag: v0.24.3
      recent_commits:
      - '9e5438a chore: add 0.24.3 compatibility entry'
      - '2b18b25 chore: bump CLI version to 0.24.3'
      - 'ad770ca fix(v0.24.3): contain Zellij status and Claude runtime drift'
      status: clean on main...origin/main
    links:
      release: https://github.com/projectious-work/aibox/releases/tag/v0.24.3
      docs: https://projectious-work.github.io/aibox/
      host_prompt: dist/RELEASE-PROMPT.md
---
