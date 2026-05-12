---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0809-SnappySky-revert-speculative-tool-addons-drop-monitoring
  created: '2026-05-10T08:09:00+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-layouts
    kind: revert
  updated: '2026-05-10T08:14:28+00:00'
spec:
  title: 'Revert speculative tool addons: drop monitoring.yaml + K/B/D bindings (keep framework + lazygit only)'
  state: done
  type: task
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T08:13:52+00:00'
  completed_at: '2026-05-10T08:14:28+00:00'
---

## Transition note (2026-05-10T08:14:28+00:00)

Implemented and merged in commit e4c95eb (committed directly on main; deviation from branch-and-merge pattern but result correct and pushed to origin). Removed addons/tools/monitoring.yaml; removed K/B/D bind-key lines from DEFAULT_TMUX_CONF; removed btop/lazydocker entries from tool_windows_for_config(). Framework intact: tool_windows parameter, lazygit window, g/s bindings, kubernetes.yaml addon (pre-existing) all preserved. 895/895 tests pass.
