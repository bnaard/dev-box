---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260502_0709-SmoothSea-optional-gh-lazygit-addons
  created: '2026-05-02T07:09:10+00:00'
  updated: '2026-05-02T07:31:40+00:00'
spec:
  title: Prepare optional tool addons for GitHub CLI and LazyGit
  state: done
  type: task
  priority: medium
  description: Move gh and later lazygit from fixed base-image packages to selectable addons, with migration-safe layout/tool handling and docs. Start with design and gh addon; lazygit depends on layout generator awareness of tool availability.
  started_at: '2026-05-02T07:31:33+00:00'
  completed_at: '2026-05-02T07:31:40+00:00'
---

## Transition note (2026-05-02T07:31:33+00:00)

Implemented git-ui addon for gh and lazygit and moved both tools out of the base image.


## Transition note (2026-05-02T07:31:36+00:00)

Verified generated project Dockerfile installs gh/lazygit via git-ui addon; base image Dockerfile no longer installs them directly; layout/addon/clippy checks pass.


## Transition note (2026-05-02T07:31:40+00:00)

Completed and verified. This repo selects git-ui in aibox.toml to preserve maintenance workflows; derived projects can omit it to avoid installing gh/lazygit.
