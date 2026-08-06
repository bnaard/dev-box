---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260803_0713-SnappyOasis-prevent-tier2-release-e2e-contention
  created: '2026-08-03T07:13:55+00:00'
  updated: '2026-08-06T09:07:28+00:00'
spec:
  title: Prevent Tier 2 release E2E resource-contention timeouts
  state: in-progress
  type: bug
  priority: medium
  description: During the v0.29.0 release, the default AIBOX_E2E_TEST_THREADS=4 ran
    the all-addons and LaTeX image builds concurrently with timing-sensitive tmux/Yazi/Vim
    visual-keybinding tests. The builds passed, but eight UI tests hit 60/90-second
    timeouts under companion saturation. Re-running the exact candidate with AIBOX_E2E_TEST_THREADS=1
    passed 39 tests with 0 failures. Adjust release gating or isolate/resource-schedule
    heavy build cases so the canonical default produces deterministic evidence without
    requiring an operator override.
  started_at: '2026-08-06T09:07:28+00:00'
---

## Transition note (2026-08-06T09:07:28+00:00)

Approved optimization implementation added overlay-capable companion storage with VFS fallback, deterministic core/addon/latex shards with candidate-bound reuse, and cumulative attempt-aware timing. Runtime benchmark remains pending after companion rebuild.
