---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260507_0957-TallFox-rename-zellij-status-modes-to-sidecar
  created: '2026-05-07T09:57:56+00:00'
spec:
  title: Rename Zellij Status Modes To Sidecar And Disabled
  state: accepted
  decision: Rename user-facing Zellij status mode values from `native | shell | hidden`
    to `sidecar | shell | disabled`, while keeping legacy aliases for compatibility
    where practical. Introduce release-runtime-smoke tiers so the default host smoke
    validates the minimal generated runtime and the full addon-heavy smoke is opt-in.
  context: After replacing Bash status fan-out with a Rust diagnostics sidecar and
    Rust status reader, `native` no longer describes the user-facing data path. The
    default Phase 2 smoke was also too expensive because it always installed git-ui
    and preview addons.
  rationale: '`sidecar` names the runtime-status data contract; `disabled` is clearer
    than `hidden`; keeping `shell` preserves the fallback status-bar mode. Tiered
    release smoke preserves confidence while avoiding slow addon installation in the
    default path.'
  alternatives:
  - option: Keep native and hidden
    reason: Rejected because the terms describe implementation details or UI state
      less accurately after the sidecar design.
  - option: Always run full smoke with all addons
    reason: Rejected because it makes every Phase 2 release pay for unrelated addon
      installation and slows the feedback loop.
  consequences: Configuration and CLI help should prefer `sidecar | shell | disabled`.
    Existing `native` and `hidden` configs should be accepted as legacy aliases during
    transition. Release smoke defaults to a smaller profile and only runs git-ui/preview/lazygit
    checks in heavier tiers.
  decided_at: '2026-05-07T09:57:56+00:00'
---
