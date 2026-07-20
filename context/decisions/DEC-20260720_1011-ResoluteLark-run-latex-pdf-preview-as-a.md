---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260720_1011-ResoluteLark-run-latex-pdf-preview-as-a
  created: '2026-07-20T10:11:00+00:00'
spec:
  title: Run LaTeX PDF preview as a Compose sidecar
  state: accepted
  decision: Generate and run the LaTeX EmbedPDF preview as a Docker Compose sidecar.
    The sidecar owns the preview server lifecycle, serves all configured documents,
    listens on the container interface, and is published to the configured host bind
    and port. Remove host-side preview process and PID management from aibox up/down.
  context: The initial LaTeX preview implementation spawned a second host-side aibox
    process after Compose startup. Although simple and loopback-safe, that lifecycle
    was surprising and inconsistent with the expectation that services enabled through
    aibox.toml run in the generated container topology.
  rationale: Compose ownership makes startup, shutdown, rebuilds, logs, and remote-container
    behavior reproducible. Publishing the sidecar port on 127.0.0.1 preserves host-only
    access while the service listens on its container interface.
  alternatives:
  - option: Keep the host-side aibox child process
    reason_rejected: Surprising lifecycle, host PID management, and weaker container
      reproducibility.
  - option: Run the preview inside the main development container
    reason_rejected: Couples a long-running service to the interactive container and
      complicates process supervision.
  consequences: The runtime image must ship a dedicated preview helper or equivalent
    entrypoint. Generated Compose gains a conditional preview service and host port
    mapping. The meaning of preview bind settings must distinguish host publication
    from the sidecar's internal listener.
  decided_at: '2026-07-20T10:11:00+00:00'
---
