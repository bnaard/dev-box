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
spec:
  title: Add host-phase runtime smoke tests to release-host
  state: backlog
  type: task
  priority: high
  description: Add a release-host regression gate that runs after final container
    images are built/pushed or at least after the final local image exists. It should
    create/apply a small generated project against the just-built image and smoke
    Yazi config parsing, lazygit state-directory startup, Zellij layout/plugin loading,
    and key/status bar logs. This would have caught the Yazi 26 matcher schema regression
    and lazygit XDG state regression before publishing.
---
