---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260507_1344-CoolLynx-session-handover
  created: '2026-05-07T13:44:42+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-07T13:44:42+00:00'
  summary: 'Session handover: Zellij sidecar status contained, Claude diagnostics and tmux evaluation backlogged, patch candidate uncommitted'
  actor: codex
  subject: aibox zellij containment and derived-runtime diagnostics
  subject_kind: session
  details:
    completed:
    - 'Diagnosed derived-project runtime evidence showing Zellij server as CPU owner: /usr/local/bin/zellij --server at roughly 300-385% CPU in user-provided ps/docker stats; local /proc still showed PID 50 zellij server with 31 tasks and loadavg around 5.6/6.2/6.0.'
    - Patched default Zellij status mode from sidecar to shell, leaving the WASM sidecar status/keybar opt-in only.
    - Patched aibox-status to read bounded metrics directly from the current container for --plugin-json, default output, and --watch, removing the diagnostics-sidecar snapshot dependency that caused false 64MiB/PROC 1 stats.
    - Updated CLI help, seed tests, and docs for shell default and experimental sidecar mode.
    - Recorded accepted decisions for defaulting away from sidecar and for prioritizing containment, Claude diagnostics, and tmux evaluation.
    - Created backlog workitems for Claude Code derived-runtime drift, deferred powerline status/tabbar redesign, and tmux evaluation.
    validation:
    - cargo fmt passed.
    - rustc images/base-debian/config/bin/aibox-status.rs -o /tmp/aibox-status-check passed.
    - /tmp/aibox-status-check --plugin-json produced main-container style metrics instead of sidecar cgroup metrics.
    - 'cargo test passed: 819 unit tests and 71 integration/E2E tests.'
    - cargo clippy --all-targets -- -D warnings passed.
    uncommitted_modified:
    - cli/src/cli.rs
    - cli/src/config.rs
    - cli/src/seed.rs
    - docs-site/docs/customization/layouts.md
    - docs-site/docs/reference/configuration.md
    - images/base-debian/config/bin/aibox-status.rs
    untracked_context:
    - context/decisions/DEC-20260507_1336-WarmEmber-default-zellij-status-to-shell-until.md
    - context/decisions/DEC-20260507_1342-HappyPeak-prioritize-zellij-containment-claude-diagnostics-and.md
    - context/logs/2026/05/LOG-20260507_1336-KindCrane-decision-created.md
    - context/logs/2026/05/LOG-20260507_1341-GentleFox-workitem-created.md
    - context/logs/2026/05/LOG-20260507_1341-SnowyQuail-workitem-created.md
    - context/logs/2026/05/LOG-20260507_1341-SureFern-workitem-created.md
    - context/logs/2026/05/LOG-20260507_1342-LoyalWren-decision-created.md
    - context/workitems/2026/05/BACK-20260507_1341-CalmEagle-evaluate-tmux-runtime-fallback.md
    - context/workitems/2026/05/BACK-20260507_1341-SharpCrow-powerline-status-tabbar-redesign.md
    - context/workitems/2026/05/BACK-20260507_1341-SoundSky-claude-code-derived-runtime-drift.md
    next_steps:
    - 'Host should stop runaway derived-project Zellij server if still hot: docker exec aibox sh -lc ''zellij kill-session aibox 2>/dev/null || true; kill -TERM 50 2>/dev/null || true; sleep 2; kill -KILL 50 2>/dev/null || true'' or docker stop aibox if CPU does not drop.'
    - Commit current containment patch and processkit records, then prepare patch release so derived projects stop defaulting to the WASM sidecar path.
    - 'Implement Claude Code/runtime diagnostics workitem: inspect/generate checks for stale aibox.lock CLI version, .mcp.json vs .claude/settings.json mismatch, Claude binary path conflicts, missing Zellij permission cache, and sidecar-vs-main status mismatch.'
    - Evaluate tmux as an alternate or default runtime multiplexer before investing further in Zellij plugin UI work.
    related_entities:
    - DEC-20260507_1336-WarmEmber-default-zellij-status-to-shell-until
    - DEC-20260507_1342-HappyPeak-prioritize-zellij-containment-claude-diagnostics-and
    - BACK-20260507_1341-SoundSky-claude-code-derived-runtime-drift
    - BACK-20260507_1341-SharpCrow-powerline-status-tabbar-redesign
    - BACK-20260507_1341-CalmEagle-evaluate-tmux-runtime-fallback
    - BACK-20260505_2222-KeenHare-investigate-zellij-status-plugin-errors
---
