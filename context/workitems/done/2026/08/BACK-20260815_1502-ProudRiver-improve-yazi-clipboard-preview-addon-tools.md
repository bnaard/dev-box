---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260815_1502-ProudRiver-improve-yazi-clipboard-preview-addon-tools
  created: '2026-08-15T15:02:02+00:00'
  updated: '2026-08-15T15:06:00+00:00'
spec:
  title: Improve Yazi clipboard and preview interaction; repair missing addon tools
  state: done
  type: story
  priority: high
  description: In the maintained v0.x line, add keyboard-driven whole-file clipboard
    yanking from Yazi, propagate Vim visual yanks through tmux to the host clipboard,
    support horizontal preview navigation and feasible preview text selection/yanking,
    and diagnose/fix configured supply-chain tools that are absent after apply/runtime
    recreation.
  started_at: '2026-08-15T15:02:48+00:00'
  completed_at: '2026-08-15T15:06:00+00:00'
---

## Transition note (2026-08-15T15:02:48+00:00)

Diagnosis established: Vim visual yanks already route through aibox-copy; Yazi has a horizontal pager binding but lacks whole-file copy and selectable-preview guidance; derived config proves the host CLI loaded a stale addon catalog and therefore ignored supply-chain.


## Transition note (2026-08-15T15:05:48+00:00)

Implemented generated and image-fallback Yazi bindings for whole-file host copy and selectable read-only Vim preview; documented horizontal/selectable preview behavior; fixed same-version installer early exit so addon catalogs refresh; preview E2E, installer integration tests, fmt, shell syntax, diff check, and clippy pass.


## Transition note (2026-08-15T15:06:00+00:00)

Implementation and verification complete; ready for protected v0.x integration.
