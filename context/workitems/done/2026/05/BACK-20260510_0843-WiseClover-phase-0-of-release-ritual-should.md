---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0843-WiseClover-phase-0-of-release-ritual-should
  created: '2026-05-10T08:43:10+00:00'
  labels:
    version: v0.25.7-followup
    area: release-process
    github_issue: '73'
  updated: '2026-05-10T09:47:03+00:00'
spec:
  title: Phase 0 of release ritual should run pk-doctor + aibox doctor before bump-version (gh#73)
  state: done
  type: task
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-10T09:46:29+00:00'
  completed_at: '2026-05-10T09:47:03+00:00'
---

## Transition note (2026-05-10T09:47:03+00:00)

Implemented and pushed in commit c8fe490 (closed gh#73). scripts/maintain.sh::cmd_release_doctors invokes pk-doctor (uv run script form) + aibox doctor; combined output → dist/RELEASE-DOCTORS.md; ERRORs block, WARNs surface. NOTE-LoyalSpruce + AGENTS.md updated. GitHub issue closed manually (auto-close didn't fire on direct push but the commit body has Closes #73).
