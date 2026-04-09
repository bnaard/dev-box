# Process Ontology: Universal Primitives Across All Domains

**Status:** Research complete
**Date:** 2026-03-26
**Relates to:** BACK-055 (process templates), DEC-016 (composable process packages), process-frameworks-research.md
**Author:** Researcher agent

---

## 1. Introduction and Methodology

This document identifies the **universal process primitives** -- the fundamental building blocks that ALL process frameworks share, regardless of domain. These are the atoms from which every process in every industry is composed.

**Why this matters for aibox:** aibox provides base processes for managing projects. Higher-level applications (multi-agent orchestrators, domain-specific tools) compose these primitives into specific frameworks. If aibox gets the primitives right, it can support any domain -- software, manufacturing, healthcare, legal, supply chain, research, and beyond.

**Methodology:**
1. Analyzed four IT/project management frameworks from our existing research (SAFe 6.0, PMBOK 7th, CMMI v3.0, IPMA ICB4)
2. Cross-referenced against manufacturing (Toyota Production System, Lean, Six Sigma), healthcare (clinical pathways), legal (case management), supply chain (order management), knowledge management (Zettelkasten, PARA, GTD), issue trackers (GitHub Issues, Linear, Jira, Asana), Shape Up (Basecamp), Kanban, ITIL v4, BPMN 2.0, systems thinking, and quality management (ISO 9001, TQM)
3. Extracted common structural patterns that appear in 3+ unrelated domains
4. Validated against the starting hypothesis of 13 primitives -- refined to 15

**Key finding:** The 13 hypothesized primitives are largely correct but needed refinement. Two additional primitives emerged (Constraint and Context/Environment), and several were sharpened based on cross-domain analysis. The primitives form a coherent ontology with well-defined relationships.

---

## 2. The 15 Universal Process Primitives

### 2.1 Primitive: Work Item

**Also known as:**
- SAFe: Story, Feature, Capability, Epic
- PMBOK: Work Package, Activity, Task
- Kanban: Card
- Shape Up: Scope, Pitch
- ITIL: Incident, Problem, Change Request, Service Request
- Healthcare: Order (lab order, medication order), Referral, Care Task
- Legal: Case, Matter, Action Item, Filing
- Manufacturing: Work Order, Job, Production Order, Kanban Card
- Supply Chain: Purchase Order, Shipment, Fulfillment Request
- GTD: Next Action, Project
- Zettelkasten: (not applicable -- Zettelkasten has no work items, only notes)
- BPMN: Task (User Task, Service Task, Manual Task)
- Six Sigma: CTQ (Critical to Quality) requirement, Improvement Action
- GitHub/Jira/Linear: Issue, Ticket, Story, Bug, Task

**Definition:** A discrete unit of work that must be performed, tracked, and completed. The most fundamental primitive -- every process framework has some notion of "a thing to be done."

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier (BACK-001, ORD-4523, CASE-2024-1187) |
| `title` | string | Human-readable short description |
| `description` | text | Detailed explanation of what needs to be done |
| `state` | enum | Current lifecycle state (see State Machine primitive) |
| `owner` | reference(Role) | Who is responsible for completion |
| `created_at` | timestamp | When the item was created |
| `updated_at` | timestamp | When it was last modified |
| `priority` | reference(Category) | How urgent/important this is |
| `parent` | reference(Work Item) | Optional hierarchical parent |
| `references` | reference(Cross-reference)[] | Links to other items |

**State machine (universal lifecycle):**
```
Draft/New --> Ready/Accepted --> In Progress --> In Review --> Done/Closed
                                    |                           |
                                    +--> Blocked                |
                                    +--> Cancelled/Withdrawn ---+
```

Every framework uses a subset or elaboration of this pattern. SAFe adds "Funnel -> Reviewing -> Analyzing" for epics. ITIL adds "Awaiting User Info" and "Known Error." Healthcare adds "Ordered -> Scheduled -> Performed -> Resulted." But the core shape is always: not-started -> active -> completed, with escape states for blocked/cancelled.

**Relationships:**
- **parent-child** with other Work Items (epic -> feature -> story -> task)
- **blocks/blocked-by** other Work Items
- **assigned-to** a Role
- **belongs-to** a Scope/Container
- **governed-by** a Process/Workflow
- **measured-by** Metrics
- **produces** Artifacts
- **validated-at** Checkpoints/Gates

**Cross-framework examples:**

| Framework | Name | Hierarchy | Key Difference |
|---|---|---|---|
| SAFe 6.0 | Epic/Feature/Story | 4 levels (Portfolio->Solution->ART->Team) | Fixed hierarchy depth, PI-bounded |
| PMBOK 7th | Work Package/Activity | WBS decomposition, variable depth | Scope-driven, can be predictive or adaptive |
| Kanban | Card | Flat (or minimal grouping) | No prescribed hierarchy, flow-focused |
| ITIL v4 | Incident/Change/Problem | Type-driven, not hierarchy-driven | Classified by nature (break-fix vs. planned change) |
| Manufacturing (TPS) | Kanban Card / Work Order | Station -> Line -> Plant | Physical flow through workstations |
| Healthcare | Clinical Order | Order Set -> Individual Order | Protocol-driven, safety-critical |
| Legal | Case / Matter | Matter -> Tasks -> Filings | Jurisdiction and statute-driven |
| GTD | Next Action / Project | Project -> Next Actions | Context-tagged, energy-level-tagged |
| Shape Up | Pitch / Scope | Pitch -> Scopes -> Tasks | Fixed-time (6-week cycle), variable scope |

**aibox implication:** The Work Item is the core of `BACKLOG.md`. aibox already uses BACK-NNN IDs with state tracking. The universal model validates this. aibox should support: (1) hierarchical parent-child relationships, (2) a configurable state machine per work item type, (3) cross-references between items. The current flat backlog table is sufficient for small projects but should allow optional nesting.

---

### 2.2 Primitive: Log Entry / Event

**Also known as:**
- SAFe: PI System Demo outcome, I&A finding
- PMBOK: Lessons learned, Issue log entry, Risk register entry
- ITIL: Incident record, Audit trail entry, Event record
- Healthcare: Clinical note, Progress note, Audit log entry, Vital sign reading
- Legal: Case note, Docket entry, Court filing record, Billing entry
- Manufacturing: Production log entry, Quality inspection record, Defect report
- Supply Chain: Tracking event (shipped, in-transit, delivered), Customs entry
- GTD: Weekly review note, Capture inbox item
- BPMN: Event (Start, Intermediate, End), Signal, Message
- Systems Thinking: Stock measurement at a point in time
- Kanban: Cumulative flow diagram data point
- Six Sigma: Measurement data point, Control chart reading
- ISO 9001: Audit finding, Nonconformity record

**Definition:** An immutable record of something that happened at a specific point in time. The fundamental unit of history and accountability. Events are append-only -- you never modify a past event, you create a new event that supersedes or corrects it.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `timestamp` | timestamp | When the event occurred |
| `type` | enum | Classification of the event |
| `actor` | reference(Role) | Who/what caused the event |
| `subject` | reference(any) | What the event is about |
| `description` | text | Human-readable description |
| `data` | structured | Event-specific payload |
| `source` | string | System or process that generated the event |

**State machine:** Events are immutable -- they have no lifecycle. They exist or they don't. (Though they can be soft-deleted or retracted by a subsequent event.)

**Relationships:**
- **about** a Work Item, Artifact, or any other primitive
- **caused-by** another Event (event chains / causal sequences)
- **triggers** a Process/Workflow or state transition
- **aggregated-into** Metrics
- **recorded-within** a Scope/Container

**Cross-framework examples:**

