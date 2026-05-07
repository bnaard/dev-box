---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_1456-CalmRabbit-rewrite-visual-e2e-tests-tmux
  created: '2026-05-07T14:56:07+00:00'
  updated: '2026-05-07T15:08:48+00:00'
spec:
  title: Rewrite runtime and visual E2E tests for tmux while preserving test intent
  state: review
  type: task
  priority: high
  description: 'Keep the sidecar container and visual testing paradigm, but discard
    current Zellij-shaped test implementations. Preserve the intent of each visual/runtime
    test, then reimplement from scratch against tmux sessions, panes, status rendering,
    Yazi/Vim workflows, clipboard behavior, and no-Zellij negative assertions. Related
    decision: DEC-20260507_1447-VastLeaf-remove-zellij-and-rebuild-aibox-runtime.'
  parent: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  scope: runtime-tests
  started_at: '2026-05-07T14:57:31+00:00'
---

## Transition note (2026-05-07T14:57:31+00:00)

Worker 2 started runtime/visual test rewrite in forked workspace; scope limited to cli/tests/e2e visual/runtime surfaces and related helpers.


## Transition note (2026-05-07T15:08:48+00:00)

Worker 2 completed scoped tmux rewrite of runtime/visual E2E tests and helper deployment. Owner constraint applied: tests no longer assume tmux-resurrect or tmux-continuum are active by default. Validation: cargo fmt; cargo clippy --features e2e --all-targets -- -D warnings; cargo test --features e2e --no-run. Live companion scenarios were not executed because production tmux runtime changes are concurrent/in flight.
