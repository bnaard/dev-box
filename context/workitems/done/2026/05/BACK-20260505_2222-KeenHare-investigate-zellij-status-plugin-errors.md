---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260505_2222-KeenHare-investigate-zellij-status-plugin-errors
  created: '2026-05-05T22:22:50+00:00'
  labels:
    area: runtime-ui
    component: zellij
    source: derived-project-regression
  updated: '2026-05-09T22:18:32+00:00'
spec:
  title: Investigate native Zellij status/key plugin runtime errors
  state: cancelled
  type: bug
  priority: high
  description: 'Derived project on aibox v0.23.17 shows both custom Zellij key/status plugin rows as `ERROR IN PLUGIN - see logs`. Evidence already rules out missing assets and helper failure: `/usr/local/share/aibox/zellij/aibox-status.wasm` exists, `/usr/local/bin/aibox-status` exists, and `/usr/local/bin/aibox-status --plugin-json` returns valid JSON. Need collect actual Zellij plugin host logs from `/tmp/**/zellij-log/zellij.log`, identify whether this is WASM load/permission/ABI/render panic, and fix source plus add a focused regression guard.'
  completed_at: '2026-05-09T22:18:32+00:00'
---

## Transition note (2026-05-09T22:18:32+00:00)

Closed as obsolete: Zellij was fully excised in the v0.25.x NobleCrane migration. The native Zellij status/key plugins this WorkItem investigated no longer exist in aibox; the runtime is now tmux-only. No fix path remains.