| Framework | Name | Key Characteristic |
|---|---|---|
| SAFe 6.0 | Flow metric data point, PI event | Aggregated into flow velocity, time, efficiency |
| PMBOK 7th | Issue log entry, Lessons learned | Feeds measurement performance domain |
| ITIL v4 | Event record, Incident record | Triggers incident/problem management workflows |
| Manufacturing | Production log, SPC data point | Feeds control charts and process capability |
| Healthcare | Clinical observation, Vital sign | Time-series, legally mandated retention |
| Legal | Docket entry, Filing timestamp | Court-mandated, tamper-proof, chronological |
| Supply Chain | Tracking event (EDI 214/990) | Enables end-to-end visibility |

**aibox implication:** aibox already has an event log design (BACK-050). This primitive validates the event-sourced architecture. Key insight: events should be typed (state-change, comment, measurement, system-event) and always reference the subject they describe. The materializer pattern (events -> aggregated metrics) is universal across all domains.

---

### 2.3 Primitive: Decision Record

**Also known as:**
- SAFe: Lean Business Case decision, PI Planning commitment
- PMBOK: Change request resolution, Lessons learned decision
- Software: Architecture Decision Record (ADR)
- ITIL: Change Advisory Board (CAB) decision, Problem resolution
- Healthcare: Treatment decision, Clinical protocol selection, Informed consent
- Legal: Ruling, Verdict, Settlement agreement, Legal opinion
- Manufacturing: Engineering Change Notice (ECN), Process change approval
- Supply Chain: Vendor selection decision, Route optimization decision
- Government: Policy decision, Regulatory ruling
- GTD: "Someday/Maybe" triage decision
- ISO 9001: Management review output, Corrective action decision

**Definition:** A record of a choice made, including the context, options considered, rationale, and consequences. Decisions are first-class artifacts because they explain *why* things are the way they are -- without decision records, institutional knowledge is lost.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier (DEC-001, ECN-4523, ADR-0012) |
| `title` | string | What was decided |
| `status` | enum | Proposed, Accepted, Deprecated, Superseded |
| `date` | date | When the decision was made |
| `context` | text | Situation requiring a decision |
| `decision` | text | What was decided and why |
| `alternatives` | text[] | Options considered and why rejected |
| `consequences` | text | Implications (positive, negative, risks) |
| `decision_makers` | reference(Role)[] | Who made the decision |
| `superseded_by` | reference(Decision Record) | If this decision was replaced |

**State machine:**
```
Proposed --> Accepted --> [Active]
                |
                +--> Deprecated
                +--> Superseded (by DEC-NNN)
```

**Relationships:**
- **affects** Work Items, Processes, Artifacts
- **supersedes** older Decision Records
- **justified-by** Events (data that informed the decision)
- **constrained-by** Constraints (regulations, budgets, etc.)
- **made-by** Roles
- **scoped-to** a Scope/Container

**Cross-framework examples:**

| Framework | Name | Formality Level |
|---|---|---|
| SAFe 6.0 | Lean Business Case | Formal -- requires cost-benefit analysis |
| PMBOK 7th | Change request disposition | Formal -- change control board approval |
| ITIL v4 | CAB decision | Formal -- advisory board with defined authority |
| Software (ADR) | Architecture Decision Record | Semi-formal -- lightweight markdown document |
| Healthcare | Treatment plan decision | Highly formal -- informed consent, legal liability |
| Legal | Court ruling | Maximum formality -- legally binding precedent |
| Manufacturing | ECN approval | Formal -- engineering review board |

**aibox implication:** aibox already uses DEC-NNN in `DECISIONS.md`. This is well-aligned with the universal pattern. Recommendation: add structured fields for `alternatives` and `consequences` to make decisions more useful. The inverse-chronological format is correct -- most recent decisions matter most.

---

### 2.4 Primitive: Artifact

**Also known as:**
- SAFe: Deliverable, Documentation, Solution
- PMBOK: Deliverable, Work product, Baseline document
- ITIL: Configuration Item (CI), Knowledge article, Service documentation
- Healthcare: Test result, Radiology image, Pathology report, Prescription
- Legal: Brief, Contract, Motion, Exhibit, Transcript, Filing
- Manufacturing: Bill of Materials (BOM), Technical drawing, Product, Assembly
- Supply Chain: Shipping document, Invoice, Packing list, Certificate of origin
- Software: Code, Build, Binary, Release, Documentation, API spec
- Research: Paper, Dataset, Model, Notebook
- BPMN: Data Object, Data Store
- ISO 9001: Documented information (the ISO term for any artifact)

**Definition:** Any tangible output produced or consumed by a process. Artifacts are the "nouns" that processes create, transform, and deliver. They have versions and may require approval before use.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Human-readable name |
| `type` | enum | Classification (document, code, build, report, etc.) |
| `version` | string | Current version (semantic or sequential) |
| `state` | enum | Draft, Review, Approved, Published, Archived, Superseded |
| `author` | reference(Role) | Who created it |
| `created_at` | timestamp | When it was created |
| `location` | URI | Where it is stored |
| `format` | string | File format or media type |

**State machine:**
```
Draft --> In Review --> Approved --> Published --> Archived
                |                                    |
                +--> Rejected (back to Draft)         |
                                                     +--> Superseded
```

**Relationships:**
- **produced-by** Work Items or Processes
- **consumed-by** Work Items or Processes
- **versioned-as** a sequence of Artifacts (version chain)
- **approved-at** a Checkpoint/Gate
- **classified-by** Categories
- **stored-in** a Scope/Container

**aibox implication:** In aibox, artifacts are files in the context directory and generated outputs. The `templates/` directory already defines artifact templates (BACKLOG.md, DECISIONS.md, etc.). aibox should treat all context files as typed artifacts with implicit versioning through git. The process template system (BACK-055) produces artifacts from templates.

---

### 2.5 Primitive: Role

**Also known as:**
- SAFe: Product Owner, Scrum Master, Release Train Engineer, Epic Owner
- PMBOK: Project Manager, Sponsor, Team Member, Stakeholder
- RACI: Responsible, Accountable, Consulted, Informed
- ITIL: Service Owner, Incident Manager, Change Manager, Problem Manager
- Healthcare: Physician, Nurse, Pharmacist, Patient, Attending, Resident
- Legal: Attorney, Judge, Plaintiff, Defendant, Paralegal, Expert Witness
- Manufacturing: Operator, Inspector, Supervisor, Process Engineer
- Supply Chain: Buyer, Supplier, Carrier, Warehouse Manager, Customs Broker
- Kanban: (minimal -- typically just team member)
- ISO 9001: Top management, Process owner, Internal auditor

**Definition:** A named set of responsibilities, permissions, and expectations. Roles are not people -- they are hats that people (or agents) wear. One person can hold multiple roles; one role can be filled by multiple people.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Role name |
| `description` | text | What this role is responsible for |
| `permissions` | string[] | What this role is allowed to do |
| `responsibilities` | string[] | What this role must do |
| `filled_by` | reference(Actor)[] | Current holders of this role |
| `reports_to` | reference(Role) | Hierarchical superior role |

**State machine:** Roles themselves are static definitions. Role *assignments* have a lifecycle:
```
Proposed --> Active --> Suspended --> Revoked
```

**Relationships:**
- **assigned-to** Work Items (as owner, reviewer, approver)
- **participates-in** Processes
- **responsible-for** Artifacts
- **makes** Decisions
- **member-of** a Scope/Container (team, department)
- **has-authority** over Checkpoints/Gates

**Cross-framework examples:**

| Framework | Key Roles | Distinguishing Feature |
|---|---|---|
| SAFe 6.0 | PO, SM, RTE, Epic Owner, Architect | Role per hierarchy level |
| PMBOK 7th | PM, Sponsor, Team, Stakeholder | Stakeholder-centric, PM is integrator |
| ITIL v4 | Service Owner, Process Manager, CAB Chair | Service-oriented role structure |
| Healthcare | Attending, Resident, Nurse, Pharmacist | Legally defined scope of practice |
| Legal | Attorney of Record, Judge, Clerk | Jurisdictionally defined authority |
| RACI | R, A, C, I | Not roles per se but responsibility assignments |

