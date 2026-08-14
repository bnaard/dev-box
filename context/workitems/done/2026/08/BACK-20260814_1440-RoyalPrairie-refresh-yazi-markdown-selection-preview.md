---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260814_1440-RoyalPrairie-refresh-yazi-markdown-selection-preview
  created: '2026-08-14T14:40:04+00:00'
  updated: '2026-08-14T14:45:08+00:00'
spec:
  title: Refresh Yazi Markdown preview when selection changes
  state: done
  type: bug
  priority: high
  description: Reproduce and fix the v0.x Yazi rich Markdown preview retaining the
    first file in a directory when subsequent Markdown files are selected. Add regression
    coverage for per-file preview identity and refresh behavior.
  started_at: '2026-08-14T14:40:09+00:00'
  completed_at: '2026-08-14T14:45:08+00:00'
---

## Transition note (2026-08-14T14:40:09+00:00)

Tracing generated rich-preview plugin cache and Yazi job identity before patching.


## Transition note (2026-08-14T14:45:08+00:00)

Full-path cache identity implemented; unit/E2E regression, full suite, format, and clippy verified. Parallel tmux timeouts passed serially.


## Transition note (2026-08-14T14:45:08+00:00)

Accepted: sibling Markdown files now receive distinct bounded cache keys derived from their complete paths.
