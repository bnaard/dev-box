---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1203-PluckyEagle-review-v0242-dependency-drift
  created: '2026-05-07T12:03:01+00:00'
  labels:
    release: v0.24.2
    area: dependencies
    source: dist/RELEASE-STATE.md
spec:
  title: Review deferred dependency and harness drift after aibox v0.24.2
  state: backlog
  type: chore
  priority: medium
  description: 'Release v0.24.2 deferred dependency freshness work from dist/RELEASE-STATE.md.
    Concrete deferred updates: uv image 0.11.10 -> 0.11.11; Rust crate family js-sys/wasm-bindgen/wasm-bindgen-macro/wasm-bindgen-macro-support/wasm-bindgen-shared
    0.2.120 -> 0.2.121. Review surfaces also called out by the release report: Node.js
    22 stream v22.22.2, Debian trixie-slim rebuild/security posture, and latest-by-default
    AI harness install/layout/auth/config surfaces. Required validation before shipping
    later: apply selected updates, rerun cargo test, cargo clippy --all-targets --
    -D warnings, cargo audit, release-check-state, and runtime visual status/Yazi
    smoke if any runtime image or harness install surface changes.'
  scope: release-follow-up
---
