---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1214-TallFrog-review-wasm-bindgen-updates
  created: '2026-05-08T12:14:14+00:00'
  labels:
    release: 0.25.5
    dependency: rust-crates
    deferred: true
spec:
  title: Review wasm-bindgen and js-sys lockfile updates after v0.25.5
  state: backlog
  type: task
  priority: medium
  description: 'Release-state check for aibox v0.25.5 reported cargo update --dry-run
    would update js-sys 0.3.97 -> 0.3.98 and wasm-bindgen crates 0.2.120 -> 0.2.121.
    Deferred from v0.25.5 because the patch release is scoped to tmux runtime attach/session
    fixes. Validation required before shipping later: run real cargo update for that
    set, review lockfile diff, and rerun fmt, clippy, cargo test, and relevant release/E2E
    gates.'
  scope: release-dependency-followup
---
