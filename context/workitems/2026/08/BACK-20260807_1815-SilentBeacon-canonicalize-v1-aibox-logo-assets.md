---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260807_1815-SilentBeacon-canonicalize-v1-aibox-logo-assets
  created: '2026-08-07T18:15:20+00:00'
spec:
  title: Make the v1 aibox logo the cross-line canonical asset source
  state: backlog
  type: task
  priority: medium
  description: The v0.x assets/logo bundle still contains the older terminal/spark
    mark while the v1.x documentation workstream contains the owner-preferred hexagonal
    AI-box mark. Promote the approved v1 mark into the canonical cross-line asset
    bundle, regenerate every SVG/ICO/PNG/touch-icon variant from that source, align
    v0.x and v1.x documentation consumers, and add a check that prevents navbar/favicon
    assets from drifting across maintained lines.
---
