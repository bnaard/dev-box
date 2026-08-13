---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260812_1851-ArtfulHorizon-fix-textual-duplicate-empty-panels
  created: '2026-08-12T18:51:51+00:00'
spec:
  title: Fix duplicated and empty panels in the Textual release-host UI
  state: backlog
  type: bug
  priority: medium
  assignee: TEAMMEMBER-cora
  description: |
    ## Observed regression

    In the Textual release-host GUI:

    1. The log area now renders two bordered boxes nested inside each other. Previously, the log appeared in a single box.
    2. Beneath the task list, an empty bordered box appears before the intended `Problems` heading and Problems panel.

    ## Expected behavior

    - Render the log in exactly one visible bordered panel, without a redundant inner or outer frame.
    - Place the `Problems` heading and its panel directly beneath the task list, without an empty intermediate widget or border.
    - Preserve the recently added Problems filtering, selection, copy, and error-bundle behavior.
    - Add a headless Textual layout/widget-tree regression test that fails if redundant visible containers return.
---
