---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260819_1648-StableJay-establish-new-documentation-basis
  created: '2026-08-19T16:48:12+00:00'
  updated: '2026-08-19T18:00:12+00:00'
spec:
  title: Establish the new documentation basis before the next release
  state: done
  type: epic
  priority: high
  assignee: TEAMMEMBER-cora
  description: Define and implement the new documentation basis before any next release
    work begins. Start by identifying the intended audiences, information architecture,
    source-of-truth boundaries, authoring/build foundation, migration approach, and
    verification criteria. The owner will provide or confirm the desired basis in
    the next work session; do not start release preparation until this item is resolved.
  scope: Documentation foundation required before beginning the next aibox release.
  started_at: '2026-08-19T16:58:05+00:00'
  completed_at: '2026-08-19T18:00:12+00:00'
---

## Transition note (2026-08-19T16:58:05+00:00)

Owner supplied the documentation basis: replace Docsy/Hextra entirely with brand-theme-hugo-vanilla v0.3.4, migrate content idiomatically, add Change log and Roadmap visuals, track upstream theme gaps, and expose local Hugo watch on port 1316.


## Transition note (2026-08-19T17:18:48+00:00)

Implemented brand-theme-hugo-vanilla v0.3.4 migration: removed Docsy/Bootstrap/Font Awesome and legacy player dependencies, migrated content to theme-native constructs, added Change log and Roadmap, enabled Go-backed Hugo Modules, passed production build and GitHub Pages dry run, crawled 66 local routes with zero broken links, exposed live watch on port 1316, and filed upstream gap issue #58. Awaiting owner visual acceptance.


## Transition note (2026-08-19T18:00:12+00:00)

Implemented Hugo brand-theme v0.3.4 migration, transposed content, added Change log and Roadmap, applied owner corrections, disabled the unverified gallery, documented upstream theme gaps, and passed docs build/deploy dry run plus Rust fmt, clippy, and tests.
