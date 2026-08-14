---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260814_1458-BrightButter-preserve-yazi-hugo-frontmatter-lines
  created: '2026-08-14T14:58:11+00:00'
  updated: '2026-08-14T15:03:14+00:00'
spec:
  title: Preserve Hugo front matter lines in Yazi preview
  state: done
  type: bug
  priority: high
  description: Fix rich-preview so leading TOML (+++) and YAML (---) front matter
    is rendered verbatim instead of being collapsed by Markdown soft-break semantics.
    Cover the supplied about.md fixture behavior.
  started_at: '2026-08-14T14:58:17+00:00'
  completed_at: '2026-08-14T15:03:14+00:00'
---

## Transition note (2026-08-14T14:58:17+00:00)

Derived screenshot and fixture confirm Rich collapses front matter as Markdown soft breaks.


## Transition note (2026-08-14T15:03:13+00:00)

Implemented verbatim TOML/YAML front-matter rendering in in-pane and full-pane preview paths; focused and full verification completed.


## Transition note (2026-08-14T15:03:14+00:00)

Accepted: front matter retains source rows while the document body keeps normal Markdown wrapping semantics.
