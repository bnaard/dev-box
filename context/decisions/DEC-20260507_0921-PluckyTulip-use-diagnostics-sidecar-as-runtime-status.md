---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260507_0921-PluckyTulip-use-diagnostics-sidecar-as-runtime-status
  created: '2026-05-07T09:21:07+00:00'
spec:
  title: Use Diagnostics Sidecar as Runtime Status Source
  state: accepted
  decision: aibox will introduce a tightly resource-limited diagnostics sidecar that
    owns runtime health/status collection for the main container. aibox-status and
    the Zellij status plugin will become cheap readers/renderers of the latest sidecar
    snapshot rather than independent collectors.
  context: The 2026-05-07 Zellij native status incident showed that collecting runtime
    status from the Zellij refresh path via shell helper fanout can create a process
    storm and make the container nearly non-startable. Host evidence showed PID growth
    from about 962 to over 10134 within minutes, high CPU, no OOM, and thousands of
    shell helper processes. The user accepted the sidecar direction and emphasized
    that the logger must be very resource-limited so logging cannot itself go runaway.
  rationale: A separate sidecar gives postmortem evidence and live status from one
    bounded collection path, reduces duplication between diagnostics and aibox-status,
    and keeps Zellij/plugin rendering out of deep /proc and cgroup collection. It
    also allows independent resource limits and degraded-mode behavior when the main
    container is under PID pressure.
  alternatives:
  - option: Zellij plugin collects logs and status directly
    reason: Rejected because it keeps deep inspection in the UI/plugin refresh path,
      depends on Zellij being healthy, and risks making recovery depend on the failing
      component.
  - option: Keep aibox-status as shell collector
    reason: Rejected because host evidence showed shell fanout from status collection
      was part of the process amplification pattern.
  - option: Main-container daemon only
    reason: Deferred because a sidecar can be separately constrained and can survive/observe
      some main-container failure modes more cleanly.
  consequences: The sidecar must have strict CPU, memory, PID, sampling, and disk
    ring-buffer limits. It should avoid Docker socket access unless explicitly approved,
    avoid /proc/*/cmdline under pressure, and expose a small latest-snapshot contract
    consumed by aibox-status and Zellij. Release gates should validate sidecar resource
    budgets and status degradation behavior.
  decided_at: '2026-05-07T09:21:07+00:00'
---
