---
name: backlog-grooming
title: Backlog Grooming Process
description: Systematic review, audit, and reprioritization of work items
---

# Backlog Grooming Process

## Purpose

Review accumulated backlog items for relevance, dependencies, and alignment with current project state. Ensure priority items are unblocked and outdated items are marked for removal or archival.

## Scope

- Review all items in `backlog` state
- Flag items referencing outdated processkit versions or superseded architecture
- Identify blocking dependencies
- Propose 90-day focus area
- Document decisions about priority shifts

## Process

### Phase 1: Audit (1-2 hours)

**Participant:** PM Agent + Junior Developer Agent + Owner (async possible)

For each item, document:
1. **Relevance check**
   - Is the problem still valid?
   - Has it been solved by recent work?
   - Does it reference outdated versions/assumptions?
   
2. **Blockers**
   - What must be done first?
   - Are there upstream dependencies (processkit, external)?
   
3. **Effort estimate** (if relevant)
   - Quick guess at scope
   - Known unknowns?

4. **Triage decision**
   - ✅ **Active**: ready to implement
   - ⏸️ **Defer**: still relevant but lower priority
   - ❌ **Obsolete**: no longer needed; mark for removal
   - ❓ **Discuss**: needs owner decision

### Phase 2: Dependency Analysis (30 min)

Map blocking relationships:
- What high-priority items are blocked?
- What's the minimum set to unblock them?
- Are there circular dependencies?

Output: `blocks/blocked_by` links in WorkItems

### Phase 3: Reprioritize (30 min)

1. **Confirm CRITICAL/HIGH items** are correctly marked
2. **Identify 90-day focus area**
   - Max 3-5 items from backlog
   - Rest defer until next cycle
3. **Document decision** as DecisionRecord

### Phase 4: Cleanup (30 min)

- Delete or archive obsolete items (mark with note)
- Update descriptions for items with changed context
- Verify priority/blocking relationships in system

## Output

- [ ] **Audit Log**: spreadsheet/document listing all items + triage decisions
- [ ] **Blocking Map**: WorkItem links showing dependencies
- [ ] **DecisionRecord**: "Backlog reprioritization after processkit v0.19.1 upgrade"
- [ ] **Next Workplan**: 3-5 items selected for 90-day focus

## Trigger

- New major processkit version released
- Quarterly planning cycle
- When backlog exceeds 50 items
- Ad-hoc owner request

## Success Criteria

- [ ] All items evaluated for relevance
- [ ] Blockers identified and documented
- [ ] Top 5 priority items clear
- [ ] Decision on next 90-day work agreed
- [ ] Obsolete items marked for removal
