---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260725_2052-ResoluteMoss-tmux-forge-open-discussions
  created: '2026-07-25T20:52:03+00:00'
  updated: '2026-07-25T21:01:02+00:00'
spec:
  title: Show open GitHub Discussions in tmux forge status
  state: done
  type: bug
  priority: medium
  description: Extend the aibox-shipped PowerKit forge plugin on both v0.x and v1.x
    to count repository-global open GitHub Discussions, render them alongside open
    issues and pull requests, degrade gracefully when Discussions or GraphQL access
    is unavailable, and cover command construction plus rendering with focused regression
    tests.
  started_at: '2026-07-25T20:57:07+00:00'
  completed_at: '2026-07-25T21:01:02+00:00'
---

## Transition note (2026-07-25T20:57:07+00:00)

Implemented exact repository-global GraphQL totalCount metrics for open issues, pull requests, and Discussions on v0.x; focused shell, rendering, and Rust configuration tests pass. Porting the same change to v1.x.


## Transition note (2026-07-25T21:00:51+00:00)

Both version-line commits are clean. Focused forge tests, bash syntax, cargo fmt, zero-warning clippy, and the complete Rust test suites pass on v0.x and v1.x.


## Transition note (2026-07-25T21:01:02+00:00)

Implemented and validated on both v0.x and v1.x. GitHub issue, pull-request, and Discussion counts are exact repository-global GraphQL totals; Discussions render as D<count> and failures degrade independently.
