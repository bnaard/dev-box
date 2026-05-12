---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_2136-WildPanda-expand-team-to-5-ai-agents
  created: '2026-05-08T21:36:01+00:00'
spec:
  title: "Expand team to 5 AI agents \u2014 Robin, Jordan, Sage join Avery + Cora"
  state: accepted
  decision: 'Added three new AI-agent TeamMembers to fill role gaps surfaced during v0.25.6 implementation: Robin (TEAMMEMBER-20260508_2135-MigratedMember-robin, ROLE-20260422_0001-MigratedRole-software-engineer/junior, Haiku-tier work), Jordan (TEAMMEMBER-20260508_2135-MigratedMember-jordan, ROLE-20260422_0001-MigratedRole-technical-writer/senior, Sonnet-tier work), Sage (TEAMMEMBER-20260508_2135-MigratedMember-sage, ROLE-20260422_0001-MigratedRole-cto/principal, Opus-tier work). Combined with existing Cora (PM/senior), Avery (SE/senior), and human owner Bernhard, the team is now 5 AI agents + 1 human across PM, engineering, junior engineering, technical writing, and architecture/strategy.'
  context: "Session retrospective on v0.25.6 implementation revealed three role gaps: (1) ~50% of HonestAnt and KeenBison agent runtime was mechanical work (vendor doc lookups, SHA pin research, test-fixture boilerplate) \u2014 being charged Sonnet rates for work a Haiku-bound junior could do; (2) docs work (security.md, AGENTS.md updates, future release notes) was being done by software-engineer Avery rather than a tech-writer-shaped agent; (3) cross-cutting design work (DEC-SilentAsh, the v0.25.6 plan, BR-CLEANUP-ARCH variant analysis) currently rests entirely on the human CEO with no Opus-bound senior architect to collaborate. Adding all three pre-emptively (rather than one at a time) avoids the dispatch-attribution churn observed earlier this session when \"use the team\" was misread as \"use generic Claude Code agents\"."
  rationale: "Pulling names from the AI-agent name pool (Robin, Jordan, Sage) keeps the team identifiable as personas rather than archetype IDs. Personalities are role-shaped, not generic: Robin is precise+follows-instructions+asks-before-deviating; Jordan is editorial+audience-aware+voice-consistent; Sage is options-with-recommendation+rationale-first+hands-off-implementation. Boundaries explicitly delineate dispatch direction: Robin escalates to Avery; Jordan escalates technical to Avery and design to Sage; Sage hands implementation to Avery rather than micromanaging. Seniorities chosen to map to expected runtime tiers (junior \u2192 light/Haiku, senior \u2192 medium/Sonnet, principal \u2192 heavy/Opus) under processkit's tier resolution."
  alternatives:
  - option: Add only the junior engineer (highest leverage)
    rejected_because: User chose to add all three pre-emptively to avoid churn. Adding incrementally would force another team-expansion DEC every time a new role need surfaced.
  - option: Run pk-team-create to bootstrap the full 8-archetype team
    rejected_because: Heavyweight; would deactivate the existing Cora+Avery setup; emits a chartering DecisionRecord that supersedes prior team decisions; over-shoots the actual need (5 distinct people, not 8 archetype tiers).
  - option: Use ROLE-20260422_0001-MigratedRole-enterprise-architect or ROLE-20260422_0001-MigratedRole-solutions-architect instead of ROLE-20260422_0001-MigratedRole-cto for Sage
    rejected_because: ROLE-20260422_0001-MigratedRole-cto best matches 'design-heavy planning across the project' (the work currently on the CEO). enterprise-architect implies org-wide standards (too broad); solutions-architect implies customer-facing solutions (wrong domain for an internal tool project).
  consequences: "Going forward: dispatch mechanical/lookups/boilerplate to Robin (Haiku-tier \u2014 saves on subscription budget); dispatch docs/release-notes/AGENTS.md to Jordan; dispatch design/architecture/cross-cutting decisions to Sage; dispatch engineering implementation to Avery; dispatch coordination/PM to Cora. Set the active interlocutor to whichever member matches the immediate task. Future briefings should surface \"active interlocutor\" + the team roster of 5. Note: Avery has been treated as cloneable (parallel agent dispatches under a single identity) \u2014 see DEC-SilentFern follow-up issue and processkit issue #20 (TBD) on the role-vs-person ambiguity that will affect how Robin/Jordan/Sage parallelism should work."
  deciders:
  - TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter
  decided_at: '2026-05-08T21:36:01+00:00'
---
