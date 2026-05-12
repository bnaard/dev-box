---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_0925-CuriousTulip-runtime-diagnostics-sidecar-rust-status-emergency
  created: '2026-05-07T09:25:35+00:00'
  labels:
    decision: DEC-20260507_0921-PluckyTulip-use-diagnostics-sidecar-as-runtime-status
    status_decision: DEC-20260507_0925-WiseClover-implement-aibox-status-as-rust-snapshot
    incident: 2026-05-07-zellij-status-process-storm
  updated: '2026-05-07T09:44:19+00:00'
spec:
  title: Implement diagnostics sidecar, Rust aibox-status, and emergency recovery command
  state: done
  type: epic
  priority: high
  description: 'Accepted implementation plan after the 2026-05-07 Zellij native status process storm. Scope: add a tightly resource-limited diagnostics sidecar as the runtime/status collector; replace shell aibox-status with a Rust executable that reads sidecar snapshots with no Bash fanout; keep Zellij plugin as renderer only; add aibox emergency <harness> to start a plain shell/agent path bypassing Zellij/Yazi/status; add tests and release gates for PID/resource budgets, stale diagnostics, and emergency access. Constraints: sidecar must have strict CPU, memory, PID, sampling, and disk ring-buffer limits; avoid Docker socket unless explicitly approved; avoid /proc/*/cmdline under pressure; do not edit context/templates by hand.'
  started_at: '2026-05-07T09:43:53+00:00'
  completed_at: '2026-05-07T09:44:19+00:00'
---

## Transition note (2026-05-07T09:43:53+00:00)

Implementation slice completed across Rust status, diagnostics sidecar, and emergency entrypoint; validation in progress.


## Transition note (2026-05-07T09:44:04+00:00)

All child implementation slices are complete; final validation has one unrelated processkit-fetch-dependent E2E failure in restricted network environment.


## Transition note (2026-05-07T09:44:19+00:00)

Implementation complete and source validation passes. Residual follow-up: review pending MIG-RUNTIME-20260507T093715 and rerun network-dependent processkit E2E when processkit fetch is available.
