---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0843-AmberThorn-aibox-should-emit-v1-v2-migration
  created: '2026-05-10T08:43:26+00:00'
  labels:
    version: v0.25.7-followup
    area: migration-emission
    github_issue: '72'
  updated: '2026-05-10T09:47:06+00:00'
spec:
  title: "aibox should emit v1\u2192v2 Migration entities on cutover releases (gh#72)"
  state: done
  type: task
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T09:46:30+00:00'
  completed_at: '2026-05-10T09:47:06+00:00'
---

## Transition note (2026-05-10T09:47:06+00:00)

Implemented and merged in commit 7d70818 + merge de8f099 (closed gh#72). New cli/src/v1_v2_migration.rs ships CutoverDescriptor catalog + emit_v1_v2_migrations() (currently empty, mechanism only); wired into apply preflight in container.rs. 6 new tests. Backfill of 3 known historical cutovers tracked as BACK-EagerSea follow-up.
