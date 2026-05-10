---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260505_2153-ToughHawk-review-docusaurus-3-10-1-upgrade
  created: '2026-05-05T21:53:29+00:00'
  labels:
    source: release-v0.23.17
    dependency: docusaurus
    deferred: true
  updated: '2026-05-10T03:24:51+00:00'
spec:
  title: Review Docusaurus 3.10.1 upgrade
  state: done
  type: task
  priority: medium
  description: During the aibox v0.23.17 docs build, Docusaurus reported an available
    update from 3.9.2 to 3.10.1. Review the Docusaurus release notes, update docs-site
    package dependencies if compatible, rebuild/deploy docs locally, and either ship
    the update in a later release or document why it remains deferred.
  started_at: '2026-05-09T22:31:54+00:00'
  completed_at: '2026-05-10T03:24:51+00:00'
---

## Transition note (2026-05-09T22:31:54+00:00)

Bumping global Docusaurus pin from 3.8 to 3.10.1 on branch v0.25.7/toughhawk-docusaurus-310.


## Transition note (2026-05-09T22:32:05+00:00)

Bumped global Docusaurus pin from 3.8 to 3.10.1. Decision: ship the bump. The 3.8 hold was a guard against the 3.9.2 ProgressPlugin regression; docs-site/package.json had already moved to 3.9.2, confirming that blocker is resolved. 3.10.1 is a stable minor in the same 3.x line. Committed on branch v0.25.7/toughhawk-docusaurus-310 (commit d8ed753). EagerDew unaffected — it installs from docs-site/package.json, not the container global install.


## Transition note (2026-05-10T03:24:51+00:00)

Implemented and merged in commit d8ed753 + merge c03d287. Bumped Docusaurus pin 3.8 → 3.10.1 in aibox.toml + addons/docs/docs-docusaurus.yaml. EagerDew compatibility verified.
