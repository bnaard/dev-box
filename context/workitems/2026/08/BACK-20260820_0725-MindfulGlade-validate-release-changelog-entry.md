---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260820_0725-MindfulGlade-validate-release-changelog-entry
  created: '2026-08-20T07:25:24+00:00'
spec:
  title: Validate public changelog entry during release
  state: backlog
  type: task
  priority: medium
  description: Add a release/docs validation gate that requires docs-site/content/changelog
    to contain the exact release version and verifies it renders as the newest Change
    Log entry. This prevents a fully published release from missing its public documentation
    timeline entry.
---