**aibox implication:** aibox uses OWNER.md for the project owner profile and TEAM.md for team structure. The agent-management skill already models agent roles. For universal applicability, aibox should support: (1) role definitions separate from person/agent assignments, (2) RACI-style responsibility mapping per process step, (3) role hierarchies for escalation.

---

### 2.6 Primitive: Process / Workflow

**Also known as:**
- SAFe: PI cadence, Inspect & Adapt ceremony, Portfolio Kanban flow
- PMBOK: Process group, Performance domain activities
- CMMI: Practice Area, Process Area
- ITIL: Practice (Incident Management, Change Enablement, etc.)
- Healthcare: Clinical pathway, Care protocol, Standard operating procedure
- Legal: Litigation process, Contract review process, Discovery procedure
- Manufacturing: Standard work, Production process, Assembly procedure
- Supply Chain: Order-to-cash, Procure-to-pay, Plan-to-produce
- BPMN: Process (the top-level container of activities, events, gateways)
- GTD: The 5-step workflow (Capture, Clarify, Organize, Reflect, Engage)
- Six Sigma: DMAIC (Define, Measure, Analyze, Improve, Control)
- ISO 9001: Documented procedure, Quality management process
- Shape Up: The Shape Up cycle (Shape, Bet, Build)

**Definition:** A defined sequence of steps, decision points, and rules that transforms inputs into outputs. Processes are the "verbs" of the ontology -- they describe *how* work gets done. Every process has a trigger (what starts it), steps (what happens), decision points (branches), and an end condition.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Process name |
| `version` | string | Semantic version |
| `purpose` | text | Why this process exists |
| `trigger` | reference(Event or Schedule) | What starts this process |
| `inputs` | reference(Artifact)[] | Required inputs |
| `outputs` | reference(Artifact)[] | Expected outputs |
| `steps` | Step[] | Ordered sequence of activities |
| `roles` | reference(Role)[] | Who participates |
| `gates` | reference(Checkpoint)[] | Quality checkpoints |
| `metrics` | reference(Metric)[] | How success is measured |
| `owner` | reference(Role) | Who maintains this process |

**State machine (process instance):**
```
Not Started --> In Progress --> Completed
                    |
                    +--> Suspended
                    +--> Aborted
                    +--> Failed (requires corrective action)
```

**Relationships:**
- **contains** Steps (which contain Work Items)
- **triggered-by** Events or Schedules
- **produces** Artifacts
- **governed-by** Constraints
- **validated-by** Checkpoints/Gates
- **measured-by** Metrics
- **performed-by** Roles
- **nested-within** other Processes (sub-processes)
- **tailored-from** a standard Process (CMMI concept)

**Cross-framework examples:**

| Framework | Process Example | Key Characteristic |
|---|---|---|
| SAFe 6.0 | PI Planning ceremony | Time-boxed, cadence-based, face-to-face |
| PMBOK 7th | (Performance domains, not prescriptive processes) | Principle-guided, tailored per project |
| ITIL v4 | Incident Management practice | Event-triggered, SLA-bound |
| DMAIC | Define-Measure-Analyze-Improve-Control | Sequential phases, data-driven |
| Healthcare | Stroke clinical pathway | Time-critical, evidence-based, protocol-driven |
| Legal | Litigation lifecycle | Jurisdiction-bound, adversarial, statute-limited |
| Manufacturing (TPS) | Standard Work sequence | Takt-time-bounded, visual, station-level |
| BPMN | Any BPMN process diagram | Formal notation with events, tasks, gateways, flows |

**aibox implication:** aibox already has process templates in `templates/processes/` (bug-fix, code-review, feature-development, release). The process-frameworks-research.md defines a process document format with YAML frontmatter, stable section IDs, and versioning. This primitive validates that structure. Key additions: (1) every process should declare its trigger condition, (2) processes should be composable (a step in one process can invoke another process), (3) process instances should be tracked in the event log.

---

### 2.7 Primitive: State Machine

**Also known as:**
- SAFe: Portfolio Kanban states, Story lifecycle
- PMBOK: Project phase gates
- ITIL: Incident lifecycle states, Change lifecycle states
- Healthcare: Patient status (admitted, discharged, transferred), Order status
- Legal: Case status (filed, discovery, trial, appeal, closed)
- Manufacturing: Production stage (raw, WIP, finished, shipped)
- Supply Chain: Order status (placed, confirmed, shipped, delivered, returned)
- Kanban: Board columns (To Do, In Progress, Done)
- BPMN: Sequence flow between activities (implicit state machine)
- Software: Git branch states, CI pipeline stages, deployment stages
- Issue trackers: Status field (Open, In Progress, In Review, Closed)

**Definition:** A formal definition of the allowed states an entity can be in and the allowed transitions between those states. State machines are not entities themselves -- they are *definitions* that govern the lifecycle of other primitives (primarily Work Items, Artifacts, and Process instances).

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | What this state machine governs |
| `states` | State[] | All possible states |
| `initial_state` | reference(State) | Starting state |
| `terminal_states` | reference(State)[] | End states |
| `transitions` | Transition[] | Allowed state changes |
| `guards` | Condition[] | Conditions that must be true for a transition |

Each **State** has: `name`, `description`, `is_terminal` (boolean), `entry_actions` (what happens when entering), `exit_actions` (what happens when leaving).

Each **Transition** has: `from_state`, `to_state`, `trigger` (what causes it), `guard` (condition), `action` (side effect).

**The Universal State Pattern:** Across all domains studied, state machines share a common meta-pattern:

```
[Backlog/Queue] --> [Active/In-Progress] --> [Validation/Review] --> [Terminal/Done]
       |                    |                       |
       |                    +--> [Blocked/Waiting]  +--> [Rejected] --> [Active]
       |                    +--> [Cancelled]
       +--> [Deferred/Someday]
```

This pattern holds for:
- Software tickets: Backlog -> In Progress -> In Review -> Done
- Manufacturing orders: Queued -> In Production -> QC Inspection -> Shipped
- Healthcare orders: Ordered -> In Progress -> Resulted -> Verified
- Legal cases: Filed -> Active -> Trial/Hearing -> Decided/Settled
- Supply chain shipments: Created -> In Transit -> Customs -> Delivered

**aibox implication:** aibox should define state machines as first-class configuration objects, not hard-coded enums. Each work item type (task, bug, feature, epic) could have a different state machine. The state machine definition should be declarative (YAML or similar) and referenced by work item types. This enables domain customization without code changes.

---

### 2.8 Primitive: Category / Taxonomy

**Also known as:**
- SAFe: Work type (feature, enabler, defect, debt), PI classification
- PMBOK: Risk category, Stakeholder category, Change type
- ITIL: Incident category, Service category, Priority matrix
- Healthcare: ICD-10 diagnosis code, CPT procedure code, Triage level
- Legal: Case type (civil, criminal, family), Practice area, Jurisdiction
- Manufacturing: Product category, Defect type, Material classification
- Supply Chain: SKU category, Shipping class, Hazmat classification
- GTD: Context (@phone, @computer, @office), Energy level, Time available
- Kanban: Swim lane classification, Class of service (expedite, standard, etc.)
- GitHub/Jira: Label, Component, Issue Type, Priority, Severity
- Six Sigma: Defect category (from Pareto analysis)
- ISO 9001: Nonconformity type, Audit finding category

**Definition:** A system for classifying and organizing entities into groups based on shared characteristics. Taxonomies impose order on complexity and enable filtering, routing, reporting, and prioritization.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Category name |
| `type` | string | What dimension this categorizes (priority, type, area, etc.) |
| `values` | Value[] | Allowed values (ordered or unordered) |
| `hierarchical` | boolean | Whether values form a tree |
| `exclusive` | boolean | Whether an entity can have only one value from this category |
| `default_value` | reference(Value) | Default if unspecified |

