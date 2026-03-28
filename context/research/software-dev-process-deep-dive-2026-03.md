# Software Development Process Deep Dive: Primitive Coverage Analysis

**Status:** Research complete
**Date:** 2026-03-27
**Relates to:** BACK-055, DISC-001, process-ontology-primitives-2026-03.md
**Purpose:** Exhaustive mapping of the full software development lifecycle against the 17 aibox primitives to identify gaps in coverage.

---

## 1. Full Software Development Lifecycle Mapped to Primitives

The 17 primitives (from the ontology + DISC-001 additions of Discussion and Actor):

1. Work Item
2. Event / Log Entry
3. Decision Record
4. Artifact
5. Role
6. Process / Workflow
7. State Machine
8. Category / Taxonomy
9. Cross-Reference / Relation
10. Checkpoint / Gate
11. Metric / Measure
12. Schedule / Cadence
13. Scope / Container
14. Constraint
15. Context / Environment
16. Discussion
17. Actor

### 1.1 Product Discovery

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| User research / interviews | Work Item (research task) | Artifact (interview notes, recordings), Actor (interviewees), Schedule (research sprint) |
| Persona creation | Artifact (persona document) | Category (user segments), Context (market environment) |
| Competitive analysis | Artifact (competitive landscape report) | Cross-Reference (competitor-to-feature mapping) |
| Opportunity identification | Work Item (opportunity/spike) | Decision (which opportunities to pursue), Metric (market size, TAM) |
| Problem framing | Artifact (problem statement) | Discussion (stakeholder alignment), Constraint (business viability) |
| Ideation / brainstorming | Discussion | Work Item (ideas generated), Decision (which to advance) |
| Prototyping / validation | Work Item (prototype task) | Artifact (prototype), Gate (validation checkpoint), Metric (user test results) |

### 1.2 Requirements

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| User story writing | Work Item (story) | Category (type: story), Role (product owner authors) |
| Acceptance criteria | Gate (criteria attached to story) | Constraint (quality standards) |
| Technical requirements | Artifact (tech spec) | Cross-Reference (story-to-spec link), Constraint (performance, security) |
| Non-functional requirements | Constraint | Metric (SLOs, performance targets), Category (NFR type: perf, security, a11y) |
| Requirements traceability | Cross-Reference | Work Item (parent-child: epic->feature->story) |
| Backlog grooming | Process | Schedule (cadence: weekly/biweekly), Role (PO + team), Discussion |
| Story mapping | Artifact (story map) | Scope (release slices), Category (user activity) |

### 1.3 Architecture and Design

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| Architecture Decision Records | Decision Record | Artifact (the ADR document), Cross-Reference (to requirements) |
| Design documents / RFCs | Artifact (design doc) | Discussion (RFC review), Gate (design review approval), Role (architect) |
| API design | Artifact (API spec, OpenAPI) | Constraint (backward compatibility), Gate (API review) |
| Data modeling | Artifact (schema, ERD) | Decision (schema choices), Cross-Reference (to features) |
| System architecture diagrams | Artifact (diagrams) | Context (system landscape), Scope (service boundaries) |
| Spike / research | Work Item (spike) | Artifact (findings), Schedule (timebox), Decision (outcome) |
| Technical debt assessment | Work Item (tech debt item) | Category (debt type), Metric (debt severity), Cross-Reference (to affected code) |

### 1.4 Implementation

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| Branching strategy | Process (git workflow) | Constraint (branch naming, trunk-based vs. gitflow), Context (environment: repo) |
| Coding | Work Item (task in progress) | State Machine (in-progress state), Actor (developer), Artifact (code) |
| Pair programming / mob programming | Process | Role (driver, navigator), Schedule (pairing session) |
| Work-in-progress limits | Constraint (WIP limit) | Metric (current WIP count), State Machine (limits items in active states) |
| Feature flags / toggles | **GAP -- see Section 3** | -- |
| Dependency management | **GAP -- see Section 3** | -- |
| Code commits | Event (commit event) | Artifact (commit), Cross-Reference (commit-to-workitem link) |

