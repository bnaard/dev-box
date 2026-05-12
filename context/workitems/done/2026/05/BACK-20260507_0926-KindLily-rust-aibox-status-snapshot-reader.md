---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_0926-KindLily-rust-aibox-status-snapshot-reader
  created: '2026-05-07T09:26:28+00:00'
  updated: '2026-05-07T09:43:44+00:00'
spec:
  title: Replace aibox-status shell helper with Rust snapshot reader
  state: done
  type: task
  priority: high
  description: Implement a Rust aibox-status executable with no Bash fanout. Preserve default plain output, --plugin-json, and --watch. Read the diagnostics sidecar latest snapshot and degrade quickly when missing/stale. Add tests proving JSON shape and no shell collector path remains.
  parent: BACK-20260507_0925-CuriousTulip-runtime-diagnostics-sidecar-rust-status-emergency
  started_at: '2026-05-07T09:43:26+00:00'
  completed_at: '2026-05-07T09:43:44+00:00'
---

## Transition note (2026-05-07T09:43:26+00:00)

Implementation completed: aibox-status is now an image-owned Rust executable that reads bounded diagnostics snapshots and falls back to direct Rust proc/cgroup reads without Bash fan-out.


## Transition note (2026-05-07T09:43:37+00:00)

Implementation and validation complete; moving through review per workitem state machine.


## Transition note (2026-05-07T09:43:44+00:00)

Validated with rustc -D warnings, plugin JSON smoke, focused tests, full CLI unit suite, clippy -D warnings, and WASM status plugin build after installing wasm32-wasip1 locally.
