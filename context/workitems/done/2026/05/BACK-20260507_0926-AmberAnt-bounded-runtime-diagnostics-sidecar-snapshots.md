---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_0926-AmberAnt-bounded-runtime-diagnostics-sidecar-snapshots
  created: '2026-05-07T09:26:33+00:00'
  updated: '2026-05-07T09:44:18+00:00'
spec:
  title: Add bounded diagnostics sidecar for runtime snapshots
  state: done
  type: task
  priority: high
  description: Add an aibox diagnostics sidecar/service with strict CPU, memory, PID, sampling, and disk ring-buffer limits. It owns cgroup/procfs/Zellij log collection for the main container, avoids Docker socket by default, avoids /proc/*/cmdline under pressure, and writes latest.json plus bounded history for postmortem and live status.
  parent: BACK-20260507_0925-CuriousTulip-runtime-diagnostics-sidecar-rust-status-emergency
  started_at: '2026-05-07T09:43:53+00:00'
  completed_at: '2026-05-07T09:44:18+00:00'
---

## Transition note (2026-05-07T09:43:53+00:00)

Implementation completed: Compose now emits a resource-limited diagnostics sidecar with shared PID namespace, bounded PID/memory/CPU settings, and snapshot output consumed by aibox-status.


## Transition note (2026-05-07T09:44:03+00:00)

Validation complete: generated Compose sidecar test passes, rustc -D warnings passes, degraded diagnostics snapshot smoke writes valid latest.json.


## Transition note (2026-05-07T09:44:18+00:00)

Done for this implementation slice. Release still needs host/container smoke with rebuilt image and pending runtime migration review.
