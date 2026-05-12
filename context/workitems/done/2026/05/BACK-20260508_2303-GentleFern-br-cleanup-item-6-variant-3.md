---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_2303-GentleFern-br-cleanup-item-6-variant-3
  created: '2026-05-08T23:03:43+00:00'
  labels:
    track: cleanup-arch
    release: v0.25.7
    deferred_from: v0.25.6 / DEC-20260508_2240-WarmLark
    parent_epic: BR-CLEANUP-ARCH
  updated: '2026-05-10T03:27:20+00:00'
spec:
  title: "v0.25.7: BR-CLEANUP-ARCH item 6 \u2014 Variant 3 Migration emission for drifted-but-not-historical files"
  state: done
  type: task
  priority: medium
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T03:24:55+00:00'
  completed_at: '2026-05-10T03:27:20+00:00'
---

## Transition note (2026-05-10T03:27:20+00:00)

Implemented and merged in commit 1898a31 + merge 55a12cc. New cli/src/runtime_sync.rs (+342 LOC) for Variant 3 Migration emission on drifted-but-not-historical files; container.rs surfaces the warning at apply time; 7 new unit tests pass. Closes the BR-CLEANUP-ARCH cleanup epic. Two follow-ups filed: Variant 3 recommendation engine (SolidVale), --force-runtime-file flag (RapidArch).
