---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_0926-CleverFern-aibox-emergency-harness-entrypoint
  created: '2026-05-07T09:26:28+00:00'
  updated: '2026-05-07T09:44:18+00:00'
spec:
  title: Add aibox emergency harness recovery entrypoint
  state: done
  type: task
  priority: high
  description: Implement aibox emergency <harness> to start/create the main container
    and exec a plain shell or selected AI harness without Zellij/Yazi/status tooling.
    It writes/prints an emergency briefing instructing recovery steps around diagnostics
    snapshots, cgroup pids/memory, Zellij logs, and recent aibox logs.
  parent: BACK-20260507_0925-CuriousTulip-runtime-diagnostics-sidecar-rust-status-emergency
  started_at: '2026-05-07T09:43:53+00:00'
  completed_at: '2026-05-07T09:44:18+00:00'
---

## Transition note (2026-05-07T09:43:53+00:00)

Implementation completed: aibox emergency <harness> starts only the main container and opens a non-Zellij recovery session with an immediate briefing.


## Transition note (2026-05-07T09:44:03+00:00)

Validation complete: emergency help and invalid harness integration tests pass.


## Transition note (2026-05-07T09:44:18+00:00)

Done for this implementation slice. Emergency path intentionally bypasses diagnostics sidecar and all TUI tooling.
