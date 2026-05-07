---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260507_1336-WarmEmber-default-zellij-status-to-shell-until
  created: '2026-05-07T13:36:34+00:00'
spec:
  title: Default Zellij Status To Shell Until Sidecar CPU Safety Is Proven
  state: accepted
  decision: New and regenerated aibox runtimes should default to `customization.zellij_status.mode
    = "shell"`. The sidecar-backed WASM Zellij key/status rows remain available as
    an explicit opt-in mode, but must not be the default again until derived-project
    CPU stability, permission pre-seeding, and main-container diagnostics correctness
    are covered by release gates.
  context: A derived project on aibox v0.24.2 showed sustained high CPU with Docker
    reporting about 386% CPU. Process inspection identified `/usr/local/bin/zellij
    --server .../aibox` as the owner at about 306% CPU with 31 threads, while Codex,
    processkit gateway, and the diagnostics sidecar were low. The same project also
    showed the status row reading sidecar cgroup data (`MEM ~1 MiB/64 MiB`, `PROC
    1`) while a direct main-container diagnostic reported `MEM ~679 MiB/unlimited`,
    `PROC 96`, and the Zellij permission cache was missing. The recurring release
    pattern is that Zellij plugin regressions affect the whole terminal server rather
    than just the bar.
  rationale: The Zellij WASM plugin boundary is too high-blast-radius for the default
    runtime until it has a hard CPU and permission regression gate in a fresh derived
    project. Shell mode uses Zellij's built-in status bar plus a separate `aibox-status
    --watch` process, so status rendering failures do not pin the Zellij server itself.
    This favors runtime survivability over custom UI until the sidecar design is proven.
  alternatives:
  - option: Keep sidecar as default and patch individual plugin bugs
    reason_not_chosen: The observed failure pins the Zellij server and has recurred
      across releases; individual patches have not provided enough confidence.
  - option: Disable all aibox status rows by default
    reason_not_chosen: Too much loss of useful runtime visibility; shell mode keeps
      visibility with lower blast radius.
  - option: Keep sidecar default only when permission cache exists
    reason_not_chosen: The CPU runaway may involve more than permissions, and diagnostics
      correctness is also currently wrong.
  consequences: Fresh projects and regenerated layouts avoid the sidecar plugin path
    by default. Users can still explicitly select `sidecar` for testing. The powerline/tab-bar
    redesign should build on a safer rendering architecture and must include host/derived-project
    CPU stability checks before becoming default. Release notes must call this a containment
    rollback, not a cosmetic preference change.
  deciders:
  - TEAMMEMBER-cora
  related_workitems:
  - BACK-20260505_2222-KeenHare-investigate-zellij-status-plugin-errors
  - BACK-20260503_0936-SnappyCrane-native-zellij-runtime-status-plugin
  - BACK-20260505_2222-BoldSwan-release-host-runtime-smoke-tests
  decided_at: '2026-05-07T13:36:34+00:00'
---
