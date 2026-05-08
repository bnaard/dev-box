---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_2234-WiseTulip-seed-rs-further-split-v0257
  created: '2026-05-08T22:34:48+00:00'
  labels:
    track: code-quality
    release: v0.25.7
    deferred_from: BACK-20260508_1519-LuckyLily
    deferred_via: DEC-v0.25.6-deferred-item-triage
spec:
  title: 'v0.25.7: Further split cli/src/seed.rs to clear &lt;2,400-line ceiling'
  state: backlog
  type: task
  priority: medium
  description: |
    ## Goal

    Continue the LuckyLily Q3 work and bring `cli/src/seed.rs` under the original <2,400-line acceptance criterion.

    ## Current state (as of v0.25.6 / commit 4cafa9e)

    - `cli/src/seed.rs` is **2,929 lines** — 529 over the ceiling.
    - The first extraction pass shipped in commit `7245992` and produced `cli/src/tmux/`:
      - `mod.rs` — 11 lines
      - `status.rs` — 544 lines (PowerKit settings + status-format rendering, originally `seed.rs:1275-1397`)
      - `layouts.rs` — 255 lines (originally `seed.rs:1400-1502`)
      - `sync.rs` — 127 lines (originally `seed.rs:1606-1656`)

    ## Why Q3 fell short

    The original spec named only the three tmux/PowerKit ranges for extraction. After those moved out, `seed.rs` still hosts a substantial amount of **non-tmux** orchestration: banner generation, `aibox-home` runtime-file synthesis, lockfile fixture emission, etc. The <2,400 ceiling assumed pulling out the named ranges would be enough; in practice it is not.

    ## Work for this WorkItem

    1. **Survey** `seed.rs` and identify cohesive function groups that can move out (candidate destinations: `cli/src/seed/banners.rs`, `cli/src/seed/runtime_home.rs`, `cli/src/seed/locks.rs` or similar).
    2. **Extract** at least one (preferably two) of those groups into submodules under `cli/src/seed/` — turn the current `seed.rs` into `seed/mod.rs` if needed.
    3. **Verify** the public API surface of `seed::*` is unchanged — call sites in `container.rs` and elsewhere should not need adjustment beyond import-path fix-ups.
    4. **Acceptance:** `wc -l cli/src/seed.rs` (or the new `seed/mod.rs`) is **strictly less than 2,400 lines**, and `cargo test` plus `cargo clippy -- -D warnings` remain green.

    ## Dispatch hint

    One general-purpose subagent with read access to `cli/src/seed.rs` and `cli/src/container.rs`. Mechanical extraction; rely on rust-analyzer/compiler for correctness. Should fit in a single session.

    ## Why this is v0.25.7, not v0.25.6

    Per DEC: "v0.25.6 deferred-item triage — drop Q2, defer Q3, resolve the rest" (2026-05-09). Reaching <2,400 needs a second extraction pass that would have churned more of v0.25.6 mid-flight; punted to keep v0.25.6 scope tight.
---
