---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_0629-GentleSeal-defer-rust-crate-updates
  created: '2026-05-08T06:29:07+00:00'
spec:
  title: Defer Rust crate dry-run updates from v0.25.2 release-check-state
  state: backlog
  type: chore
  priority: medium
  description: Release-check-state on 2026-05-08 reported lockfile-resolvable updates
    (js-sys and wasm-bindgen family). Deferred from v0.25.2 release. Before shipping
    this update pass, run `cargo update`, rerun `cargo test`, `cargo clippy --all-targets
    -- -D warnings`, `cargo audit`, `./scripts/maintain.sh test-e2e`, and `./scripts/maintain.sh
    test-e2e-visual-status`.
---