### 1.5 Testing

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| Unit testing | Gate (tests must pass) | Artifact (test suite), Metric (coverage %), Constraint (coverage threshold) |
| Integration testing | Gate | Artifact (test reports), Process (CI pipeline step) |
| End-to-end testing | Gate | Artifact (E2E test suite), Context (test environment), Schedule (nightly runs) |
| Performance testing | Gate | Metric (latency, throughput), Constraint (SLOs), Artifact (benchmark results) |
| Security testing | Gate | Constraint (OWASP, CVE scan), Artifact (security scan report), Category (vulnerability severity) |
| Accessibility testing | Gate | Constraint (WCAG level), Artifact (a11y audit report) |
| User acceptance testing | Gate | Role (product owner/stakeholder), Process (UAT sign-off) |
| Test environments | **GAP -- partially covered, see Section 3** | -- |

### 1.6 CI/CD

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| Build pipeline | Process (automated) | Gate (build succeeds), Artifact (build output), Event (build event) |
| Test pipeline | Process (automated) | Gate (all tests pass), Metric (test duration, flakiness), Event (test results) |
| Static analysis / linting | Gate (automated) | Constraint (lint rules), Artifact (lint config) |
| Deployment pipeline | Process (automated) | Gate (deployment verification), Context (target environment), Event (deploy event) |
| Pipeline configuration | Artifact (CI config files) | Constraint (pipeline rules), Cross-Reference (pipeline-to-service) |
| Artifact registry | Scope (container for build artifacts) | Artifact (published packages, images), Category (artifact type) |

### 1.7 Release Management

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| Versioning strategy | Constraint (semver rules) | Process (version bump procedure), Decision (versioning scheme) |
| Changelog generation | Artifact (CHANGELOG) | Event (aggregated from commit/merge events), Process (changelog update) |
| Release notes | Artifact | Role (release manager), Cross-Reference (to resolved work items) |
| Release sign-off | Gate | Role (QA lead, product owner), Checklist (criteria) |
| Rollback procedures | Process (rollback workflow) | **GAP -- see Section 3** |
| Hotfix process | Process | Gate (expedited review), Constraint (severity threshold for hotfix) |
| Feature freeze | Constraint (temporal) | Schedule (freeze date), Scope (release scope) |
| Canary / staged rollout | Process | Metric (error rates), Gate (promotion criteria), Context (environment tiers) |

### 1.8 Operations

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| Monitoring setup | Metric (system metrics) | Constraint (alert thresholds), Artifact (dashboard config) |
| Alerting rules | Constraint (threshold-based) | Metric (monitored values), Process (alert-to-incident escalation) |
| Incident response | Process (incident workflow) | Work Item (incident), State Machine (incident states), Role (incident commander), Category (severity) |
| On-call rotations | Schedule | Role (on-call engineer), **GAP -- rotation-specific features, see Section 3** |
| Postmortems / retrospectives | Artifact (postmortem doc) | Decision (action items), Discussion (blameless review), Event (incident timeline) |
| SLOs / SLAs | Constraint (service level targets) | Metric (actual vs target), Schedule (reporting cadence) |
| Runbooks | Artifact (runbook document) | **GAP -- see Section 3** |
| Capacity planning | Metric (utilization forecasts) | Constraint (capacity limits), Schedule (planning cadence), Scope (per-service) |

### 1.9 Maintenance

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| Tech debt tracking | Work Item (tech debt item) | Category (debt type: code, arch, test, infra), Metric (severity/cost), Cross-Reference (to affected components) |
| Dependency updates | Work Item (update task) | **GAP -- external dependency tracking, see Section 3** |
| Security patching | Work Item (patch task) | Constraint (CVE severity SLA), Gate (security review), Schedule (patch window) |
| Deprecation management | Process (deprecation workflow) | Schedule (deprecation timeline), Artifact (deprecation notice), Cross-Reference (to dependents) |
| Performance optimization | Work Item (optimization task) | Metric (performance baselines), Constraint (SLO targets) |
| Documentation maintenance | Work Item (doc update) | Artifact (docs), Cross-Reference (to changed features) |

### 1.10 End-of-Life

| Activity | Primary Primitive | Supporting Primitives |
|---|---|---|
| Sunset planning | Process (EOL workflow) | Schedule (sunset timeline), Decision (EOL decision), Cross-Reference (to affected users/services) |
| Migration planning | Process + Work Item | Artifact (migration guide), Scope (migration project), Constraint (backward compat period) |
| Data archival | Process | Artifact (archived data), Constraint (retention requirements), Schedule (archival deadline) |
| Communication plan | Artifact (comms plan) | Schedule (notification cadence), Role (responsible communicator), Cross-Reference (to affected stakeholders) |

