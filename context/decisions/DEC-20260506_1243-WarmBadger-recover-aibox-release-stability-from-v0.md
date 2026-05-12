---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260506_1243-WarmBadger-recover-aibox-release-stability-from-v0
  created: '2026-05-06T12:43:47+00:00'
spec:
  title: Recover aibox release stability from v0.23.16 baseline
  state: accepted
  decision: 'Use v0.23.16 as the provisional known-running baseline for release recovery, then reintroduce later runtime changes in narrow steps: native Zellij status/key-hint plugin default from v0.23.17, tool and library bumps including Zellij/Yazi/uv/Cargo from v0.23.17-v0.23.18, generated runtime/Yazi state changes from v0.23.18, and host release-smoke harness changes from v0.23.19.'
  context: 'The v0.23.19 host Phase 2 smoke test failed badly during first real execution, streaming raw terminal escape sequences and requiring container cleanup. Multiple recent changes overlap: the native Zellij status plugin became the generated default, toolchain/runtime dependencies were bumped, generated runtime configs changed, and the host smoke harness was added/hardened.'
  rationale: A rollback-style recovery gives a cleaner causal window than continuing to stack fixes on unstable main. Separating native plugin behavior, toolchain bumps, generated runtime changes, and smoke-harness behavior avoids blaming the wrong layer and keeps the host smoke path safe before it is trusted as a release gate.
  alternatives:
  - option: Continue patching main directly
    reason_not_chosen: Too many overlapping regressions make causality unclear and risk another unsafe host smoke run.
  - option: Blame only the native Zellij status plugin
    reason_not_chosen: The dependency/toolchain bump and the smoke harness itself are plausible contributors and must be isolated.
  - option: Disable all E2E/runtime smoke work permanently
    reason_not_chosen: The problem is unsafe test design and sequencing, not the goal of release coverage.
  consequences: Main is treated as unstable until the recovery branch proves the sequence. Host smoke must be made non-destructive and must not stream raw TUI output before being used as a gate. The toolchain bump, especially Zellij, is handled as its own suspected regression axis rather than folded into the status-plugin hypothesis.
  decided_at: '2026-05-06T12:43:47+00:00'
---
