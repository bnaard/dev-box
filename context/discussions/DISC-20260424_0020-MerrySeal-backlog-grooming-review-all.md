---
apiVersion: processkit.projectious.work/v2
kind: Discussion
metadata:
  id: DISC-20260424_0020-MerrySeal-backlog-grooming-review-all
  created: '2026-04-24T00:20:00+00:00'
  updated: '2026-04-24T00:24:15+00:00'
spec:
  question: 'Backlog grooming: Review all items in backlog state for relevance post-processkit v0.19.1 and establish 90-day focus area'
  state: active
  opened_at: '2026-04-24T00:20:00+00:00'
  outcomes:
  - DEC-20260424_0020-ShinyCrow-backlog-grooming-2026-04
---

## Purpose
Systematic review of accumulated backlog items to identify:
1. Items that are no longer relevant (e.g., SoundRabbit pointing to v0.8.0)
2. Blocking dependencies that prevent progress on high-priority items
3. The minimal 3-5 item set for the next 90-day cycle

## Process
Following PROC-backlog-grooming phases:
- Phase 1: Audit (relevance check, blockers, effort)
- Phase 2: Dependency Analysis (blocking relationships)
- Phase 3: Reprioritize (confirm CRITICAL/HIGH, establish 90-day focus)
- Phase 4: Cleanup (mark obsolete items)

## Context
- processkit upgraded from v0.8.0 → v0.19.1 (major version jump)
- Completed: #53 (MCP config merging), created workitems for #54, #51
- Ready to identify next work priorities
