---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260807_0835-LucidLeaf-review-deferred-v0-30-1-dependency
  created: '2026-08-07T08:35:42+00:00'
  labels:
    release: v0.30.1
    source: release-check-state
spec:
  title: Review deferred v0.30.1 dependency updates
  state: backlog
  type: task
  priority: medium
  description: 'Release-state review identified deferred updates: Zensical 0.0.53,
    pnpm 11.20.0, Tau 0.3.7, and the resolvable Cargo.lock updates (aho-corasick,
    android_system_properties, clap family, libredox, minijinja, regex-automata).
    Review upstream compatibility and security implications, decide which belong in
    a follow-up release, then implement and validate.'
---
