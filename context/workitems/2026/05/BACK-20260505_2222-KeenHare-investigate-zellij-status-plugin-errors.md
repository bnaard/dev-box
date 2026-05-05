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
spec:
  title: Investigate native Zellij status/key plugin runtime errors
  state: backlog
  type: bug
  priority: high
  description: 'Derived project on aibox v0.23.17 shows both custom Zellij key/status
    plugin rows as `ERROR IN PLUGIN - see logs`. Evidence already rules out missing
    assets and helper failure: `/usr/local/share/aibox/zellij/aibox-status.wasm` exists,
    `/usr/local/bin/aibox-status` exists, and `/usr/local/bin/aibox-status --plugin-json`
    returns valid JSON. Need collect actual Zellij plugin host logs from `/tmp/**/zellij-log/zellij.log`,
    identify whether this is WASM load/permission/ABI/render panic, and fix source
    plus add a focused regression guard.'
---
