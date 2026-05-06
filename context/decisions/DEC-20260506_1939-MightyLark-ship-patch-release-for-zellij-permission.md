---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260506_1939-MightyLark-ship-patch-release-for-zellij-permission
  created: '2026-05-06T19:39:22+00:00'
spec:
  title: Ship Patch Release for Zellij Permission Cache Guard
  state: accepted
  decision: Ship a new aibox patch release containing the native Zellij status permission-cache
    doctor guard, generated-compose E2E coverage, and all unreleased runtime TUI startup
    hardening already on main after v0.23.20.
  context: The user requested commit, push, and a patch release after the native Zellij
    status plugin prompted for permissions again. Diagnosis showed the active project
    had native status mode and seeded permission files, but a stale v0.23.20 generated
    compose shape omitted the cache mounts, allowing `aibox apply` from an older generator
    to overwrite the runtime projection without the needed mounts.
  rationale: A patch release is warranted because the failure path affects downstream
    generated devcontainers, and the new doctor guard catches the specific systemic
    stale-generator/config projection mismatch before users recreate containers with
    missing Zellij permission-cache mounts.
  alternatives:
  - option: Keep the fix local until a later release
    tradeoff: Would leave downstream users exposed to the same stale generated compose
      overwrite path.
  - option: Switch the project back to shell status mode
    tradeoff: Would avoid the native plugin prompt but retreat from the intended native
      status default and fail to address stale generated compose projection.
  consequences: Phase 1 will be performed with the scripted release path. Phase 2
    remains host-only via `./scripts/maintain.sh release-host <version>`. The release
    should mention that the fix detects and prevents native Zellij status permission-cache
    projection drift rather than only reseeding local files.
  deciders:
  - TEAMMEMBER-cora
  decided_at: '2026-05-06T19:39:22+00:00'
---
