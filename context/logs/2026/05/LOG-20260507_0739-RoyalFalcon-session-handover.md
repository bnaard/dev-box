---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260507_0739-RoyalFalcon-session-handover
  created: '2026-05-07T07:39:22+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-07T07:39:22+00:00'
  summary: 'Updated handover before OrbStack restart: production containers can show
    the same process-explosion class as E2E; add automatic runtime logging/post-mortem
    capture to aibox/processkit.'
  actor: codex
  subject: aibox runtime process explosion triage
  subject_kind: incident
  details:
    reason_for_update: 'User corrected earlier framing: the OrbStack/e2e overload
      should be treated as a reproducible symptom of a production-relevant runtime
      failure class, not as test-only.'
    latest_host_evidence:
      container: aibox
      timestamp_utc: '2026-05-07 07:29:52'
      docker_stats: CPU 3.98%, memory 178.9MiB/9.76GiB, PIDS 94
      cgroup_pids_current: '96'
      memory_events: oom 0, oom_kill 0
      process_state_by_command:
      - 3 R sh
      - 2 S zellij
      - 2 S python
      - 2 R sort
      - 1 S yazi
      - 1 S vim-loop
      - 1 S vim
      - 1 S uv
      - 1 S sleep
      - 1 S sh
      - 1 S node
      - 1 S docker-init
      - 1 S codex
      - 1 S bash
      zellij_log_signal: Cannot read SHELL env, falling back to use /bin/sh, repeated
        about once per minute
    current_assessment:
      production_baseline: The sampled production aibox container was healthy at about
        96 PIDs, so runaway is triggered rather than constant.
      not_version_bump: The 0.24.0 bump is not the initial trigger; reported production
        failures were on 0.23.21.
      suspect_window:
      - v0.23.2 made shell aibox-status substantially heavier
      - v0.23.3 introduced native plugin invoking aibox-status --plugin-json
      - v0.23.4 made native status mode default
      - v0.23.9 revised status behavior again
      leading_hypothesis: Zellij native status plugin periodically invokes the Bash
        aibox-status collector; the collector scans /proc with subprocess-heavy helpers.
        If collection stalls or overlaps under startup/permission/Zellij fallback
        conditions, it can amplify PID pressure and produce the observed high-PID
        state.
      related_production_bug: SHELL is still missing in the live Zellij parent environment,
        causing repeated /bin/sh fallback logs; keep the unreleased SHELL=/bin/bash
        runtime fix.
    release_state:
      release_not_complete: true
      local_head: '6f1d53e chore: bump CLI version to 0.24.0'
      origin_main: 'b93582a fix: harden runtime tui handoffs'
      git_state: main ahead 1; cli/tests/e2e/visual_matrix.rs modified; this handover
        log untracked before commit
      no_tag_or_release: v0.24.0 was not tagged or published
    must_do_next:
      immediate: User will restart OrbStack; do not run heavy E2E/Docker tests until
        resumed.
      implementation_priorities:
      - Replace or heavily bound the Bash aibox-status collector path, especially
        /proc scanning; avoid per-PID cat/tr/awk subprocess fanout.
      - Add an in-flight request guard/backoff in the Zellij status plugin so timer
        events cannot overlap status collection.
      - Add fast PID-count bailout and degraded-mode rendering when pids.current/process
        count is above threshold.
      - Preserve SHELL=/bin/bash and cache ownership fixes from current unreleased
        work.
      - Rework the visual/E2E harness only after runtime collector safety is fixed;
        the harness can trigger the same class but is not the root production-only
        boundary.
      automatic_logging_requirement: 'Implement built-in aibox/processkit runtime
        observability so post-mortem evidence is captured without manual host intervention.
        Candidate designs: lightweight in-container watchdog process, sidecar container,
        or integrated aibox runtime supervisor that periodically records pids.current,
        memory.events, cpu.stat, process state counts by /proc/*/stat+comm, zellij
        log tail, status-plugin errors, and container/version metadata into a bounded
        ring buffer under .aibox/ or a mounted diagnostics directory. It must avoid
        /proc/*/cmdline in high-PID states and must keep overhead bounded.'
    safe_host_probe_for_future_bad_state: 'Use stat+comm only, not /proc/*/cmdline:
      docker stats; cgroup pids.current and memory.events; for /proc/[0-9] read stat
      state plus comm; tail zellij logs.'
---