---

## 2. Comprehensive Primitive Coverage Matrix

Every primitive mapped to which lifecycle stages use it heavily (H), moderately (M), or lightly (L):

| Primitive | Discovery | Reqts | Arch | Impl | Test | CI/CD | Release | Ops | Maint | EOL |
|---|---|---|---|---|---|---|---|---|---|---|
| Work Item | H | H | M | H | M | L | M | H | H | M |
| Event | L | L | L | H | H | H | H | H | M | M |
| Decision | M | M | H | L | L | L | M | M | M | H |
| Artifact | H | H | H | H | H | H | H | H | M | H |
| Role | M | H | H | H | M | L | M | H | L | M |
| Process | M | H | M | H | M | H | H | H | M | H |
| State Machine | L | M | L | H | L | M | M | H | M | M |
| Category | M | H | M | M | H | M | M | H | H | L |
| Cross-Reference | M | H | H | H | M | M | H | M | H | H |
| Gate | M | M | H | L | H | H | H | M | M | M |
| Metric | M | L | L | M | H | H | M | H | H | L |
| Schedule | M | H | L | M | M | M | H | H | M | H |
| Scope | M | M | H | M | L | M | H | H | M | M |
| Constraint | L | H | H | M | H | H | H | H | H | H |
| Context | H | M | H | H | H | H | M | H | M | M |
| Discussion | H | H | H | M | L | L | M | H | L | M |
| Actor | H | M | M | H | L | L | M | H | L | M |

**Key observation:** All 17 primitives are exercised by the software development lifecycle. No primitive is redundant. But several lifecycle stages require concepts that sit uncomfortably in the current primitives -- see Section 3.

---

## 3. Gap Analysis: What is NOT Covered by the 17 Primitives

### 3.1 Environments (dev, staging, production)

**Current coverage:** Partially covered by Context/Environment (primitive 15) and Scope/Container (primitive 13).

**The gap:** Environments in software development are more than ambient context -- they are **deployable targets with specific configurations, infrastructure, and promotion rules.** A "staging environment" has:
- A concrete infrastructure definition (servers, containers, DNS)
- A current deployed version (artifact reference)
- Promotion rules (what gates must pass to promote from staging to prod)
- Access controls (who can deploy here)
- Configuration that differs from other environments (env vars, secrets, feature flags)

**Assessment:** This is best modeled as a **specialized Scope** with additional attributes: `deployed_version` (reference to Artifact), `promotion_source` (reference to another Environment/Scope), `promotion_gates` (reference to Gate[]), `config` (key-value pairs or reference to Artifact). No new primitive needed, but the Scope primitive needs an `environment` subtype with these additional fields in the `custom:` YAML map.

**Recommendation:** Define an `environment` subtype of Scope. Not a new primitive.

### 3.2 Feature Flags / Toggles

**Current coverage:** Not directly covered by any primitive.

**The gap:** Feature flags are a cross-cutting concern. A feature flag is:
- A named boolean (or multi-valued) switch
- Scoped to an environment (or set of environments)
- Linked to one or more work items (the features it controls)
- Governed by a lifecycle (created -> active -> percentage rollout -> fully on -> removed)
- Subject to constraints (max number of active flags, cleanup SLA)
- Tracked via events (flag toggled on/off, percentage changed)

**Assessment:** Feature flags sit at the intersection of Constraint (they constrain feature visibility), Context/Environment (they vary by environment), and Work Item (they have a lifecycle). They are NOT adequately modeled by any single primitive. However, creating a new primitive for them would be over-specializing the ontology for software development.

**Recommendation:** Model as a Work Item with `subtype: feature-flag` and a dedicated state machine (see Section 5). The flag's per-environment state can be tracked via a `custom:` field with environment-to-state mappings. This keeps the primitive count at 17 while allowing full lifecycle management.

### 3.3 API Contracts / Interface Definitions

**Current coverage:** Partially covered by Artifact (the spec itself) and Constraint (backward compatibility rules).