**Common category dimensions across all domains:**
1. **Priority/Urgency:** How soon must this be addressed? (Critical/High/Medium/Low or P0-P4)
2. **Type/Nature:** What kind of thing is this? (Feature/Bug/Chore or Incident/Problem/Change)
3. **Area/Domain:** What part of the system/organization does this belong to? (Backend/Frontend or Cardiology/Orthopedics)
4. **Size/Effort:** How big is this? (XS/S/M/L/XL or story points or t-shirt sizes)
5. **Risk/Impact:** What happens if this goes wrong? (Critical/Major/Minor/Cosmetic)

**Relationships:**
- **classifies** Work Items, Artifacts, Events, and most other primitives
- **organized-within** a Scope/Container
- **used-by** Processes (for routing, filtering, prioritization)
- **measured-by** Metrics (distribution across categories)

**aibox implication:** aibox currently uses informal categorization (tags in frontmatter, labels in backlog). For universality, aibox should support: (1) user-defined category dimensions, (2) both flat and hierarchical taxonomies, (3) both exclusive (one priority) and non-exclusive (many labels) categories. The existing tag system in process document frontmatter is a good start.

---

### 2.9 Primitive: Cross-Reference / Relation

**Also known as:**
- SAFe: Dependency (on Program Board), Feature-to-Story link
- PMBOK: Dependency (FS, FF, SS, SF), Traceability link
- ITIL: CI relationship, Incident-to-Problem link, Known Error reference
- Healthcare: Referral link, Lab order to result link, Medication interaction
- Legal: Case citation, Precedent reference, Cross-motion reference
- Manufacturing: BOM component link, Drawing reference, ECN cross-reference
- Supply Chain: PO-to-Invoice link, Shipment-to-Order link
- Zettelkasten: Link between notes (the core mechanism)
- BPMN: Sequence Flow, Message Flow, Association
- GitHub: "References #123", "Fixes #456", PR-to-Issue link
- Systems Thinking: Causal link (reinforcing or balancing)

**Definition:** An explicit, typed connection between two entities. Cross-references create the web of relationships that gives context its meaning. Without references, every item exists in isolation.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier (often implicit) |
| `source` | reference(any) | The entity making the reference |
| `target` | reference(any) | The entity being referenced |
| `type` | enum | The nature of the relationship |
| `direction` | enum | Unidirectional or bidirectional |
| `strength` | enum | Required/Optional/Informational |
| `created_at` | timestamp | When the link was created |

**Universal relationship types (appear in 3+ domains):**
1. **parent-child** (hierarchical containment): Epic->Feature, Matter->Task, Order->Line Item
2. **blocks/blocked-by** (dependency): Task A blocks Task B, Motion blocks Trial
3. **relates-to** (informational association): Issue relates to Issue, Note links to Note
4. **duplicates** (equivalence): Duplicate incident, Duplicate bug
5. **supersedes** (replacement): New decision supersedes old, New version supersedes old
6. **implements/implemented-by** (realization): Story implements Feature, Order fulfills Request
7. **caused-by/causes** (causal): Incident caused by Problem, Defect caused by Change
8. **references** (citation): ADR references Issue, Brief cites Precedent

**Relationships:**
- **connects** any two primitives in the ontology
- **typed-by** Categories (relationship type taxonomy)
- **tracked-in** Event log (link creation/deletion events)

**aibox implication:** aibox uses implicit cross-references today (mentioning BACK-NNN or DEC-NNN in text). For a robust system, aibox should: (1) support explicit typed references in frontmatter or structured fields, (2) maintain a reference index (which items link to what), (3) detect broken references. The Zettelkasten insight is powerful: the value of notes grows with the density of links between them.

---

### 2.10 Primitive: Checkpoint / Gate

**Also known as:**
- SAFe: PI boundary, Definition of Done, System Demo
- PMBOK: Phase gate, Quality checkpoint, Milestone (decision point)
- CMMI: Process appraisal, Maturity assessment
- ITIL: Change review, Release validation, Service acceptance
- Healthcare: Clinical decision point, Discharge criteria check, Surgical safety checklist
- Legal: Motion hearing, Summary judgment, Pre-trial conference
- Manufacturing: Quality inspection point, Go/No-Go decision, Final acceptance test
- Supply Chain: Goods receipt inspection, Customs clearance, Final delivery confirmation
- Six Sigma: Tollgate review (end of each DMAIC phase)
- ISO 9001: Internal audit, Management review, Supplier evaluation
- Shape Up: Betting table (deciding which pitches to bet on)

**Definition:** A designated point in a process where work is evaluated against defined criteria before it can proceed. Gates enforce quality, compliance, and alignment. They are the mechanism by which organizations ensure standards are met.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Gate name |
| `criteria` | Criterion[] | What must be true to pass |
| `authority` | reference(Role) | Who can approve passage |
| `severity` | enum | Blocking (must pass) or Advisory (may pass with waiver) |
| `automated` | boolean | Whether criteria are machine-checkable |
| `result` | enum | Passed, Failed, Waived |
| `evidence` | reference(Artifact)[] | What proves the criteria are met |
| `evaluated_at` | timestamp | When the gate was evaluated |

**State machine:**
```
Pending --> Evaluating --> Passed
                |
                +--> Failed --> Remediation --> Re-evaluation --> Passed
                +--> Waived (with justification)
```

**Relationships:**
- **validates** Work Items, Artifacts, Process instances
- **enforced-by** Processes
- **evaluated-by** Roles
- **produces** Events (pass/fail records)
- **requires** Artifacts (as evidence)
- **defined-within** a Scope/Container

**Cross-framework examples:**

| Framework | Gate Name | Blocking? | Automated? |
|---|---|---|---|
| SAFe 6.0 | Story DoD, Feature DoD | Yes | Partially (CI checks) |
| PMBOK 7th | Phase gate | Yes | No (human review) |
| Six Sigma | DMAIC tollgate | Yes | No |
| Manufacturing | Final QC inspection | Yes | Often (automated testing) |
| Healthcare | Surgical safety checklist (WHO) | Yes (life-critical) | No |
| Software CI | Pre-merge checks | Yes | Yes (fully automated) |
| ISO 9001 | Internal audit | Advisory | No |

**aibox implication:** aibox already has quality gate triggers (`triggers/quality-gate.yaml`). This primitive validates that design. Enhancements: (1) gates should be declarative (list of criteria, each with pass/fail evaluation), (2) gates can be automated (run tests) or manual (require human sign-off), (3) gate results should be logged as events. The existing Definition of Done pattern in process templates maps directly.

---

### 2.11 Primitive: Metric / Measure

**Also known as:**
- SAFe: Flow Metrics (velocity, time, efficiency, load, distribution, predictability)
- PMBOK: EVM indices (SPI, CPI), KPIs, Performance measures
- CMMI: Process performance baselines, Statistical process control measures
- ITIL: SLA metrics, Service level targets, Mean Time to Restore (MTTR)
- Healthcare: Clinical outcome measures, Patient satisfaction scores, Readmission rates
- Legal: Billable hours, Case resolution time, Win/loss rate
- Manufacturing: OEE (Overall Equipment Effectiveness), Defect rate, Yield, Takt time
- Supply Chain: On-time delivery rate, Fill rate, Inventory turnover, Lead time
- Six Sigma: Sigma level, DPMO (Defects Per Million Opportunities), Cp/Cpk
- OKR: Key Result (measurable outcome)
- Kanban: Lead time, Cycle time, Throughput, WIP count
- Systems Thinking: Stock level, Flow rate

**Definition:** A quantified observation about a system, process, or outcome. Metrics are derived from Events (aggregated over time) and provide the feedback signal for improvement. Without metrics, improvement is guesswork.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Metric name |
| `description` | text | What this metric measures and why it matters |
| `unit` | string | Unit of measurement (hours, count, percentage, etc.) |
| `formula` | expression | How to compute the value from raw data |
| `target` | number | Desired value or range |
| `threshold` | range | Acceptable range (green/yellow/red) |
| `frequency` | reference(Schedule) | How often it is computed |
| `source` | reference(Event type)[] | What events feed this metric |
| `current_value` | number | Most recent computed value |
| `trend` | enum | Improving, Stable, Degrading |

