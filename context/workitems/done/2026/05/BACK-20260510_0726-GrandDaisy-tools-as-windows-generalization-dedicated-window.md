---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0726-GrandDaisy-tools-as-windows-generalization-dedicated-window
  created: '2026-05-10T07:26:48+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-layouts
    area-2: addons
  updated: '2026-05-10T07:48:50+00:00'
spec:
  title: 'Tools-as-windows generalization: dedicated window per enabled tool addon (lazygit / k9s / btop / lazydocker)'
  state: done
  type: task
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T07:48:18+00:00'
  completed_at: '2026-05-10T07:48:50+00:00'
---

## Transition note (2026-05-10T07:48:50+00:00)

Implemented and merged in commit 15de96b + merge 1c3c885. Dedicated tmux window per enabled tool addon (lazygit/k9s/btop/lazydocker), `tool_windows` parameter threaded through tmux_layout_script. New addon yaml addons/tools/monitoring.yaml (btop apt + lazydocker GitHub releases, default-disabled); k9s already in addons/tools/kubernetes.yaml. Five prefix bindings (`g`/`K`/`B`/`D`/`s`) added to DEFAULT_TMUX_CONF. 894/895 tests pass. Base-image install follow-up filed as BACK-20260510_0748-ToughPanda.
