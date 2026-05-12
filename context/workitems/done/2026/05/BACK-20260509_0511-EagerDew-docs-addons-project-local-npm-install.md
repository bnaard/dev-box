---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_0511-EagerDew-docs-addons-project-local-npm-install
  created: '2026-05-09T05:11:22+00:00'
  labels:
    track: addons
    release: v0.25.7
    surfaced_during: v0.25.6 release Phase 1 docs-deploy
  updated: '2026-05-09T22:19:07+00:00'
spec:
  title: 'v0.25.7: docs addons should run project-local `npm install` (prism-react-renderer surprise)'
  state: done
  type: task
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-09T22:18:28+00:00'
  completed_at: '2026-05-09T22:19:07+00:00'
---

## Transition note (2026-05-09T22:19:07+00:00)

Implemented and merged in commit fac2aa1 + merge b36241f. New cli/src/docs_install.rs (~300 LOC + 11 unit tests), wired in container.rs::cmd_sync.
