---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260720_1124-CuriousDaisy-keep-latex-builds-inside-the-development
  created: '2026-07-20T11:24:09+00:00'
spec:
  title: Keep LaTeX builds inside the development container
  state: accepted
  decision: Remove the host aibox latex and preview latex command surfaces. Derived
    projects build and continuously watch LaTeX documents only through managed scripts
    deployed into the development container. The Compose preview sidecar remains read-only
    and only serves completed PDFs with browser reload notifications.
  context: The initial sidecar change preserved host CLI commands that either built
    on the host or delegated into the container. This mixed execution model was inconsistent
    with aibox reproducibility and with the previously established managed in-container
    LaTeX scripts.
  rationale: TeX engines, packages, fonts, caches, and build scripts must come from
    the project image so builds are reproducible. Keeping the serving sidecar read-only
    gives it a narrow security boundary while container-local scripts retain explicit
    control over one-shot and watch builds.
  alternatives:
  - option: Let the preview sidecar also supervise latexmk watchers
    reason_rejected: Unnecessary for the first correction and would combine build
      write access with the read-only serving boundary.
  - option: Retain host CLI wrappers that delegate to the container
    reason_rejected: Adds a redundant and confusing host surface for operations that
      belong inside the development environment.
  consequences: The earlier accepted sidecar decision remains valid only for serving
    and lifecycle ownership; its host build/watch/status command portion is superseded.
    Documentation and agent guidance must use the deployed in-container scripts.
  decided_at: '2026-07-20T11:24:09+00:00'
---
