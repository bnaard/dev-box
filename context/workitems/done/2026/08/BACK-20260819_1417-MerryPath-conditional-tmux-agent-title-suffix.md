---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260819_1417-MerryPath-conditional-tmux-agent-title-suffix
  created: '2026-08-19T14:17:47+00:00'
  updated: '2026-08-19T16:48:42+00:00'
spec:
  title: Add conditional tmux agent title suffix
  state: done
  type: story
  priority: high
  assignee: TEAMMEMBER-avery
  description: Add a safe {agent_suffix} title placeholder that renders a model and
    harness suffix only for agent windows, avoids dangling punctuation on non-agent
    windows, covers configuration validation/generation/tests/docs, reconciles the
    live runtime, and continues manual option acceptance testing.
  started_at: '2026-08-19T14:18:00+00:00'
  completed_at: '2026-08-19T16:48:42+00:00'
---

## Transition note (2026-08-19T14:18:00+00:00)

Implementation started immediately after owner approval during live title configuration acceptance testing.


## Transition note (2026-08-19T16:48:34+00:00)

Implementation and manual acceptance complete. Fresh generated aibox.toml now defaults to {state_symbol}{repository}{agent_suffix}; Codex, Claude, shell, and lazygit titles were confirmed; focused attention tests, full Rust suite with serial reruns for two parallel visual timeouts, Clippy, formatting, diff checks, and Hugo build passed.


## Transition note (2026-08-19T16:48:42+00:00)

Accepted by the owner across agent and non-agent tmux windows. Automated validation passed; the two parallel visual E2E timeouts both passed when rerun serially.
