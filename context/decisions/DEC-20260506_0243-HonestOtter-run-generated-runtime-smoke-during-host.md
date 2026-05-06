---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260506_0243-HonestOtter-run-generated-runtime-smoke-during-host
  created: '2026-05-06T02:43:49+00:00'
spec:
  title: Run Generated Runtime Smoke During Host-Side Release
  state: accepted
  decision: The host-side `release-host` phase must run an automated generated-runtime
    smoke after GHCR image push and write a timestamped log bundle under `dist/release-smoke/`.
    The smoke creates a fresh downstream-style project, runs `aibox init`, runs `aibox
    apply --no-cache --standardize-config`, starts the generated container, and probes
    Yazi, lazygit, the aibox status helper, and Zellij plugin logs.
  context: Recent derived-project regressions in Yazi config parsing, lazygit state
    directories, and Zellij status/key plugin startup were only caught after a patch
    release. The existing SSH companion E2E suite still exists, but host Phase 2 did
    not automatically validate the pushed release image with a fresh generated project.
  rationale: Putting this smoke inside `release-host` makes the validation mandatory
    at the point where macOS binaries, GHCR images, and host container runtime access
    are all available. Writing logs automatically gives the agent a durable evidence
    bundle to inspect after the owner runs Phase 2.
  consequences: A host-side release now takes longer and depends on Docker or Podman
    being usable on the host. A smoke failure fails the host phase after logs are
    captured, while preserving the temporary smoke project/container for inspection.
  decided_at: '2026-05-06T02:43:49+00:00'
---
