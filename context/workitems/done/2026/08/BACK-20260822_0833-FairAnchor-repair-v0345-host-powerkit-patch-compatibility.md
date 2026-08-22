---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260822_0833-FairAnchor-repair-v0345-host-powerkit-patch-compatibility
  created: '2026-08-22T08:33:35+00:00'
  labels:
    release: v0.34.5
    component: tmux-powerkit
  updated: '2026-08-22T08:34:34+00:00'
spec:
  title: Repair v0.34.5 host image PowerKit patch compatibility
  state: done
  type: bug
  priority: high
  description: Fix the host Phase 2 runtime image build failure caused by an exact
    escaped-attribute count in patch-powerkit-window-separators.sh. Validate the pinned
    PowerKit 6ac71f0 renderer structurally, add a regression for expanded conditional
    branches, integrate through v0.x-release, and provide a safe immutable-release
    recovery path.
  started_at: '2026-08-22T08:33:44+00:00'
  completed_at: '2026-08-22T08:34:34+00:00'
---

## Transition note (2026-08-22T08:33:44+00:00)

Root cause confirmed against immutable PowerKit pin 6ac71f0d; structural validation fix and expanded-branch regression implemented locally.


## Transition note (2026-08-22T08:34:33+00:00)

Structural patch validation passes the local eight-attribute renderer and immutable pinned nine-attribute renderer; focused Dracula visual, generator, release contract, shell syntax, and diff checks pass.


## Transition note (2026-08-22T08:34:34+00:00)

Fix complete locally and ready for protected v0.x-release integration so host Phase 2 can be re-prepared from the post-tag release-branch commit.