**State machine:** Metrics themselves don't have states. Metric *values* are time-series data points (a stream of Events). But a metric's *health* can be:
```
Within Target (Green) --> Approaching Threshold (Yellow) --> Breached (Red)
                                                                |
                                                                +--> Escalation triggered
```

**Relationships:**
- **measures** Work Items, Processes, Roles, Scopes
- **derived-from** Events (aggregation)
- **compared-against** targets and thresholds
- **reported-on** Schedules (cadence)
- **triggers** Processes (when thresholds are breached)
- **informs** Decisions

**The Universal Metric Categories:**
1. **Time metrics:** Lead time, cycle time, flow time, wait time, response time
2. **Volume metrics:** Throughput, velocity, count, inventory level
3. **Quality metrics:** Defect rate, pass rate, accuracy, sigma level
4. **Efficiency metrics:** Utilization, flow efficiency, yield, OEE
5. **Predictability metrics:** Variance, standard deviation, forecast accuracy
6. **Satisfaction metrics:** NPS, CSAT, engagement score

Every domain has metrics in at least 4 of these 6 categories.

**aibox implication:** aibox's event-sourced architecture (BACK-050) with a materializer is the correct pattern for metrics. The materializer computes metrics from raw events. Key insight: aibox should ship with a small set of universal metrics (lead time, throughput, WIP, quality gate pass rate) and allow users to define custom metrics via formulas over event types.

---

### 2.12 Primitive: Schedule / Cadence

**Also known as:**
- SAFe: PI cadence (8-12 weeks), Iteration cadence (2 weeks), IP iteration
- PMBOK: Project schedule, Milestone dates, Sprint duration
- ITIL: Service window, Maintenance window, Review cadence
- Healthcare: Medication schedule, Follow-up interval, Shift schedule
- Legal: Court calendar, Statute of limitations deadline, Filing deadline
- Manufacturing: Production schedule, Takt time, Shift pattern, Maintenance interval
- Supply Chain: Delivery schedule, Reorder point, Safety stock review cycle
- GTD: Weekly review, Daily review
- Shape Up: 6-week cycle + 2-week cooldown
- Kanban: (no fixed cadence -- pull-based -- but often has regular replenishment meetings)
- Cron: Cron expression (the ultimate schedule primitive)

**Definition:** A time-based pattern that triggers activities at regular intervals or specific dates. Cadences create rhythm and predictability. Deadlines create urgency. Together they structure work in time.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Schedule name |
| `type` | enum | Recurring cadence, Fixed deadline, Time window |
| `pattern` | string | Recurrence pattern (weekly, biweekly, cron expression) |
| `start_date` | date | When the schedule begins |
| `end_date` | date | When the schedule ends (or null for indefinite) |
| `duration` | duration | How long each occurrence lasts |
| `timezone` | string | Applicable timezone |
| `triggered_process` | reference(Process) | What happens when the schedule fires |

**The Two Fundamental Time Patterns:**

1. **Cadence (recurring):** Regular rhythm that creates predictability. Examples: 2-week sprints, quarterly planning, annual audits, daily standups, weekly reviews. GTD's weekly review is a cadence. SAFe's PI is a cadence.

2. **Deadline (point-in-time):** A specific date/time by which something must happen. Examples: filing deadlines, release dates, contract expiration, statute of limitations. Deadlines create urgency and are often externally imposed.

Every process framework uses one or both. Even "flow-based" approaches like Kanban, which reject iteration cadences, still use cadences for retrospectives and replenishment meetings.

**Relationships:**
- **triggers** Processes and Events
- **constrains** Work Items (deadlines)
- **paces** Scopes/Containers (iteration boundaries)
- **measured-by** Metrics (schedule adherence)
- **owned-by** Roles

**aibox implication:** aibox's heartbeat cycle (BACK-046) is a cadence. The PI concept (N heartbeat cycles) is a higher-level cadence. aibox should support: (1) user-defined cadences (daily, weekly, per-sprint, per-PI), (2) deadline tracking on work items, (3) cadence-triggered processes (retrospective, planning, review). The scheduled-tasks research (scheduled-tasks-2026-03.md) likely covers implementation.

---

### 2.13 Primitive: Scope / Container

**Also known as:**
- SAFe: Portfolio, Value Stream, ART, Team, PI (as a time-bounded scope)
- PMBOK: Project, Program, Portfolio, Phase, Work Package (as scope boundary)
- ITIL: Service, Service Portfolio, IT Service
- Healthcare: Ward, Department, Patient encounter, Care episode
- Legal: Case, Matter, Docket, Practice group
- Manufacturing: Product line, Assembly line, Work cell, Plant
- Supply Chain: Warehouse, Distribution center, Fulfillment zone, Channel
- GTD: Project (outcome requiring multiple actions), Area of responsibility
- PARA: Project, Area, Resource, Archive
- Shape Up: Bet (a 6-week scope), Scope (within a bet)
- GitHub: Repository, Organization, Project board, Milestone
- Jira: Project, Board, Sprint, Epic (as container)
- Kanban: Board, Swim lane

**Definition:** A boundary that groups related items together and defines a context for work. Scopes are the organizational hierarchy -- they answer "where does this belong?" Scopes can be nested (portfolio > project > sprint > task).

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Container name |
| `type` | enum | Portfolio, Project, Iteration, Team, Area, etc. |
| `description` | text | Purpose and boundaries |
| `parent` | reference(Scope) | Enclosing scope (for nesting) |
| `owner` | reference(Role) | Who is responsible for this scope |
| `state` | enum | Active, Planned, Completed, Archived |
| `start_date` | date | When this scope is active from |
| `end_date` | date | When this scope ends (or null for ongoing) |
| `constraints` | reference(Constraint)[] | Rules that apply within this scope |

**State machine:**
```
Planned --> Active --> Completed --> Archived
               |
               +--> Suspended
               +--> Cancelled
```

**The Universal Nesting Pattern:**

Every domain exhibits hierarchical scoping, typically 3-5 levels deep:

| Domain | Level 1 | Level 2 | Level 3 | Level 4 |
|---|---|---|---|---|
| SAFe | Portfolio | Value Stream/ART | Team | Iteration |
| PMBOK | Portfolio | Program | Project | Phase/WP |
| Healthcare | Hospital | Department | Ward | Patient Encounter |
| Legal | Firm | Practice Group | Matter | Task |
| Manufacturing | Company | Plant | Line | Station |
| Supply Chain | Network | Region | Warehouse | Zone |
| PARA | (all) | Project/Area | | |
| GTD | (all) | Area of Focus | Project | Next Action |

**Relationships:**
- **contains** Work Items, Artifacts, Events, Roles, sub-Scopes
- **governed-by** Processes and Constraints
- **measured-by** Metrics (aggregated from contents)
- **owned-by** Roles
- **bounded-by** Schedules (time-boxed scopes like sprints)

**aibox implication:** aibox already has scoping through the context directory structure and PROJECTS.md with PROJ-NNN IDs. The `aibox.toml` defines a project-level scope. For multi-project support, aibox should: (1) support project-level and sub-project scoping, (2) allow work items to belong to a scope, (3) aggregate metrics per scope. The PARA model's Project/Area/Resource/Archive maps well to aibox's context directory structure (active projects, ongoing areas, reference material, archive/).

---

### 2.14 Primitive: Constraint

**Also known as:**
- SAFe: Guardrails (budget, scope, quality), PI objectives bounds
- PMBOK: Constraint (scope, time, cost -- the "triple constraint"), Assumption
- CMMI: Policy, Standard, Regulation
- ITIL: SLA, OLA (Operational Level Agreement), Underpinning Contract
- Healthcare: Regulation (HIPAA, FDA), Clinical guideline, Formulary restriction
- Legal: Statute, Regulation, Precedent, Jurisdiction, Ethical rule
- Manufacturing: Specification limit (USL/LSL), Safety regulation, Environmental regulation
- Supply Chain: Customs regulation, Trade agreement, Capacity limit, Lead time constraint
- BPMN: Rule (Business Rule Task)
- Six Sigma: Specification limits (USL, LSL), Process capability requirement
- ISO 9001: Requirement, Statutory/regulatory requirement
- Kanban: WIP limit

