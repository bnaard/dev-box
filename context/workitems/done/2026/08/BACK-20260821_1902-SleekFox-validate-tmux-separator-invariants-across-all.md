---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260821_1902-SleekFox-validate-tmux-separator-invariants-across-all
  created: '2026-08-21T19:02:34+00:00'
  updated: '2026-08-21T19:13:33+00:00'
spec:
  title: Validate tmux separator invariants across all themes
  state: done
  type: task
  priority: high
  assignee: TEAMMEMBER-avery
  description: Generalize the PowerKit window separator regression so it derives and
    validates status-gap, index, name, outgoing-arrow, rounded session edge, comma
    escaping, and style-reset invariants for every supported aibox theme in isolated
    tmux sessions without changing the live project theme.
  parent: BACK-20260821_1839-AssuredMaple-reset-inherited-dim-style-on-tmux
  started_at: '2026-08-21T19:02:41+00:00'
  completed_at: '2026-08-21T19:13:33+00:00'
---

## Transition note (2026-08-21T19:02:41+00:00)

Generalizing the focused real-tmux proof into a supported-theme matrix while preserving the live theme.


## Transition note (2026-08-21T19:13:32+00:00)

Added an exhaustive 76-theme PowerKit semantic-role matrix and retained isolated real-tmux parser/color/style-reset proof; all theme tests, clippy, fmt, shellcheck, and diff checks pass.


## Transition note (2026-08-21T19:13:33+00:00)

Verified all 76 exposed themes define valid status/session/active/inactive PowerKit roles and supported emphasis attributes. Real-tmux separator proof passes independently of palette. Existing screencast theme harness was found to consume the live palette and was not used as evidence; exploratory changes were removed.
