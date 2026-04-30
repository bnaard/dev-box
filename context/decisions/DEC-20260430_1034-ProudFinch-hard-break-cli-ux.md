---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260430_1034-ProudFinch-hard-break-cli-ux
  created: '2026-04-30T10:34:27+00:00'
spec:
  title: Hard-break CLI UX redesign without legacy aliases
  state: accepted
  decision: Implement the clean-sheet aibox CLI grammar as a hard breaking change
    with no legacy command aliases or backward-compatibility shim for the old command
    taxonomy.
  context: After accepting the verb/resource CLI redesign, the owner clarified that
    they are currently the only user and can handle a full interface break directly.
  rationale: Removing compatibility constraints simplifies implementation, avoids
    a confusing dual interface, and makes the new command contract easier to validate
    and document.
  alternatives:
  - option: Keep hidden aliases for one release
    status: rejected
    reason: Unnecessary for a single-user pre-broad-adoption project and would preserve
      ambiguity in tests/help/diagnostics.
  - option: Deprecate old commands gradually
    status: rejected
    reason: Adds migration overhead without meaningful user benefit at current adoption
      level.
  consequences: All docs, tests, generated guidance, and diagnostics should move directly
    to the new command names. Old commands should fail normally through Clap rather
    than emitting compatibility hints, except where historical context files remain
    archival.
  decided_at: '2026-04-30T10:34:27+00:00'
---
