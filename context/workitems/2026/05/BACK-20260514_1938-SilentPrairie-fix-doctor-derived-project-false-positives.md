---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260514_1938-SilentPrairie-fix-doctor-derived-project-false-positives
  created: '2026-05-14T19:38:28+00:00'
  labels:
    upstream: processkit
    source: aibox
    component: pk-doctor
    derived_projects: true
spec:
  title: Fix pk-doctor false actionable findings in derived projects
  state: backlog
  type: bug
  priority: high
  assignee: TEAMMEMBER-avery
  description: |
    Upstream processkit should absorb the aibox-local pk-doctor fixes from 2026-05-14 so derived projects do not inherit false actionable doctor findings.

    Observed in /workspace aibox after processkit v0.26.9 sync:

    1. agents_md_hygiene reported managed-block drift for AGENTS.md pk-commands because it hash-compared the local derived-project command block against the processkit repo template. That is not valid for derived projects: pk-commands intentionally contains project-local build/test/lint/fmt commands. Doctor should validate the schema/presence of pk-commands locally, but must not require byte-for-byte equality with processkit's own command block.

    2. id_vocabulary reported historical lexical shorthand collisions as WARN/actionable findings. Full IDs remain unique and authoritative, and resolving historical collisions would require a risky entity-ID rewrite migration across WorkItems/Decisions/LogEntries. The durable fix is to reserve lexical tokens globally for future allocations and report historical collisions as non-actionable inventory unless an explicit migration is commissioned.

    3. preauth_applied should stay synchronized with the generated MCP manifest when processkit removes a server such as runtime-prune, so stale preauth entries do not become derived-project WARNs.

    Expected upstream outcome: pk-doctor in derived projects should reach 0 WARN / 0 actionable after regeneration and real source fixes, without asking project agents to overwrite project-specific AGENTS pk-commands or rename historical entities.
---