**Definition:** A rule, limit, regulation, or condition that restricts the degrees of freedom in how work is performed. Constraints are the boundaries within which all other primitives operate. They are not optional -- violation of a constraint is a nonconformity or compliance failure.

This primitive was **not in the original hypothesis** but emerged strongly from cross-domain analysis. Every domain has hard constraints that shape all processes. WIP limits in Kanban, specification limits in manufacturing, SLAs in IT service management, statutes in law, regulations in healthcare -- these are all the same underlying primitive.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Constraint name |
| `type` | enum | Regulatory, Policy, Resource, Time, Quality, Budget |
| `description` | text | What the constraint requires or prohibits |
| `scope` | reference(Scope) | Where this constraint applies |
| `severity` | enum | Mandatory (violation = noncompliance) or Advisory (best practice) |
| `source` | string | Where the constraint comes from (regulation, policy, contract, physics) |
| `enforcement` | enum | Automated, Manual, Audit-based |
| `violation_action` | reference(Process) | What happens when violated |

**State machine:** Constraints are definitions, not instances. They are either Active or Inactive.
```
Draft --> Active --> Superseded
            |
            +--> Suspended (temporary waiver)
```

**Relationships:**
- **restricts** Work Items, Processes, Scopes, Roles
- **enforced-at** Checkpoints/Gates
- **violated-by** Events (triggering corrective action)
- **derived-from** external sources (laws, contracts, policies)
- **measured-by** Metrics (compliance rate, violation count)
- **documented-as** Artifacts (policy documents, SLAs)

**aibox implication:** aibox should support constraint definitions that are enforced by quality gates and process steps. Examples: "WIP limit of 5 items in progress," "all code changes require review," "maximum response time of 2 hours for critical incidents." Constraints bridge the gap between process definitions and automated enforcement.

---

### 2.15 Primitive: Context / Environment

**Also known as:**
- SAFe: Solution Context, Portfolio Context
- PMBOK: Enterprise Environmental Factors (EEFs), Organizational Process Assets (OPAs)
- CMMI: Organizational environment, Standard environment
- ITIL: Service environment, Configuration baseline
- Healthcare: Patient context (allergies, medications, history), Clinical setting
- Legal: Jurisdiction, Applicable law, Case background
- Manufacturing: Production environment, Tooling setup, Environmental conditions
- Supply Chain: Market conditions, Supplier landscape, Demand forecast
- GTD: Context (@phone, @computer, @errands, @office)
- Systems Thinking: System boundary, Environment

**Definition:** The ambient information, conditions, and configuration that surrounds and influences work but is not itself a work item. Context is "everything you need to know to do the work effectively." It includes the environment in which work happens, the background knowledge required, and the situational factors that affect decisions.

This primitive was **not in the original hypothesis** but is fundamental -- especially for AI agents that need explicit context to function. Every framework acknowledges that work does not happen in a vacuum; the environment shapes everything.

**Universal attributes:**
| Attribute | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `name` | string | Context name |
| `type` | enum | Environmental, Organizational, Technical, Regulatory, Situational |
| `description` | text | What conditions or knowledge this represents |
| `scope` | reference(Scope) | Where this context applies |
| `artifacts` | reference(Artifact)[] | Documents, configs, knowledge bases |
| `constraints` | reference(Constraint)[] | Active constraints in this context |
| `valid_from` | date | When this context became effective |
| `valid_until` | date | When this context expires (or null) |

**Relationships:**
- **surrounds** all other primitives within its Scope
- **contains** Artifacts (reference material, configuration)
- **imposes** Constraints
- **informs** Decisions
- **consumed-by** Roles (especially AI agents that need explicit context loading)

**aibox implication:** This is perhaps the most important primitive for aibox. The entire `context/` directory IS this primitive. aibox's core value proposition is structuring context so that AI agents and humans can work effectively. CLAUDE.md, OWNER.md, work-instructions/, research/ -- these are all context artifacts. aibox should treat the context directory as a first-class, typed, versioned environment that is loaded into agents at the start of every work session. The GTD "@context" concept maps to aibox's process flavors -- different contexts for different work types.

---

## 3. Primitive Relationship Diagram

```
                                    +-----------+
                                    |  CONTEXT  |
                                    | /Environ. |
                                    +-----+-----+
                                          |
                                    surrounds/informs
                                          |
                         +----------------+----------------+
                         |                                 |
                   +-----+-----+                    +------+------+
                   |   SCOPE   |                    | CONSTRAINT  |
                   | /Container|----restricts------>|  /Rule      |
                   +-----+-----+                    +------+------+
                         |                                 |
                    contains                          enforced-at
                         |                                 |
          +--------------+--------------+           +------+------+
          |              |              |           | CHECKPOINT  |
          |              |              |           |   /Gate     |
    +-----+-----+  +----+----+  +------+------+   +------+------+
    |   WORK    |  |  ROLE   |  |  SCHEDULE   |          |
    |   ITEM    |  |         |  |  /Cadence   |     validates
    +-----+-----+  +----+----+  +------+------+          |
          |              |              |           +-----+-----+
          |         performs        triggers        |  PROCESS  |
          |              |              |           | /Workflow |
          |              +------+-------+           +-----+-----+
          |                     |                         |
          +-------governed-by---+-----------+             |
          |                                 |          contains
          |                                 |          steps
     +----+----+                     +------+------+      |
     | STATE   |                     |  DECISION   |      |
     | MACHINE |                     |  RECORD     |      |
     +----+----+                     +------+------+      |
          |                                 |             |
       governs                          informs           |
     lifecycle                              |             |
          |              +------------------+             |
          |              |                                |
    +-----+-----+  +----+-----+                    +-----+-----+
    | CATEGORY  |  |   LOG    |<----produced-by----|  ARTIFACT  |
    | /Taxonomy |  |  ENTRY   |                    |            |
    +-----+-----+  |  /Event  |                    +-----+-----+
          |         +----+-----+                          |
     classifies          |                          versioned
          |         aggregated-into                        |
          |              |                          +-----+-----+
          +---------+----+-----+                    |  METRIC   |
                    |  CROSS-  |----measures-------->| /Measure  |
                    |REFERENCE |                    +-----------+
                    | /Relation|
                    +----------+
```

**Key relationships summarized:**

| From | Relationship | To |
|---|---|---|
| Context | surrounds | all primitives within its Scope |
| Scope | contains | Work Items, Roles, Artifacts, sub-Scopes |
| Constraint | restricts | Work Items, Processes, Scopes, Roles |
| Checkpoint | validates | Work Items, Artifacts, Process instances |
| Checkpoint | enforces | Constraints |
| Process | contains | Steps (which are Work Items) |
| Process | triggered-by | Events and Schedules |
| Process | produces | Artifacts and Events |
| Role | performs | Processes |
| Role | assigned-to | Work Items |
| Role | evaluates | Checkpoints |
| State Machine | governs | Work Item lifecycle, Artifact lifecycle, Process lifecycle |
| Category | classifies | all primitives (priority, type, area, etc.) |
| Cross-Reference | connects | any two primitives |
| Log Entry | records | state changes, actions, observations on any primitive |
| Metric | derived-from | Log Entries (aggregated) |
| Metric | measures | Work Items, Processes, Scopes |
| Schedule | triggers | Processes |
| Schedule | constrains | Work Items (deadlines), Scopes (iteration boundaries) |
| Decision Record | affects | Work Items, Processes, Constraints |
| Artifact | produced-by | Work Items, Processes |

---

## 4. The Primitive Composition Model

These 15 primitives compose to express any process framework. Here are examples showing how different frameworks are built from the same atoms:

### 4.1 SAFe 6.0 Expressed in Primitives

