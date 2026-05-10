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
  updated: '2026-05-10T03:24:45+00:00'
spec:
  title: Review deferred dependency and harness drift after aibox v0.24.2
  state: done
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
  started_at: '2026-05-09T22:36:57+00:00'
  completed_at: '2026-05-10T03:24:45+00:00'
---

## Transition note (2026-05-09T22:36:57+00:00)

Started drift review on branch v0.25.7/pluckyeagle-v0242-drift.


## Transition note (2026-05-09T22:37:10+00:00)

Drift review completed on branch v0.25.7/pluckyeagle-v0242-drift, commit 2c1d224.

Applied (all patch-level, lockfile-resolvable, cargo audit clean):
- js-sys 0.3.97 → 0.3.98
- wasm-bindgen/macro/macro-support/shared 0.2.120 → 0.2.121
- cc 1.2.61 → 1.2.62, filetime 0.2.27 → 0.2.28, hashbrown 0.17.0 → 0.17.1
- plain 0.2.3 removed, redox_syscall 0.7.5 removed

Pre-existing Rust 1.94 lint/fmt issues fixed in same commit (lock.rs, addon_disablement.rs, docs_install.rs).

Deferred (base-image rebuild required):
- uv 0.11.10 → 0.11.12: BACK-20260508_1214-SureSeal, BACK-20260507_0552-BraveFalcon
- Node.js 22 v22.22.2: floating major, review-only surface
- Debian trixie-slim: floating tag, review-only
- AI harnesses: latest-by-default, no action needed

Validation: audit clean, clippy -D warnings clean, fmt clean, 851/852 tests pass.


## Transition note (2026-05-10T03:24:45+00:00)

Implemented and merged in commit 2c1d224 + merge 64ffdcb. Broadest dependency sweep: 8 patch bumps + 2 dep removals + pre-existing clippy/fmt fixes for lock.rs, addon_disablement.rs, docs_install.rs. Surfaced uv 0.11.12 follow-up via NOTE-VastVale.
