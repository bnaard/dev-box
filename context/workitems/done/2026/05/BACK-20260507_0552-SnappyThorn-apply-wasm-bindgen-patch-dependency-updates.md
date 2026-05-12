---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_0552-SnappyThorn-apply-wasm-bindgen-patch-dependency-updates
  created: '2026-05-07T05:52:38+00:00'
  labels:
    release: 0.24.0
    source: release-check-state
    deferred_dependency: rust-wasm-bindgen
  updated: '2026-05-10T03:25:00+00:00'
spec:
  title: Apply wasm-bindgen patch dependency updates after v0.24.0
  state: cancelled
  type: task
  priority: medium
  description: Release-check-state for aibox v0.24.0 reported cargo update --dry-run would update js-sys 0.3.97 -> 0.3.98 and wasm-bindgen, wasm-bindgen-macro, wasm-bindgen-macro-support, wasm-bindgen-shared 0.2.120 -> 0.2.121. Deferred from v0.24.0 to keep dependency churn out of the runtime TUI release. Before shipping, run a real cargo update for these crates, inspect lockfile changes, run cargo test, cargo clippy --all-targets -- -D warnings, cargo audit, and the Zellij WASM plugin build/tests.
  completed_at: '2026-05-10T03:25:00+00:00'
---

## Transition note (2026-05-10T03:25:00+00:00)

Superseded by BACK-PluckyEagle (commit 2c1d224, merge 64ffdcb). Same wasm-bindgen + js-sys patches now in main via PluckyEagle.