| SAFe Concept | Primitive Composition |
|---|---|
| Portfolio | Scope(type=portfolio) |
| Epic | Work Item(type=epic, parent=portfolio) + State Machine(portfolio-kanban) |
| Lean Business Case | Artifact(type=business-case) + Checkpoint(gate=epic-approval) |
| ART | Scope(type=art) with Role(type=RTE) |
| Feature | Work Item(type=feature, parent=art-backlog) |
| PI | Scope(type=pi) + Schedule(cadence=8-12-weeks) |
| PI Planning | Process(trigger=pi-start) with Checkpoint(gate=commitment) |
| Story | Work Item(type=story, parent=feature) |
| Iteration | Scope(type=iteration) + Schedule(cadence=2-weeks) |
| Flow Metrics | Metric(source=work-item-events) x 6 |
| I&A Ceremony | Process(trigger=pi-end) producing Decision Record + Work Items(improvement) |
| DoD | Checkpoint(criteria=[...]) at Process step boundary |

### 4.2 GTD Expressed in Primitives

| GTD Concept | Primitive Composition |
|---|---|
| Inbox | Scope(type=inbox, state=unprocessed) |
| Next Action | Work Item(type=action, state=ready) |
| Project | Scope(type=project) containing Work Items |
| Waiting For | Work Item(state=blocked, blocked-by=external) |
| Someday/Maybe | Work Item(state=deferred) in Scope(type=someday) |
| Context | Category(type=context, values=[@phone, @computer, @office]) |
| Weekly Review | Process(trigger=Schedule(cadence=weekly)) |
| Reference | Artifact(type=reference) in Scope(type=reference) |
| Area of Focus | Scope(type=area, ongoing=true) |

### 4.3 Clinical Pathway (Healthcare) Expressed in Primitives

| Healthcare Concept | Primitive Composition |
|---|---|
| Patient Encounter | Scope(type=encounter) |
| Clinical Order | Work Item(type=order) with Constraint(type=formulary) |
| Clinical Pathway | Process(type=pathway) with Checkpoints at decision points |
| Vital Signs | Log Entry(type=observation) feeding Metric(type=vital) |
| Diagnosis | Decision Record(type=diagnosis, evidence=[observations]) |
| Medication | Artifact(type=prescription) with Constraint(type=interaction-check) |
| Care Team | Role[] assigned to Scope(type=encounter) |
| Discharge Criteria | Checkpoint(criteria=[...], severity=blocking) |
| Triage Level | Category(type=acuity, values=[1,2,3,4,5]) |

### 4.4 Kanban Expressed in Primitives

| Kanban Concept | Primitive Composition |
|---|---|
| Board | Scope(type=board) |
| Card | Work Item in the board scope |
| Column | State in State Machine(governing=work-items-on-board) |
| WIP Limit | Constraint(type=wip-limit, scope=column) |
| Swim Lane | Category(type=swim-lane) classifying Work Items |
| Class of Service | Category(type=service-class, values=[expedite, standard, etc.]) |
| Cumulative Flow | Metric(type=cumulative-flow, source=state-change-events) |
| Lead Time | Metric(formula=done_at - created_at) |
| Replenishment Meeting | Process(trigger=Schedule(cadence=as-needed)) |
| Blocked Indicator | Work Item(state=blocked) |

### 4.5 BPMN Expressed in Primitives

| BPMN Concept | Primitive Composition |
|---|---|
| Process | Process |
| Task | Work Item (step in a Process) |
| Event (Start/End/Intermediate) | Log Entry(type=process-event) or Schedule(trigger) |
| Gateway (Exclusive/Parallel/Inclusive) | Checkpoint(type=decision-point) in Process |
| Sequence Flow | Cross-Reference(type=sequence, source=step-A, target=step-B) |
| Message Flow | Cross-Reference(type=message, between processes) |
| Data Object | Artifact |
| Lane/Pool | Scope(type=lane) with Role assignment |
| Timer Event | Schedule(trigger) |
| Error Event | Log Entry(type=error) triggering Process(error-handling) |

---

## 5. Cross-Domain Validation Matrix

This matrix shows which primitives appear in which domains, confirming universality:

| Primitive | Software/IT | Manufacturing | Healthcare | Legal | Supply Chain | Knowledge Mgmt | Quality Mgmt |
|---|---|---|---|---|---|---|---|
| Work Item | Issue/Story | Work Order | Clinical Order | Case/Matter | Purchase Order | Next Action | Corrective Action |
| Log Entry | Commit/Event | Production Log | Clinical Note | Docket Entry | Tracking Event | Journal Entry | Audit Finding |
| Decision Record | ADR | ECN | Treatment Decision | Ruling | Vendor Selection | (informal) | Management Review |
| Artifact | Code/Build | BOM/Drawing | Test Result | Brief/Contract | Invoice/BOL | Note/Document | Procedure Doc |
| Role | Developer/PM | Operator/Inspector | Physician/Nurse | Attorney/Judge | Buyer/Carrier | Author/Reviewer | Auditor/Process Owner |
| Process | CI/CD Pipeline | Standard Work | Clinical Pathway | Litigation Process | Order-to-Cash | Weekly Review | Audit Procedure |
| State Machine | Ticket States | Production Stages | Order Lifecycle | Case Lifecycle | Shipment Stages | (minimal) | NC Lifecycle |
| Category | Label/Type | Defect Category | ICD-10/CPT | Case Type | SKU Category | Tag | NC Category |
| Cross-Reference | "Fixes #123" | BOM Reference | Referral Link | Case Citation | PO-to-Invoice | Zettel Link | NC-to-CA Link |
| Checkpoint | Code Review/CI | QC Inspection | Safety Checklist | Hearing/Motion | Goods Receipt | (informal) | Internal Audit |
| Metric | Lead Time/SPI | OEE/Yield | Readmission Rate | Resolution Time | On-time Delivery | (minimal) | Sigma Level |
| Schedule | Sprint/PI | Shift/Takt | Medication Schedule | Court Calendar | Delivery Schedule | Weekly Review | Audit Schedule |
| Scope | Repo/Project | Plant/Line | Ward/Encounter | Matter/Docket | Warehouse/Zone | Project/Area | Process Area |
| Constraint | WIP Limit | Spec Limit | Regulation | Statute | Trade Rule | (minimal) | Requirement |
| Context | Tech Stack/Env | Production Env | Patient History | Jurisdiction | Market Conditions | Reference Material | Organizational Context |

**Universality score:** All 15 primitives appear in at least 5 of 7 domains surveyed. Work Item, Log Entry, Role, Process, Category, and Scope appear in all 7. This confirms the ontology is genuinely universal.

---

## 6. Minimal Viable Primitive Set

Not all 15 primitives are equally fundamental. They can be layered:

**Layer 0 -- Irreducible Core (cannot build any process without these):**
1. **Work Item** -- something to do
2. **Log Entry / Event** -- record of what happened
3. **State Machine** -- lifecycle governance
4. **Role** -- who does it

**Layer 1 -- Structural (needed to organize work at scale):**
5. **Scope / Container** -- grouping boundary
6. **Cross-Reference / Relation** -- connections between items
7. **Category / Taxonomy** -- classification

**Layer 2 -- Process (needed to define how work flows):**
8. **Process / Workflow** -- sequence of steps
9. **Checkpoint / Gate** -- quality enforcement
10. **Artifact** -- produced outputs
11. **Schedule / Cadence** -- time structure

**Layer 3 -- Governance (needed for continuous improvement and compliance):**
12. **Decision Record** -- rationale preservation
13. **Metric / Measure** -- quantified feedback
14. **Constraint** -- rules and limits
15. **Context / Environment** -- ambient knowledge

This layering matters for aibox because it defines the implementation order: Layer 0 must exist before anything else works. Each subsequent layer adds capability but requires the previous layers.

---

## 7. Recommendations for aibox

### 7.1 Architecture Recommendations

