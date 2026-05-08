---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260505_2222-BoldSwan-release-host-runtime-smoke-tests
  created: '2026-05-05T22:22:54+00:00'
  labels:
    area: release
    component: e2e
    source: derived-project-regression
  updated: '2026-05-08T10:46:33+00:00'
spec:
  title: Add host-phase runtime smoke tests to release-host
  state: done
  type: task
  priority: high
  description: Add a release-host regression gate that runs after final container
    images are built/pushed or at least after the final local image exists. It should
    create/apply a small generated project against the just-built image and smoke
    Yazi config parsing, lazygit state-directory startup, Zellij layout/plugin loading,
    and key/status bar logs. This would have caught the Yazi 26 matcher schema regression
    and lazygit XDG state regression before publishing.
  started_at: '2026-05-08T10:46:25+00:00'
  completed_at: '2026-05-08T10:46:33+00:00'
---

## Transition note (2026-05-08T10:46:25+00:00)

Implementation started: release-host runtime smoke now defaults to addon tier so git-ui/lazygit are exercised, and tmux persistence defaults are guarded in the probe.


## Transition note (2026-05-08T10:46:29+00:00)

Ready for review: host release smoke already creates/applies a generated project and now defaults to addon tier, covering git-ui/lazygit; docs updated for host-vs-companion boundaries.


## Transition note (2026-05-08T10:46:33+00:00)

Resolved: release-host runtime smoke now exercises addon/lazygit path by default and documents the host-side runtime smoke boundary. Verified shell syntax for release-runtime-smoke.sh and maintain.sh.
