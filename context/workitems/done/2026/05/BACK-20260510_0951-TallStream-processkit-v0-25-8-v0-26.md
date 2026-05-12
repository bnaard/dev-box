---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0951-TallStream-processkit-v0-25-8-v0-26
  created: '2026-05-10T09:51:55+00:00'
  labels:
    version: v0.25.7-integration
    area: processkit-upgrade
  updated: '2026-05-10T11:24:00+00:00'
spec:
  title: "processkit v0.25.8 \u2192 v0.26.0 integration: sync + source-upgrade migration + new MCP vocabulary"
  state: done
  type: task
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T11:23:48+00:00'
  completed_at: '2026-05-10T11:24:00+00:00'
---

## Transition note (2026-05-10T11:24:00+00:00)

Implemented and merged in commits 4ee2d99 + 92f5d34 + merge 30d9eaf. Source-upgrade Migration MIG-20260510T100327 applied (28 changed, 0 conflicts, 48 new, 0 removed). 8 new vocabulary constants in cli/src/processkit_vocab.rs (5 RoleSlot tools, query_budget_drift, 2 route_task fields). Compliance contract v2 rewrite. 905 tests pass.