**The gap:** An API contract is:
- A versioned specification (Artifact)
- A compatibility guarantee (Constraint: no breaking changes without major version bump)
- A dependency surface (Cross-Reference: which services consume this API)
- Subject to review (Gate: API review before publish)
- Has its own lifecycle (draft -> published -> deprecated -> retired)

**Assessment:** Well covered by composing existing primitives. An API spec is an Artifact with a state machine. Compatibility rules are Constraints. Consumer tracking is Cross-References.

**Recommendation:** No new primitive needed. Use Artifact subtype `api-contract` with a dedicated state machine and mandatory Cross-References to consumers.

### 3.4 External Dependency Tracking

**Current coverage:** Poorly covered. Dependencies (npm packages, crate dependencies, third-party APIs, external services) are not naturally modeled by any primitive.

**The gap:** External dependencies have:
- A name and version (potentially pinned or ranged)
- A license (Constraint: license compatibility)
- A security posture (Events: CVE notifications, Metrics: vulnerability count)
- Update urgency (Category: severity of updates needed)
- An owner in the consuming project (Role: who is responsible for updating)
- Impact scope (Cross-Reference: which components use this dependency)
- A lifecycle (adopted -> active -> deprecated -> removed)

**Assessment:** This is a genuine modeling challenge. Dependencies are not Work Items (they are not "things to do"). They are not Artifacts (you did not produce them). They are external entities that your project depends on. The closest primitive is Cross-Reference (your project references an external entity), but Cross-References are lightweight links, not entities with their own attributes and lifecycle.

**Recommendation:** Model as a specialized Artifact with `subtype: external-dependency`. The key insight is that from your project's perspective, a dependency IS an artifact you consume -- you track its version, license, security status, and lifecycle. Alternatively, this could be a specialized Work Item if the primary interaction is "keep this updated." For aibox specifically, a `dependency` subtype of Artifact with `custom:` fields for version, license, security-status, and update-urgency is sufficient. Heavy dependency tracking (Dependabot-style) is out of scope for a context management tool.

### 3.5 Technical Debt Registry

**Current coverage:** Mostly covered by Work Item with `subtype: tech-debt`.

**The gap:** Tech debt items are work items, but they have unique attributes:
- Interest cost (how much ongoing pain does this debt cause?)
- Principal cost (how much work to fix it?)
- Affected code areas (Cross-Reference to components/files)
- Debt type: code debt, architecture debt, test debt, documentation debt, infrastructure debt
- Accrual reason: deliberate vs. accidental vs. bit rot

**Assessment:** Well covered by Work Item + Category (debt type) + Metric (interest/principal cost) + Cross-Reference (affected areas). No new primitive needed.

**Recommendation:** Define a `tech-debt` category taxonomy with types (code, arch, test, doc, infra) and custom fields for interest/principal estimates.

### 3.6 On-Call Rotations

**Current coverage:** Partially covered by Schedule (cadence) + Role (on-call engineer).