1. **Implement all 15 primitives as first-class concepts in the context system.** Each primitive should have a recognizable representation in the context directory structure. Most already do:
   - Work Item = BACKLOG.md entries (BACK-NNN)
   - Decision Record = DECISIONS.md entries (DEC-NNN)
   - Scope = PROJECTS.md entries (PROJ-NNN) + context directory structure
   - Artifact = any file in context/
   - Role = OWNER.md, TEAM.md
   - Process = templates/processes/*.md
   - Log Entry = event log (BACK-050)
   - Context = CLAUDE.md, work-instructions/, research/

2. **Missing or underspecified primitives to add:**
   - **State Machine** -- currently implicit in backlog status column; should be explicit and configurable per work item type
   - **Checkpoint / Gate** -- exists as trigger but should be a standalone concept with criteria lists
   - **Metric** -- planned via materializer but needs a metric definition format
   - **Schedule / Cadence** -- heartbeat exists but user-facing cadence definitions are missing
   - **Constraint** -- not currently represented; add as a section in project config or a dedicated file
   - **Cross-Reference** -- currently informal (text mentions); should support structured references
   - **Category / Taxonomy** -- currently ad-hoc labels; should support user-defined dimensions

3. **Primitive schema definitions.** Create a `schemas/primitives/` directory with versioned schema definitions for each primitive. These schemas define the universal attributes, allowed states, and relationships. Process packages then reference these schemas.

### 7.2 Context Directory Mapping

Recommended mapping of primitives to context directory structure:

```
context/
  BACKLOG.md           -- Work Item registry (BACK-NNN)
  DECISIONS.md         -- Decision Record registry (DEC-NNN)
  PROJECTS.md          -- Scope registry (PROJ-NNN)
  PRD.md               -- Context (product requirements as ambient knowledge)
  OWNER.md             -- Role (project owner profile)
  CONSTRAINTS.md       -- Constraint registry (new)
  work-instructions/   -- Context (reference material)
    DEVELOPMENT.md     -- Context
    GENERAL.md         -- Context
    TEAM.md            -- Role definitions
  processes/           -- Process definitions
  research/            -- Artifact (research outputs)
  archive/             -- Scope (archived items)
```

### 7.3 Process Package Mapping

The 13 composable process packages (DEC-016) should map to primitive combinations:

| Process Package | Primary Primitives Used |
|---|---|
| core (always present) | Context, Role |
| backlog | Work Item, State Machine, Category |
| decisions | Decision Record |
| standups | Log Entry, Schedule |
| retrospectives | Log Entry, Metric, Schedule, Process |
| projects | Scope, Cross-Reference |
| research | Artifact, Log Entry |
| releases | Process, Checkpoint, Artifact |
| code-review | Process, Checkpoint, Role |
| estimation | Metric, Category (size), Work Item |
| quality-gates | Checkpoint, Constraint, Metric |
| documentation | Artifact, Process |
| incident-response | Work Item (incident type), Process, Constraint (SLA) |

### 7.4 Implementation Priority

Based on the layered model (Section 6), the implementation order should be:

**Phase 1 -- Already done (Layer 0-1):**
- Work Item (BACKLOG.md with BACK-NNN)
- Role (OWNER.md, TEAM.md)
- Scope (PROJECTS.md, context/ structure)
- Category (informal tags/labels)
- Cross-Reference (informal text mentions)

**Phase 2 -- In progress (Layer 1-2):**
- Log Entry / Event (BACK-050 event log)
- Process (templates/processes/*.md)
- Checkpoint (quality gate triggers)
- Artifact (implicit via files)

**Phase 3 -- Next (Layer 2-3):**
- State Machine -- explicit, configurable state definitions per work item type
- Metric -- metric definition format + materializer computation
- Schedule -- user-facing cadence definitions
- Decision Record enhancements -- add structured alternatives/consequences

**Phase 4 -- Future (Layer 3):**
- Constraint -- formal constraint definitions with automated enforcement
- Context -- typed context loading for agents (which context files to load when)
- Cross-Reference -- structured reference index with broken-link detection
- Category -- user-defined taxonomy dimensions

### 7.5 Key Design Principles

1. **Primitives are domain-agnostic.** The primitive definitions never mention "software" or "code." A Work Item is equally valid for a legal case, a manufacturing order, or a research task. Domain specificity comes from the *configuration* (state machine definitions, category values, process templates), not the primitives themselves.

2. **Composition over prescription.** aibox should not prescribe "use SAFe" or "use Kanban." It should provide the 15 primitives and let users (or process packages) compose them into whatever framework fits their domain.

3. **Events are the source of truth.** Every state change, decision, checkpoint evaluation, and metric computation should be traceable to events. This is the event-sourcing principle: the event log is the primary data store; everything else is a materialized view.

4. **Progressive elaboration.** Users start with Layer 0 primitives (work items with states and owners) and add layers as needed. A solo developer needs Layer 0-1. A team needs Layer 2. An organization needs Layer 3. aibox's composable process packages (DEC-016) already align with this model.

5. **Text-first, schema-validated.** Primitives are represented as Markdown with structured metadata (frontmatter, tables, or inline YAML). This keeps the human-readable quality that makes aibox context useful for AI agents while allowing programmatic validation against schemas.

---

## 8. Sources and Cross-References

### Frameworks analyzed in existing research (process-frameworks-research.md):
- SAFe 6.0 (Scaled Agile Framework)
- PMBOK 7th Edition (Project Management Body of Knowledge)
- CMMI v3.0 (Capability Maturity Model Integration)
- IPMA ICB4 (Individual Competence Baseline)
- OPM3 (Organizational Project Management Maturity Model)

### Additional frameworks analyzed for this ontology:
- **Toyota Production System / Lean:** Ohno, T. (1988). Toyota Production System; Womack & Jones (1996). Lean Thinking. Core primitives: work order (kanban card), standard work (process), takt time (schedule/cadence), andon (event/signal), jidoka (checkpoint), kaizen (improvement process), muda/waste (constraint).
- **Six Sigma / DMAIC:** Harry & Schroeder (2000). Six Sigma. Core primitives: CTQ (work item), DMAIC phases (process), tollgate (checkpoint), control chart (metric), specification limit (constraint), DPMO (metric), Pareto category (taxonomy).
- **BPMN 2.0:** OMG (2013). Business Process Model and Notation v2.0. Core elements map 1:1: Task=Work Item, Event=Log Entry, Gateway=Checkpoint, Data Object=Artifact, Lane=Scope+Role, Sequence Flow=Cross-Reference.
- **ITIL v4:** AXELOS (2019). ITIL Foundation. Core primitives: service (scope), incident/problem/change (typed work items), SLA (constraint+metric), event (log entry), configuration item (artifact), practice (process).
- **ISO 9001:2015:** Core primitives: documented information (artifact), nonconformity (work item), corrective action (work item+process), internal audit (process+checkpoint), management review (process+decision), requirement (constraint).
- **GTD (Getting Things Done):** Allen, D. (2001/2015). Getting Things Done. Core primitives: next action (work item), project (scope), context (category), weekly review (process+schedule), reference (artifact/context), someday/maybe (state in state machine).
- **PARA Method:** Forte, T. (2017). The PARA Method. Core primitives: project (scope, time-bounded), area (scope, ongoing), resource (context/artifact), archive (scope, completed).
- **Zettelkasten:** Luhmann, N.; Ahrens, S. (2017). How to Take Smart Notes. Core insight: the primitive is the note (artifact) + the link (cross-reference). Value emerges from density of cross-references, not from individual items.
- **Shape Up:** Fried, J. & Singer, R. (2019). Shape Up. Core primitives: pitch (artifact+decision), bet (scope, 6-week), scope (work item grouping within bet), hill chart (metric visualization), cooldown (schedule).
- **Kanban (David Anderson):** Anderson, D. (2010). Kanban. Core primitives: card (work item), board (scope), column (state), WIP limit (constraint), class of service (category), lead time (metric).
- **Systems Thinking:** Meadows, D. (2008). Thinking in Systems. Core primitives: stock (metric/state), flow (event stream), feedback loop (process), delay (schedule), leverage point (constraint/checkpoint).
