---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260507_0722-ToughAsh-session-handover
  created: '2026-05-07T07:22:49+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-07T07:22:49+00:00'
  summary: Paused v0.24.0 release after aibox-e2e-testrunner overload during visual Yazi E2E; user needs to shut down OrbStack.
  actor: codex
  subject: aibox v0.24.0 release
  subject_kind: release
  details:
    state: blocked
    head_commit: '6f1d53e chore: bump CLI version to 0.24.0'
    pushed: false
    tag_created: false
    release_created: false
    important_commits:
    - 'b93582a fix: harden runtime tui handoffs'
    - '6f1d53e chore: bump CLI version to 0.24.0'
    local_uncommitted:
    - cli/tests/e2e/visual_matrix.rs marker-aware Yazi preview polling and bounded cleanup, not amended
    release_gate_status:
    - fmt passed
    - clippy passed
    - 'normal cargo test passed: 808 unit, 71 e2e, 26 integration'
    - 'Tier 2 companion E2E passed: 116 passed, 0 failed, 3 ignored'
    - 'full visual tier 1 status/theme passed: 12/12 recordings'
    - 'full visual tier 2 tab traversal passed: 3/3 recordings'
    - full visual tier 3 Yazi preview failed/hung during markdown preview rerun after stale session
    host_evidence:
      aibox_e2e_testrunner_cpu: 799.04%
      aibox_e2e_testrunner_memory: 3.347GiB / 9.76GiB
      aibox_e2e_testrunner_pids: 18582 docker stats, 18546 cgroup pids.current
      process_states: 14224 S, 4509 D, 111 Z, 45 R, plus smaller session groups
      oom: memory.events oom=0 oom_kill=0
      stale_session_observed: aibox-yazi-preview-markdown
      zellij_log_evidence: Client sent over 1000 consecutive unknown messages, probably an infinite loop
    analysis: The overload is in the shared E2E companion, not the production runtime container. The visual harness starts Zellij under asciinema, backgrounds a driver that calls zellij action dump-screen, then kills/tears down zellij. A failed run left a stale Zellij session and the companion accumulated massive processes including thousands in D state. E2E runner exec/scp calls do not bound remote command runtime after SSH connects, so cleanup/probes can hang once the companion is unhealthy.
    next_steps:
    - Shut down OrbStack/stop aibox-e2e-testrunner before continuing.
    - Do not rerun full visual E2E until harness is fixed.
    - Add bounded remote exec/scp timeouts in E2eRunner, companion PID guard before/after visual cases, and hard cleanup of zellij/yazi/asciinema sessions with timeout.
    - Split Yazi preview visual matrix away from release-critical status-row gate or run it in a disposable companion/container.
    - After restart, inspect/amend or revert the uncommitted visual_matrix.rs patch intentionally, then rerun targeted safe tests.
    - Resume release from local commit 6f1d53e only after companion stability is addressed.
---
