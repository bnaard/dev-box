---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1516-BrightStream-stale-state-cleanup-architecture-foundational
  created: '2026-05-08T15:16:52+00:00'
  labels:
    track: cleanup-arch
    release: v0.25.6
    blocks_others: true
  updated: '2026-05-08T20:44:44+00:00'
spec:
  title: 'v0.25.6: Stale-state cleanup architecture (foundational)'
  state: done
  type: epic
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-08T19:56:14+00:00'
  completed_at: '2026-05-08T20:44:44+00:00'
---

## Transition note (2026-05-08T19:56:14+00:00)

Items 1-2 already landed (commit e0ee7bc). Items 3-5 implementation in progress this session via subagent. Item 6 deferred.


## Transition note (2026-05-08T20:44:38+00:00)

Items 1-2 already shipped (commit e0ee7bc). Items 3-5 implementation complete this session: [apply].purge_disabled_harness_state toml key, addon purge_template across 9 yamls, harness state cleanup with Migration emission, tmux-powerkit cache + plugin walker. Item 6 (Variant 3 Migration emission) deferred. 13 new unit tests; 926 green.


## Transition note (2026-05-08T20:44:44+00:00)

Accepted as done. Item 6 (Variant 3) tracked separately for later.
