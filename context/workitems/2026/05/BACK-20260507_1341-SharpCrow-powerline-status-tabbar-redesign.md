---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1341-SharpCrow-powerline-status-tabbar-redesign
  created: '2026-05-07T13:41:16+00:00'
  labels:
    area: zellij-status
    design: powerline
    blocked_by: zellij-sidecar-stability
spec:
  title: Backlog powerline-style status and tab bar redesign behind stable non-WASM path
  state: backlog
  type: story
  priority: medium
  description: 'Defer visual redesign until status transport is stable. Desired design: powerline-like bidirectional chevron segments based on /tmp/design-input/, preferring the outgoing/incoming chevron style in dual-line-status-bar.png rather than the outgoing chevron plus rectangular start in zellij-style-bars.svg. Include a powerline-style tab bar listing all open screens/tabs in the current session with numeric shortcuts. Acceptance: design works without making the Zellij WASM sidecar the default; sidecar remains opt-in until CPU stability gates pass.'
  scope: runtime-ui
---
