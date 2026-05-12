---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260507_0629-SunnyPlum-harden-native-zellij-status-plugin-contract
  created: '2026-05-07T06:29:14+00:00'
spec:
  title: Harden Native Zellij Status Plugin Contract Before 0.24.0
  state: accepted
  decision: 'Before releasing aibox 0.24.0, make the native Zellij status integration explicit and release-tested: ship physical WASM files for the default, keybar, and runtime plugin identities; keep permission caches and doctor checks aligned with those identities; and make the E2E companion plus host Phase 2 smoke validate the same file contract and redesigned row text.'
  context: The 0.24.0 release gate exposed repeated status-row failures after recent runtime fixes. The failures had different causes but all surfaced as blank or missing Zellij rows, creating brittle feedback between generated layouts, image contents, permission caches, and visual tests.
  rationale: Zellij itself appears stable enough, but its native plugin contract is path and permission sensitive. Symlink aliases, stale E2E deployment, and old text assertions made the integration too easy to drift. Physical role-specific plugin files plus one shared release contract remove ambiguity across shipped images, companion tests, and host smoke.
  alternatives:
  - option: Continue with symlink aliases
    reason_rejected: Symlinks are plausible in the real image but differ from the passing E2E deployment and keep identity/path ambiguity in a permission-sensitive Zellij integration.
  - option: Disable native plugin rows and use shell status only
    reason_rejected: This avoids the integration surface but gives up the approved Zellij-style status/keybar redesign.
  - option: Bypass the full visual release gate
    reason_rejected: The release changes the visual status rows directly, so bypassing the gate would leave the brittle area unverified.
  consequences: The release is held until the image recipe, generated Dockerfile fallback, E2E runner, visual assertions, doctor checks, and host Phase 2 smoke agree on the same three-plugin contract. Future changes to native status rows should update that single contract rather than treating layout, permissions, and test probes independently.
  decided_at: '2026-05-07T06:29:14+00:00'
---
