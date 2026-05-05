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
spec:
  title: Review Docusaurus 3.10.1 upgrade
  state: backlog
  type: task
  priority: medium
  description: During the aibox v0.23.17 docs build, Docusaurus reported an available
    update from 3.9.2 to 3.10.1. Review the Docusaurus release notes, update docs-site
    package dependencies if compatible, rebuild/deploy docs locally, and either ship
    the update in a later release or document why it remains deferred.
---