**The gap:** On-call rotations have:
- A rotation schedule (who is on-call when)
- Escalation chains (if primary doesn't respond, who is next?)
- Handoff process (end-of-shift handoff notes)
- Coverage rules (minimum coverage, backup requirements)
- Override/swap mechanism (two people swap shifts)
- Historical records (who was actually on-call when an incident happened)

**Assessment:** Schedule + Role covers the basics. But escalation chains are a Process, overrides are Events, and the rotation pattern itself is more complex than a simple recurring Schedule. The "who was on-call at time T" query requires correlating Schedules with Events.

**Recommendation:** Model as a Process (`on-call-rotation`) with an associated Schedule, Role assignments with temporal validity, and an escalation sub-process. The existing primitives cover this adequately if used together, but aibox might benefit from a process template for on-call management.

### 3.7 Incident Severity Classification

**Current coverage:** Fully covered by Category (severity taxonomy: SEV1/SEV2/SEV3/SEV4) + Constraint (SLA per severity level) + State Machine (incident lifecycle).

**Recommendation:** No gap. Define a severity Category with well-known values and associated Constraints (response time SLA per level).

### 3.8 Runbooks

**Current coverage:** Covered by Artifact (the runbook document).

**The gap:** Runbooks are a special kind of Artifact because they are:
- Executable (they describe step-by-step procedures)
- Triggered by specific conditions (linked to alerts/incidents)
- Version-critical (using the wrong version during an incident is dangerous)
- Testable (should be validated periodically)

**Assessment:** This is an Artifact with a Process embedded in its content. The "executable" nature is what distinguishes a runbook from a regular document. For aibox, runbooks are Artifacts with `subtype: runbook` that Cross-Reference the incidents/alerts that trigger them and have a Schedule for periodic validation.

**Recommendation:** Artifact subtype `runbook` with Cross-References to triggering alerts and a validation Schedule. Consider a process template for runbook creation/maintenance.

### 3.9 SLOs/SLAs

**Current coverage:** Covered by Constraint (the target) + Metric (the measured value) + Gate (compliance check).

**Assessment:** Well covered. An SLO is a Constraint with a Metric that measures compliance and a Schedule for reporting. An SLA is a Constraint with contractual enforcement (a stronger severity level). The gap is minor: the relationship between the Constraint (target) and the Metric (actual) should be formalized so that threshold breaches automatically trigger Events.

**Recommendation:** No new primitive. Formalize the Constraint-to-Metric binding so that metric threshold breaches generate Events that can trigger incident Processes.

### 3.10 Rollback Procedures

**Current coverage:** Covered by Process (rollback workflow).

**The gap:** Rollback is a Process, but it has a unique trigger: it is initiated by a failed deployment or degraded production metrics. The rollback target (which version to roll back to) requires knowing the deployment history (Event log) and the environment's previous state.

**Assessment:** Process + Event (deployment history) + Context/Environment (current and target states) covers this. The key requirement is that deployment Events include enough information to reconstruct the previous state.

**Recommendation:** No new primitive. Ensure deployment Events capture the before/after artifact versions per environment. Add a rollback process template.

### 3.11 A/B Testing / Experiments

**Current coverage:** Partially covered. A/B tests are:
- Work Items (the experiment to run)
- With Metrics (conversion rates, statistical significance)
- Subject to Constraints (sample size requirements, duration minimums)
- Producing Decisions (which variant wins)
- Implemented via feature flags (see 3.2)

**The gap:** The statistical rigor aspect -- hypothesis, control group, sample size calculation, significance testing -- is not natively modeled. This is the same gap identified in the sector analysis for scientific research ("experiment as work item with hypothesis-result structure").

**Assessment:** For software A/B tests, a Work Item with `subtype: experiment` and custom fields for hypothesis, variants, sample-size, significance-threshold, and result is sufficient. The statistical analysis itself is an Artifact.

**Recommendation:** Work Item subtype `experiment` with custom fields. Process template for experiment lifecycle.

---

## 4. Comparison Against Current aibox Process Templates

### 4.1 What Exists

| Template | Steps | Roles | Gates |
|---|---|---|---|
| `feature-development.md` | 6 steps: identify -> branch -> implement -> PR -> review feedback -> merge | developer | Tests pass, reviewed, merged |
| `bug-fix.md` | 6 steps: reproduce -> failing test -> fix -> verify -> PR -> merge | developer | Reproducing test, regression test, reviewed |
| `code-review.md` | 5 steps: open PR -> check correctness -> comment/approve -> address feedback -> merge | developer, reviewer | CI pass, test coverage, no unrelated changes, docs updated |
| `release.md` | 5 steps: test+lint -> changelog -> version bump -> tag -> build+publish | developer | Tests pass, changelog updated, tag created |

### 4.2 Missing Micro-Processes

The following processes are common in software development but have NO template in aibox:

**High priority (needed for most software projects):**

| Missing Process | Description | Key Steps | Why Important |
|---|---|---|---|
| **incident-response** | Handling production incidents | detect -> triage -> investigate -> mitigate -> resolve -> postmortem | Ops-critical. No project operates without incidents. |
| **technical-design** | Creating and reviewing design documents / RFCs | draft -> circulate -> discuss -> revise -> approve or reject | Prevents architectural drift. The gap between "identify feature" and "implement" is huge. |
| **spike-research** | Time-boxed investigation of unknowns | define question -> timebox -> research -> document findings -> decide | Without this, spikes have no structure and drag on indefinitely. |
| **dependency-update** | Keeping dependencies current and secure | scan -> triage -> update -> test -> release | Security-critical. Most projects neglect this until a CVE forces action. |
| **hotfix** | Emergency fix for production issues | identify -> branch from release -> fix -> expedited review -> deploy -> backport | Different from regular bug-fix: urgency changes the gate requirements. |

**Medium priority (needed for mature projects):**

| Missing Process | Description | Key Steps | Why Important |
|---|---|---|---|
| **retrospective** | Team reflection and improvement | gather data -> generate insights -> decide actions -> track follow-through | Continuous improvement requires a structured retrospective process. |
| **on-call-handoff** | Transferring on-call responsibility | review active incidents -> brief incoming -> update rotation -> confirm | Prevents incidents from falling through the cracks during shift changes. |
| **deprecation** | Removing old features/APIs | announce -> migration period -> warn on use -> disable -> remove | Without this, deprecated features linger and accumulate tech debt. |
| **postmortem** | Blameless analysis of incidents | timeline reconstruction -> root cause analysis -> action items -> publish | Distinct from retrospective: focused on a specific incident, producing a permanent artifact. |
| **security-review** | Security assessment of changes | threat model -> code review for security -> dependency audit -> pen test -> sign-off | Security gates need their own process beyond "code review." |
| **backlog-grooming** | Regular backlog refinement | review new items -> estimate -> prioritize -> split large items -> remove stale items | Without this, backlogs rot: items become stale, priorities drift, estimates are missing. |

**Lower priority (needed for larger teams or specific contexts):**

| Missing Process | Description | Key Steps | Why Important |
|---|---|---|---|
| **onboarding** | New team member ramp-up | access provisioning -> codebase walkthrough -> first task assignment -> mentoring -> independence | Reduces time-to-productivity for new team members. |
| **capacity-planning** | Forecasting and allocating team capacity | measure velocity -> forecast demand -> identify bottlenecks -> adjust staffing -> communicate | Prevents overcommitment and burnout. |
| **architecture-review** | Periodic review of system architecture | assess current state -> identify drift from target -> propose changes -> decide -> schedule work | Prevents architecture erosion over time. |
| **experiment** | Running A/B tests | hypothesis -> design -> implement variants -> run -> analyze -> decide | Needed for data-driven product development. |
| **rollback** | Reverting a bad deployment | detect regression -> decide to rollback -> execute rollback -> verify -> investigate root cause | Critical safety net that should be well-rehearsed. |
| **end-of-life** | Sunsetting a product/service/feature | decide -> notify users -> migration support -> disable -> archive -> clean up | Prevents zombie services and indefinite support obligations. |

---

## 5. State Machine Definitions

### 5.1 Feature Lifecycle

```
                                    +---> Deferred (parked for later)
                                    |          |
                                    |          v
Idea --> Proposed --> Accepted --> Planned --> In Design --> In Development --> In Review --> In Testing --> Ready for Release --> Released --> Monitoring
                |                                 |              |                  |             |                                         |
                +---> Rejected                    |              +---> Blocked      +---> Failed  +---> Failed                              +---> Needs Rework
                                                  |                     |                  |             |                                         |
                                                  +---> Cancelled       +---> (unblocked)  +---> back    +---> back to In Testing                  +---> back to In Dev
```

**Simplified state list:**

| State | Entry Condition | Exit Condition |
|---|---|---|
| `idea` | Someone proposes it | Product owner evaluates |
| `proposed` | Written up with enough detail | Accepted or rejected at grooming |
| `accepted` | Product owner approves | Planned for a sprint/cycle |
| `planned` | Scheduled in a sprint/cycle | Design work begins |
| `in-design` | Design doc / ADR in progress | Design approved |
| `in-development` | Code work begins | PR opened |
| `in-review` | PR opened, awaiting review | Review approved |
| `in-testing` | Merged to staging, QA begins | All tests pass |
| `ready-for-release` | Passed all gates | Deployed to production |
| `released` | In production | Monitoring period ends |
| `done` | Stable in production | Terminal |
| `deferred` | Parked, may revisit | Re-activated to proposed |
| `rejected` | Will not do | Terminal |
| `cancelled` | No longer relevant | Terminal |
| `blocked` | Waiting on external dependency | Dependency resolved |

### 5.2 Bug Lifecycle

```
Reported --> Triaged --> Confirmed --> In Progress --> In Review --> In Testing --> Ready for Release --> Released --> Verified Fixed
    |           |           |              |               |             |
    +---> Duplicate   +---> Won't Fix     +---> Blocked   +---> Failed  +---> Regression Found
    +---> Invalid     +---> Cannot Reproduce                                      |
                                                                                  +---> back to In Progress
```

| State | Entry Condition | Exit Condition |
|---|---|---|
| `reported` | Bug submitted | Triaged by team |
| `triaged` | Severity + priority assigned | Confirmed reproducible or closed |
| `confirmed` | Reproducible, root cause identified | Work begins |
| `in-progress` | Developer working on fix | PR opened |
| `in-review` | PR opened | Review approved |
| `in-testing` | Fix merged, QA verifying | Verified or regression found |
| `ready-for-release` | Fix verified in staging | Deployed to production |
| `released` | Fix in production | Reporter confirms resolution |
| `verified-fixed` | Confirmed fixed in production | Terminal |
| `duplicate` | Same as existing bug | Terminal (linked to original) |
| `invalid` | Not a bug / user error | Terminal |
| `wont-fix` | Accepted risk / too costly | Terminal |
| `cannot-reproduce` | Cannot reproduce reliably | Terminal (may reopen) |
| `blocked` | Waiting on external factor | Unblocked |

### 5.3 Incident Lifecycle

```
Detected --> Acknowledged --> Investigating --> Mitigating --> Resolved --> Postmortem --> Closed
                                  |                |
                                  +---> Escalated  +---> Monitoring (may re-escalate)
```

| State | Entry Condition | Exit Condition | SLA (example) |
|---|---|---|---|
| `detected` | Alert fires or user reports | On-call acknowledges | SEV1: 5 min |
| `acknowledged` | On-call responds | Investigation begins | SEV1: 15 min |
| `investigating` | Root cause analysis in progress | Mitigation identified | SEV1: 1 hour |
| `escalated` | Needs additional expertise/authority | Escalation responded to | -- |
| `mitigating` | Applying fix or workaround | Service restored | SEV1: 4 hours |
| `monitoring` | Fix applied, watching for recurrence | Stable for monitoring period | 24 hours |
| `resolved` | Service confirmed stable | Postmortem scheduled | 48 hours |
| `postmortem` | Postmortem in progress | Postmortem published | 5 business days |
| `closed` | Action items tracked, lessons learned | Terminal | -- |

**Severity classification (Category):**

| Severity | Description | Response SLA | Resolution SLA |
|---|---|---|---|
| SEV1 / Critical | Service completely down, data loss, security breach | 5 min | 4 hours |
| SEV2 / Major | Service severely degraded, major feature unavailable | 15 min | 8 hours |
| SEV3 / Minor | Service partially degraded, workaround available | 1 hour | 48 hours |
| SEV4 / Low | Cosmetic issue, minor inconvenience | 4 hours | 1 week |

### 5.4 Release Lifecycle

```
Planning --> Development --> Feature Freeze --> Stabilization --> Release Candidate --> Staging Verification --> Production Deploy --> Monitoring --> Stable
                                                     |                   |                     |                       |
                                                     +---> Bug found     +---> Failed          +---> Failed            +---> Rollback
                                                            |                   |                     |                       |
                                                            +---> fix           +---> back to RC      +---> back to Stab.    +---> Hotfix cycle
```

| State | Entry Condition | Exit Condition |
|---|---|---|
| `planning` | Release scope defined | All planned features accepted |
| `development` | Sprint(s) in progress | Feature freeze date reached |
| `feature-freeze` | No new features accepted | All features code-complete |
| `stabilization` | Bug fixes and polish only | No SEV1/SEV2 bugs remaining |
| `release-candidate` | RC build created | All gates pass |
| `staging-verification` | RC deployed to staging | QA sign-off |
| `production-deploy` | Deploying to production | Deployment complete |
| `monitoring` | Deployed, watching metrics | Monitoring period passed with no issues |
| `stable` | Release confirmed good | Terminal |
| `rolled-back` | Critical issue found post-deploy | Hotfix cycle begins |

### 5.5 Technical Debt Item Lifecycle

```
Identified --> Assessed --> Prioritized --> Scheduled --> In Progress --> In Review --> Resolved --> Validated
                  |              |                             |
                  +---> Accepted (live with it)                +---> Blocked
                  +---> Deferred (revisit later)
```

| State | Entry Condition | Exit Condition |
|---|---|---|
| `identified` | Debt recognized and documented | Impact assessed |
| `assessed` | Interest + principal costs estimated | Prioritization decision |
| `prioritized` | Ranked against other work | Scheduled in a sprint/cycle |
| `accepted` | Decision to live with the debt | Terminal (may re-assess later) |
| `deferred` | Not now, but track it | Re-prioritized |
| `scheduled` | Planned in a sprint/cycle | Work begins |
| `in-progress` | Refactoring/improvement in progress | PR opened |
| `in-review` | Code review | Approved |
| `resolved` | Merged | Validated in production |
| `validated` | Confirmed improvement (metrics improved) | Terminal |

### 5.6 PR / Code Review Lifecycle

```
Draft --> Open --> Changes Requested --> Updated --> Approved --> Merged
           |            |                               |
           +---> Closed (abandoned)                     +---> Merge Conflicts --> Resolved --> Approved
```

| State | Entry Condition | Exit Condition |
|---|---|---|
| `draft` | PR created as draft (WIP) | Author marks as ready |
| `open` | Ready for review | Reviewer acts |
| `changes-requested` | Reviewer requests changes | Author pushes updates |
| `updated` | Author addressed feedback | Re-review |
| `approved` | Minimum approvals met, CI green | Author merges |
| `merge-conflicts` | Cannot merge due to conflicts | Author resolves conflicts |
| `merged` | PR merged to target branch | Terminal |
| `closed` | PR abandoned or superseded | Terminal |

**Gates in code review:**

| Gate | Type | Criteria |
|---|---|---|
| CI pipeline | Automated | All checks pass (build, test, lint) |
| Minimum approvals | Policy | >= 1 approval (configurable) |
| No unresolved threads | Manual | All review comments addressed |
| Branch up-to-date | Automated | No merge conflicts, rebased on target |
| Coverage threshold | Automated | No decrease in test coverage |
| Security scan | Automated | No new vulnerabilities introduced |

---

## 6. Summary of Findings

### 6.1 Primitive Coverage Verdict

The 17 primitives provide **excellent coverage** of the full software development lifecycle. No stage of the lifecycle falls completely outside the ontology. The primitives are general enough to model every software development activity through composition.

### 6.2 Gaps That Need Attention (but NOT new primitives)

| Gap | Recommended Model | Priority |
|---|---|---|
| Environments (dev/staging/prod) | Scope subtype `environment` with deployment state | High |
| Feature flags | Work Item subtype `feature-flag` with per-environment state | Medium |
| External dependencies | Artifact subtype `external-dependency` with version/license/security fields | Medium |
| Deployment history | Event type `deployment` with before/after artifact versions | High |
| Escalation chains | Process subtype with ordered Role references | Low |

### 6.3 Missing Process Templates (prioritized)

**Must-have for v1:**
1. `incident-response.md` -- every software project needs this
2. `technical-design.md` -- bridges requirements and implementation
3. `spike-research.md` -- structures exploration work
4. `hotfix.md` -- emergency variant of bug-fix with different gates

**Should-have for v1.x:**
5. `dependency-update.md`
6. `retrospective.md`
7. `postmortem.md`
8. `backlog-grooming.md`
9. `deprecation.md`
10. `security-review.md`

**Nice-to-have:**
11. `on-call-handoff.md`
12. `rollback.md`
13. `experiment.md`
14. `onboarding.md`
15. `architecture-review.md`
16. `end-of-life.md`
17. `capacity-planning.md`

### 6.4 Key Architectural Insight

The primitives work because they are **composable**. Every real-world software development activity is modeled as a COMPOSITION of primitives, not as a single primitive. For example:

- **A deployment** = Process (the pipeline) + Event (the deployment event) + Gate (health checks) + Context (target environment) + Artifact (the build being deployed)
- **An incident** = Work Item (the incident) + State Machine (incident lifecycle) + Category (severity) + Constraint (SLA) + Process (response procedure) + Event (timeline) + Artifact (postmortem) + Decision (action items)
- **A code review** = Process (review workflow) + Gate (approval criteria) + Role (author + reviewer) + Artifact (the PR/diff) + Event (comments, approvals) + Constraint (minimum approvals)

This composability is the ontology's strength. aibox should lean into it by making composition explicit and easy, rather than trying to create specialized primitives for every software development concept.
