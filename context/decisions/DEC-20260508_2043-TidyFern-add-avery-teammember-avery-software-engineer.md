---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_2043-TidyFern-add-avery-teammember-avery-software-engineer
  created: '2026-05-08T20:43:28+00:00'
spec:
  title: "Add Avery (TEAMMEMBER-20260508_2042-MigratedMember-avery) \u2014 software-engineer/senior \u2014 to fill team's engineering gap"
  state: accepted
  decision: Created TEAMMEMBER-20260508_2042-MigratedMember-avery as an ai-agent with default_role=ROLE-software-engineer and default_seniority=senior. Avery is the dispatch target for engineering implementation tasks; Cora remains the dispatch target for product/release-coordination tasks; Bernhard (CEO) remains the human owner.
  context: "During v0.25.6 implementation, the user instruction \"use the team for token and limit efficiency\" was misinterpreted as \"use the harness's generic Agent tool\" rather than \"dispatch via processkit TeamMembers with their resolved RuntimeBindings\". Recovery query revealed the team has only Cora (product-manager/senior, AI agent) and Bernhard (CEO/principal, human). No engineering role exists \u2014 yet the remaining v0.25.6 backlog (security hardening, e2e gap closure, status-bar rework) is engineering work. Cora's stated boundaries explicitly require escalation when scope expands beyond product/release coordination, so dispatching engineering work under her identity would knowingly violate her own role definition."
  rationale: "Adding a single TeamMember surgically (rather than running pk-team-create to bootstrap the full 8-archetype team) is the minimum-disruption fix: it leaves Cora's existing setup intact, fills the engineering gap, and keeps the team small and human-readable. Naming \"Avery\" was suggested by the AI-agent name pool. The personality emphasises code-first, small reversible steps, and refactoring discipline \u2014 fits a generalist senior IC. The role ROLE-software-engineer already existed in the project's role catalog, so no role creation was needed."
  alternatives:
  - option: Run pk-team-create to bootstrap the full 8-archetype team
    rejected_because: Heavyweight; deactivates Cora's current setup; emits a chartering DecisionRecord that supersedes Cora's; over-shoots the immediate need (engineering dispatch target). Defer to a future quarterly rebalance.
  - option: Continue under Cora's identity for engineering work
    rejected_because: Knowingly violates Cora's stated boundaries (escalate when scope expands beyond product). Skews future event-log attribution; makes a future pk-team-review noisier.
  - option: Attribute engineering work to Bernhard (the harness owner)
    rejected_because: Conflates AI execution with human authorisation; harder to distinguish 'CEO approved' from 'CEO did the typing'.
  consequences: "Future engineering tasks should dispatch via TEAMMEMBER-20260508_2042-MigratedMember-avery (set_active_interlocutor \u2192 dispatch \u2192 restore Cora). Avery has no explicit RuntimeBinding yet; runtime resolution will fall through to role+seniority defaults from the project's bindings layer (likely the same Sonnet-class model Cora resolves to). If a different binding is desired (e.g. higher capability for security-critical work), record a separate ROLE-software-engineer / seniority-binding override."
  deciders:
  - TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter
  decided_at: '2026-05-08T20:43:28+00:00'
---
