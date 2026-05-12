---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260423_2050-FierceFinch-prevent-missing-command-registrations
  created: '2026-04-23T20:50:33+00:00'
spec:
  title: Prevent missing command registrations in derived projects
  state: accepted
  decision: 'Implement three-part fix: (1) upstream processkit v0.19.2+ ships skills with complete command directories; (2) extend aibox doctor to validate all installed skills have CLI command registration; (3) aibox sync auto-generates missing command files from SKILL.md definitions as safeguard.'
  context: pk-doctor was shipped in processkit v0.19.1 without its commands/ directory, making /pk-doctor unavailable in Claude Code until manually copied. This will affect any derived project that uses v0.19.1. The issue should not recur.
  rationale: 'Layered defense: upstream prevents the problem, detection catches it if it happens, auto-remediation makes it transparent to users. Three layers because processkit release cycles are independent of aibox, so detection + remediation protect derived projects until processkit ships a fix.'
  alternatives:
  - option: Ignore
    rationale: Let users discover missing commands (too costly, each new project wastes time)
  - option: Only upstream fix
    rationale: "Only fix in processkit (slow \u2014 derived projects stuck until new release)"
  - option: Only detection
    rationale: "Only add doctor check (reactionary \u2014 users see errors but have no fix)"
  decided_at: '2026-04-23T20:50:33+00:00'
---
