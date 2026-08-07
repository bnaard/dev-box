---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260807_0835-LucidLeaf-review-deferred-v0-30-1-dependency
  created: '2026-08-07T08:35:42+00:00'
  labels:
    release: v0.30.1
    source: release-check-state
  updated: '2026-08-07T09:54:41+00:00'
spec:
  title: Review deferred v0.30.1 dependency updates
  state: backlog
  type: task
  priority: medium
  description: 'Security review completed for the curated addon catalog. pnpm 11.20.0
    and Tau 0.3.7 were implemented and merged in PR #349 for v0.30.1. Remaining routine,
    non-security updates to assess separately: Zensical 0.0.53 and the resolvable
    Cargo.lock updates (aho-corasick, android_system_properties, clap family, libredox,
    minijinja, regex-automata). fzf 0.74.2 and uv 0.12.2 were also reviewed and deferred
    because their release notes showed bugfix/performance changes rather than security
    fixes.'
---
