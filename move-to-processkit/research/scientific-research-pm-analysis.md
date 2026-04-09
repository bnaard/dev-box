# Scientific Research Project Management — Deep Analysis

**Date:** 2026-03-27
**Status:** Complete
**Purpose:** Analyze what a scientific research process template needs, map to existing primitives, identify gaps

---

## 1. Academic Research Workflow

### The Full Lifecycle

```
Hypothesis → Literature Review → Methodology Design → Data Collection
→ Analysis → Paper Writing → Internal Review → Submission
→ Peer Review → Revision → Acceptance → Proofs → Publication
→ Citation Tracking → Follow-up Studies
```

### Primitive Mapping

| Workflow Stage | Primary Primitive | Supporting Primitives |
|---|---|---|
| Hypothesis | **Work Item** (type: hypothesis) | Context, Discussion, Cross-Reference (to prior work) |
| Literature Review | **Artifact** (type: literature-review) | Category (by topic/method), Cross-Reference (citations) |
| Methodology Design | **Process** (type: methodology) | Decision (method selection), Constraint (ethical/resource), Gate (IRB approval) |
| Data Collection | **Work Item** (type: data-collection) | Schedule, Artifact (raw data), State Machine, Constraint (sample size) |
| Analysis | **Work Item** (type: analysis) | Artifact (scripts, results), Metric (statistical outputs) |
| Paper Writing | **Artifact** (type: manuscript) | State Machine, Role (author assignments), Schedule (deadline) |
| Peer Review | **Event** (type: review-received) | Actor (reviewer), Decision (accept/reject/revise), Gate |
| Publication | **Artifact** (type: published-paper) | Cross-Reference (DOI, citation), Category (journal, field) |
| Citation Tracking | **Metric** (citation-count) | Cross-Reference (citing works), Event (citation-alert) |

### State Machine: Research Project Lifecycle

```
States: IDEATION → LITERATURE_REVIEW → DESIGN → IRB_PENDING → APPROVED
  → DATA_COLLECTION → ANALYSIS → WRITING → INTERNAL_REVIEW
  → SUBMITTED → UNDER_REVIEW → REVISION_REQUESTED → RESUBMITTED
  → ACCEPTED → IN_PRODUCTION → PUBLISHED → ARCHIVED

Gates:
  DESIGN → IRB_PENDING: methodology-complete gate
  IRB_PENDING → APPROVED: irb-approval gate (external, blocking)
  ANALYSIS → WRITING: results-validated gate
  INTERNAL_REVIEW → SUBMITTED: co-author-signoff gate
  REVISION_REQUESTED → RESUBMITTED: revision-complete gate
```

### What is MISSING from Primitives

- **Citation / Reference**: A first-class bibliographic reference entity (DOI, authors, journal, year) with bidirectional linking. Cross-Reference is too generic — it does not carry bibliographic metadata.
- **Version / Revision**: Manuscripts go through numbered revisions with tracked changes between them. Artifact versioning exists but needs explicit "revision response" semantics (mapping reviewer comments to changes made).
- **External Review**: The peer review cycle involves external actors whose identity may be unknown (double-blind). Actor primitive assumes known entities.
- **Embargo / Access Control**: Publications may be under embargo before release. No primitive for time-locked visibility.

### Roles

| Role | Responsibilities |
|---|---|
| **Principal Investigator (PI)** | Overall direction, funding, final approval on submissions, corresponding author |
| **Co-PI** | Shared leadership on multi-institution grants |
| **Postdoc** | Independent research execution, mentoring grad students, often lead author |
| **Graduate Student** | Primary data collection and analysis, thesis chapters as papers |
| **Lab Technician** | Protocol execution, equipment maintenance, sample preparation |
| **Research Assistant** | Data entry, literature searches, administrative support |
| **Statistician / Data Analyst** | Statistical design, power analysis, analysis validation |
| **Collaborator (external)** | Domain expertise, shared resources, co-authorship |
| **Peer Reviewer** | External, anonymous evaluation of manuscripts |
| **Journal Editor** | Manages review process, makes accept/reject decisions |

---

## 2. Lab-Based Research

### Core Concerns

1. **Protocols**: Step-by-step experimental procedures that must be followed exactly for reproducibility. Protocols are versioned — any change requires a new version with justification.
2. **Experiments**: Instances of protocol execution with specific parameters, samples, and results.
3. **Equipment Scheduling**: Shared instruments (microscopes, sequencers, spectrometers) need reservation systems with maintenance windows.
4. **Sample Tracking**: Chain of custody from collection through storage, processing, analysis, and disposal. Biological samples have expiry, storage conditions, and regulatory tracking requirements.
5. **Safety Compliance**: MSDS sheets, PPE requirements, waste disposal protocols, incident reporting, safety training records.
6. **Reagent/Supply Inventory**: Lot numbers, expiry dates, storage conditions, reorder triggers.

### Primitive Mapping

| Lab Concern | Primary Primitive | Supporting Primitives |
|---|---|---|
| Protocol | **Process** (type: protocol) | Artifact (protocol document), State Machine (protocol lifecycle), Gate (validation) |
| Experiment | **Work Item** (type: experiment) | Process (protocol ref), Artifact (raw data, notebook entry), Metric (measurements), Context |
| Equipment Scheduling | **Schedule** (type: equipment) | Constraint (availability windows), Scope (shared resources) |
| Sample Tracking | **Artifact** (type: sample) + **State Machine** | Category (sample type), Constraint (storage conditions), Cross-Reference (parent sample) |
| Safety Compliance | **Constraint** (type: safety) | Gate (safety-training-complete), Process (incident-reporting), Role (safety officer) |
| Reagent Inventory | **Artifact** (type: reagent) | Metric (quantity on hand), Constraint (expiry, storage), Event (reorder-trigger) |

### State Machine: Protocol Lifecycle

```
States: DRAFT → UNDER_REVIEW → VALIDATED → ACTIVE → SUPERSEDED → ARCHIVED

Transitions:
  DRAFT → UNDER_REVIEW: author submits for review
  UNDER_REVIEW → VALIDATED: reviewer approves (gate: successful test runs)
  VALIDATED → ACTIVE: PI approves for lab use
  ACTIVE → SUPERSEDED: new version validated
  ACTIVE → ARCHIVED: protocol no longer needed
```

### State Machine: Sample Lifecycle

```
States: COLLECTED → REGISTERED → IN_STORAGE → CHECKED_OUT → IN_PROCESSING
  → PROCESSED → ANALYZED → EXHAUSTED → DISPOSED

Constraints at each state:
  IN_STORAGE: temperature range, location logged
  CHECKED_OUT: who, when, purpose logged (chain of custody)
  DISPOSED: disposal method, date, compliance record
```

### What is MISSING from Primitives

- **Inventory / Physical Resource**: Artifacts are information objects. Samples and reagents are physical things with location, quantity, condition, and chain-of-custody. Need a **Physical Resource** primitive or an Artifact subtype with physical-world metadata (location, quantity, condition, lot number, expiry).
- **Instrument / Equipment**: A schedulable shared resource with maintenance state, calibration records, and usage logs. Schedule alone is insufficient — it does not model the resource itself.
- **Chain of Custody**: A specialized audit trail showing who had possession of what, when, and why. Event log captures this partially but lacks the structured provenance chain.
- **Safety Record**: Training certifications, incident reports, compliance audits. These are neither Work Items nor Artifacts in the current sense.

### Lab-Specific Roles

| Role | Responsibilities |
|---|---|
| **Lab Manager** | Equipment maintenance schedules, supply ordering, safety compliance |
| **Safety Officer** | Safety training, incident investigation, regulatory compliance |
| **Lab Technician** | Protocol execution, sample preparation, equipment operation |
| **Animal Care Technician** | (if applicable) Animal husbandry, IACUC compliance |

---

## 3. Computational Research

### Core Concerns

1. **Code Management**: Research code (analysis scripts, simulations, ML pipelines) must be versioned, documented, and reproducible.
2. **Data Management**: Raw data immutability, processed data lineage, storage tiers (hot/cold/archive).
3. **Experiment Tracking**: Hyperparameters, random seeds, environment specs, results — all logged for every run.
4. **Reproducibility**: Given the same code + data + environment, identical results must be obtainable.
5. **Compute Resource Scheduling**: HPC clusters, GPU allocations, cloud compute budgets.
6. **Environment Management**: Conda environments, Docker containers, dependency pinning.

### Primitive Mapping

| Concern | Primary Primitive | Supporting Primitives |
|---|---|---|
| Code Repository | **Artifact** (type: code-repo) | Cross-Reference (to paper, to data), State Machine (development lifecycle) |
| Analysis Script | **Artifact** (type: script) | Context (parameters, environment), Cross-Reference (input data, output figures) |
| Computational Experiment | **Work Item** (type: comp-experiment) | Metric (results), Context (hyperparameters), Constraint (compute budget), Artifact (logs) |
| Data Pipeline | **Process** (type: pipeline) | Artifact (input/output datasets), State Machine (pipeline run states) |
| Compute Scheduling | **Schedule** (type: compute) | Constraint (GPU hours, budget), Scope (cluster allocation) |
| Environment Spec | **Artifact** (type: environment) | Constraint (pinned versions), Context (platform requirements) |

### State Machine: Computational Experiment

```
States: DESIGNED → QUEUED → RUNNING → COMPLETED → FAILED → ANALYZING → REPORTED → ARCHIVED

Transitions:
  DESIGNED → QUEUED: submit to compute scheduler
  QUEUED → RUNNING: resources allocated
  RUNNING → COMPLETED: job finishes successfully
  RUNNING → FAILED: job errors (with error log artifact)
  COMPLETED → ANALYZING: results inspection begins
  ANALYZING → REPORTED: results incorporated into paper/report
  FAILED → DESIGNED: debug and redesign
```

### State Machine: Data Lifecycle

```
States: RAW → VALIDATED → CLEANED → PROCESSED → ANALYZED → PUBLISHED → ARCHIVED

Invariants:
  RAW: immutable, never modified after initial creation
  Each transition produces a new dataset (not in-place modification)
  Every transition logged with: script used, parameters, timestamp, operator
```

### What is MISSING from Primitives

- **Lineage / Provenance Graph**: Data provenance tracking — which script transformed which input into which output. This is a directed acyclic graph (DAG) of transformations. Cross-Reference is flat; provenance is hierarchical and directional.
- **Environment Specification**: A structured description of the computational environment (OS, language version, package versions, hardware). Not just an Artifact — it is an executable specification.
- **Compute Job**: A Work Item subtype that maps to an actual compute cluster job with resource requirements (CPUs, GPUs, memory, wall time), queue status, and cost.
- **Reproducibility Bundle**: A composite artifact linking code version + data version + environment spec + random seeds + results. The primitives individually exist but there is no "bundle" concept that enforces their co-presence.

### Computational Research Roles

| Role | Responsibilities |
|---|---|
| **Computational Scientist** | Algorithm design, implementation, experiment execution |
| **Research Software Engineer** | Code quality, testing, packaging, deployment |
| **Data Engineer** | Data pipelines, storage infrastructure, access management |
| **HPC Administrator** | Cluster management, resource allocation, user support |

---

## 4. Grant Management

### Core Concerns

1. **Proposal Writing**: Multi-section documents with strict formatting, page limits, and deadlines.
2. **Budget Planning**: Personnel costs, equipment, travel, indirect costs, multi-year projections.
3. **Compliance**: Funder-specific rules (NSF, NIH, ERC, UKRI each have different requirements).
4. **Milestone Tracking**: Deliverables promised in the proposal mapped to actual progress.
5. **Reporting**: Annual progress reports, financial reports, final reports — each with funder-specific formats.
6. **Subaward Management**: Budget and deliverable tracking for collaborating institutions.

### Primitive Mapping

| Concern | Primary Primitive | Supporting Primitives |
|---|---|---|
| Grant Proposal | **Artifact** (type: proposal) | State Machine (proposal lifecycle), Gate (internal review, submission), Schedule (deadline) |
| Budget | **Artifact** (type: budget) + **Metric** | Constraint (funder rules, overhead rates), Scope (budget categories) |
| Milestone | **Work Item** (type: milestone) | Gate (deliverable-complete), Schedule (due date), Metric (% complete) |
| Progress Report | **Artifact** (type: report) | Cross-Reference (to milestones, publications), Schedule (reporting period) |
| Subaward | **Scope** (type: subaward) | Constraint (budget allocation), Role (subaward PI), Metric (spending rate) |
| Award Period | **Schedule** (type: award-period) | Constraint (no-cost extension rules), Event (year-boundary) |

### State Machine: Grant Proposal Lifecycle

```
States: CONCEPT → DRAFTING → INTERNAL_REVIEW → BUDGET_APPROVED
  → INSTITUTIONAL_REVIEW → SUBMITTED → UNDER_REVIEW → SCORED
  → FUNDED → NOT_FUNDED → RESUBMISSION

Gates:
  DRAFTING → INTERNAL_REVIEW: draft-complete gate (all sections, budget, biosketches)
  INTERNAL_REVIEW → BUDGET_APPROVED: department/dean approval of budget
  BUDGET_APPROVED → INSTITUTIONAL_REVIEW: sponsored programs office review
  INSTITUTIONAL_REVIEW → SUBMITTED: institutional sign-off (often 5 business days before deadline)
```

### State Machine: Active Grant Lifecycle

```
States: AWARDED → ACTIVE → YEAR_N_REPORTING → NO_COST_EXTENSION
  → FINAL_REPORTING → CLOSED

Annual cycle within ACTIVE:
  ACTIVE → YEAR_N_REPORTING: annual report due
  YEAR_N_REPORTING → ACTIVE: report submitted, next year begins
```

### What is MISSING from Primitives

- **Budget / Financial Tracking**: Metric captures numbers but not structured financial data (budget categories, spending rates, burn-down by category, indirect cost calculations). Need a **Budget** primitive or a structured Metric subtype.
- **Funding Agency**: An external entity with specific rules, templates, and deadlines. Actor is close but lacks the concept of an external institution with imposed constraints.
- **Reporting Template**: Funder-specific document structures that must be followed exactly. Process is close but these are document-format specifications, not workflow specifications.
- **Multi-Year Timeline**: Schedule handles dates but grants have hierarchical time structures (award period > budget years > reporting periods > milestone deadlines).

### Grant Management Roles

| Role | Responsibilities |
|---|---|
| **PI** | Scientific direction, budget authority, report sign-off |
| **Grants Administrator** | Budget tracking, compliance, financial reporting |
| **Department Administrator** | Institutional approval, cost-share tracking |
| **Program Officer** | (funder side) Grant oversight, review coordination |

---

## 5. Collaboration

### Core Concerns

1. **Multi-Institution Coordination**: Different time zones, different institutional policies, different IRB boards.
2. **Authorship Tracking**: Contribution tracking for CRediT (Contributor Roles Taxonomy) — 14 defined roles from "Conceptualization" to "Writing — review & editing."
3. **Data Sharing Agreements (DSAs)**: Legal documents governing what data can be shared, how, and with whom.
4. **Material Transfer Agreements (MTAs)**: Legal documents for sharing biological materials between institutions.
5. **Intellectual Property**: Patent decisions, invention disclosures, licensing.
6. **Communication**: Regular meetings, shared documents, progress updates across institutions.

### Primitive Mapping

| Concern | Primary Primitive | Supporting Primitives |
|---|---|---|
| Multi-institution project | **Scope** (type: consortium) | Role (institution-level), Constraint (institutional policies), Schedule (coordination meetings) |
| Authorship | **Role** (type: author) + **Category** (CRediT taxonomy) | Actor (contributor), Metric (contribution level), Decision (authorship order) |
| Data Sharing Agreement | **Constraint** (type: DSA) | Gate (DSA-signed), Artifact (agreement document), Actor (institutions) |
| Material Transfer Agreement | **Constraint** (type: MTA) | Gate (MTA-executed), Artifact (agreement document) |
| IP Management | **Decision** (type: IP) | Constraint (funding agency IP rules), Artifact (invention disclosure), Gate (patent-filed) |
| Consortium Meetings | **Event** (type: meeting) | Schedule (cadence), Artifact (minutes), Discussion |

### What is MISSING from Primitives

- **Agreement / Contract**: A formal bilateral or multilateral legal document between parties with terms, expiry, and compliance tracking. Constraint is unilateral; an Agreement is mutual and has parties, terms, and signatures.
- **Contribution Record**: CRediT requires tracking 14 specific contribution types per person per paper. Role is too coarse — one person can have multiple contribution types on one work item.
- **Institution**: An organizational entity with policies, overhead rates, IRB processes, and legal standing. Actor is individual-level; Institution is organizational.
- **Communication Channel**: Ongoing asynchronous communication threads across institutions. Discussion is close but lacks the notion of persistent cross-organizational channels.

### Collaboration Roles

| Role | Responsibilities |
|---|---|
| **Consortium Lead** | Overall coordination, reporting to funder |
| **Site PI** | Leads work at one institution |
| **Data Manager** | Cross-site data harmonization, sharing compliance |
| **Project Coordinator** | Meeting scheduling, document management, timeline tracking |

---

## 6. Compliance

### Core Concerns

1. **IRB / Ethics Board**: Human subjects research requires ethics approval before any data collection. Amendments for protocol changes. Annual continuing review.
2. **IACUC**: Animal research equivalent of IRB.
3. **Data Privacy**: GDPR (EU), HIPAA (US health), FERPA (US education), state-level laws.
4. **Biosafety**: IBC (Institutional Biosafety Committee) for recombinant DNA, select agents.
5. **Export Control**: ITAR/EAR restrictions on sharing certain technologies/data internationally.
6. **Research Integrity**: Fabrication, falsification, plagiarism policies. Data retention requirements (typically 7-10 years post-publication).
7. **Conflict of Interest**: Financial disclosures, management plans.

### Primitive Mapping

| Concern | Primary Primitive | Supporting Primitives |
|---|---|---|
| IRB Protocol | **Artifact** (type: irb-protocol) + **State Machine** | Gate (irb-approval), Constraint (approved scope), Schedule (continuing review), Process (amendment) |
| Data Privacy | **Constraint** (type: data-privacy) | Category (data sensitivity level), Gate (privacy-impact-assessment), Process (data-handling) |
| Biosafety | **Constraint** (type: biosafety) | Gate (ibc-approval), Process (safety-protocol), Role (biosafety officer) |
| Research Integrity | **Constraint** (type: integrity) | Process (data-retention), Artifact (raw data archives), Gate (plagiarism-check) |
| Conflict of Interest | **Constraint** (type: COI) | Artifact (disclosure form), Schedule (annual renewal), Decision (management plan) |
| Training Requirements | **Gate** (type: training-complete) | Schedule (expiry/renewal), Role (trainee), Artifact (certificate) |

### State Machine: IRB Protocol

```
States: DRAFTING → SUBMITTED → UNDER_REVIEW → MODIFICATIONS_REQUIRED
  → APPROVED → ACTIVE → CONTINUING_REVIEW → AMENDMENT_SUBMITTED
  → AMENDMENT_APPROVED → EXPIRED → CLOSED

Key rules:
  APPROVED → ACTIVE: can begin data collection
  ACTIVE → CONTINUING_REVIEW: annual review required (Gate: annual deadline)
  ACTIVE → AMENDMENT_SUBMITTED: any protocol change requires amendment
  If continuing review lapses: ACTIVE → EXPIRED (all research must stop)
```

### State Machine: Data Sensitivity Classification

```
Levels: PUBLIC → INTERNAL → CONFIDENTIAL → RESTRICTED → HIGHLY_RESTRICTED

Each level adds constraints:
  PUBLIC: no restrictions
  INTERNAL: institutional access only
  CONFIDENTIAL: need-to-know, encryption at rest
  RESTRICTED: IRB-approved access list, audit logging, encrypted in transit + at rest
  HIGHLY_RESTRICTED: all of above + isolated storage, annual access review, breach notification plan
```

### What is MISSING from Primitives

- **Approval**: A formal authorization from an external body (IRB, IACUC, IBC) with: approval number, scope of approval, conditions, expiry date, and renewal requirements. Gate is a checkpoint; Approval is a persistent authorization that can expire or be revoked.
- **Certification / Training Record**: A person-held credential with issue date, expiry, and renewal requirements. This is neither a Role nor a Gate — it is a time-bounded qualification.
- **Audit Trail**: A tamper-evident log of all access to sensitive data. Event log is close but lacks the immutability guarantees and access-specific structure needed for compliance.
- **Retention Policy**: A rule specifying how long data/artifacts must be preserved and when/how they must be destroyed. Constraint is close but retention is a lifecycle rule, not a point-in-time constraint.

### Compliance Roles

| Role | Responsibilities |
|---|---|
| **IRB Coordinator** | Manages submissions, reviews, amendments |
| **Data Protection Officer** | GDPR compliance, data processing agreements |
| **Biosafety Officer** | IBC submissions, lab inspections |
| **Research Integrity Officer** | Misconduct investigations, training |
| **Export Control Officer** | Technology control plans, deemed export analysis |

---

## 7. Publication Pipeline

### Full Pipeline

```
Outline → First Draft → Co-author Review → Revision → PI Approval
→ Target Journal Selection → Format to Journal Style → Submission
→ Editor Desk Review → Sent to Reviewers → Reviews Received
→ Accept / Minor Revision / Major Revision / Reject
→ (if revision) Revision + Response Letter → Resubmission
→ (cycle until accepted or rejected)
→ Accepted → Proofs → Proof Corrections → Published → Post-publication erratum (if needed)
```

### Primitive Mapping

| Pipeline Stage | Primary Primitive | Supporting Primitives |
|---|---|---|
| Manuscript Draft | **Artifact** (type: manuscript, version: N) | Role (authors), Cross-Reference (to data, code, figures) |
| Co-author Review | **Event** (type: internal-review) | Actor (co-author), Discussion (comments), Decision (approve/revise) |
| Journal Selection | **Decision** (type: journal-selection) | Category (journal tier, field match), Constraint (open access requirements, funder mandates) |
| Formatting | **Work Item** (type: formatting) | Constraint (journal template), Artifact (formatted manuscript) |
| Submission | **Event** (type: submission) | Artifact (submission package), Schedule (expected timeline) |
| Peer Review | **Event** (type: peer-review) | Actor (anonymous reviewer), Artifact (review comments), Decision (editorial decision) |
| Response Letter | **Artifact** (type: response-letter) | Cross-Reference (reviewer comment → change made) |
| Proofs | **Artifact** (type: proofs) | Gate (proof-approved), Schedule (48-hour turnaround typical) |
| Published Paper | **Artifact** (type: published-paper) | Cross-Reference (DOI), Metric (citation count, altmetrics) |

### State Machine: Manuscript Lifecycle

```
States: OUTLINE → DRAFT_V1 → INTERNAL_REVIEW → DRAFT_VN → PI_APPROVED
  → FORMATTING → SUBMITTED → DESK_REVIEW → WITH_REVIEWERS
  → DECISION_RECEIVED → {ACCEPTED, MINOR_REVISION, MAJOR_REVISION, REJECTED}
  → REVISION_IN_PROGRESS → RESUBMITTED → (loop back to WITH_REVIEWERS)
  → ACCEPTED → PROOFS → PROOF_CORRECTIONS → IN_PRODUCTION → PUBLISHED

Special transitions:
  REJECTED → FORMATTING (retarget to different journal)
  PUBLISHED → ERRATUM (if errors discovered)
  PUBLISHED → RETRACTED (if serious issues)
```

### State Machine: Review Response

```
Per reviewer comment:
States: RECEIVED → TRIAGED → {WILL_ADDRESS, WILL_REBUT, NOT_APPLICABLE}
  → ADDRESSED → RESPONSE_WRITTEN → VERIFIED

The response letter maps every comment to a state and a specific manuscript change (or rebuttal).
```

### What is MISSING from Primitives

- **Submission Record**: Tracking which version was sent to which journal on what date, with what cover letter, and what the outcome was. This is a compound entity linking Artifact + Actor (journal) + Event (submission) + Decision (outcome).
- **Review Round**: A grouped set of reviews for one submission round. Not a single Event — it is a collection of reviews that together produce an editorial Decision.
- **Response Map**: A structured mapping from reviewer comments to manuscript changes. Cross-Reference handles 1:1 links but a response map is a structured table of comment-response pairs.
- **Journal Requirements**: Structured metadata about a target journal (word limits, figure limits, reference style, open access policy, review timeline). This is a specialized Context object.

### Publication Roles

| Role | Responsibilities |
|---|---|
| **Corresponding Author** | Manages submission, handles reviews, proof corrections |
| **First Author** | Primary writer, did most of the work |
| **Senior/Last Author** | PI, supervised the work |
| **Co-author** | Contributed to specific aspects, reviews drafts |
| **Journal Editor** | Manages review process |
| **Peer Reviewer** | Evaluates manuscript quality and validity |
| **Copy Editor** | (publisher side) Language and style corrections at proof stage |

---

## 8. Data Management

### Core Concerns

1. **Raw Data**: Immutable original data as collected. Must be preserved exactly.
2. **Processed Data**: Derived from raw data through documented transformations.
3. **Analysis Scripts**: Code that transforms raw → processed → results.
4. **Figures**: Generated from data, must be reproducible from scripts.
5. **Supplementary Materials**: Extended data, methods, code — linked to but not in main paper.
6. **Data Management Plan (DMP)**: Required by most funders. Describes what data will be produced, how stored, how shared, how preserved.
7. **FAIR Principles**: Findable, Accessible, Interoperable, Reusable — standard for research data.
8. **Data Repositories**: Zenodo, Dryad, Figshare, domain-specific (GenBank, PDB, ICPSR).

### Primitive Mapping

| Concern | Primary Primitive | Supporting Primitives |
|---|---|---|
| Raw Data | **Artifact** (type: raw-data, immutable: true) | Constraint (immutability), Context (collection conditions), Cross-Reference (to protocol) |
| Processed Data | **Artifact** (type: processed-data) | Cross-Reference (to raw data, to script), Context (processing parameters) |
| Analysis Script | **Artifact** (type: script) | Cross-Reference (input data → output), Context (environment spec) |
| Figures | **Artifact** (type: figure) | Cross-Reference (to generating script, to data), Category (main vs. supplementary) |
| Supplementary Materials | **Artifact** (type: supplement) | Cross-Reference (to main paper), Category (type: data/methods/code) |
| DMP | **Artifact** (type: data-management-plan) | Constraint (funder requirements), Schedule (review dates), Process (data handling) |
| Repository Deposit | **Event** (type: deposit) | Artifact (deposited dataset), Cross-Reference (DOI), Gate (embargo-lifted) |

### State Machine: Dataset Lifecycle

```
States: PLANNED → COLLECTING → COLLECTED → VALIDATED → CLEANING
  → CLEANED → ANALYZED → DEPOSITED → PUBLISHED → PRESERVED

Gates:
  COLLECTED → VALIDATED: quality-check gate (completeness, format, range checks)
  ANALYZED → DEPOSITED: de-identification gate (if human subjects)
  DEPOSITED → PUBLISHED: embargo-lifted gate (if applicable)

Invariant: RAW data artifacts are append-only; corrections create new versions, never modify in place.
```

### State Machine: FAIR Compliance

```
Per dataset, track four dimensions:
  Findable: {NO_PID, HAS_PID, INDEXED_IN_REGISTRY}
  Accessible: {NOT_ACCESSIBLE, RESTRICTED_ACCESS, OPEN_ACCESS}
  Interoperable: {PROPRIETARY_FORMAT, STANDARD_FORMAT, LINKED_METADATA}
  Reusable: {NO_LICENSE, RESTRICTIVE_LICENSE, OPEN_LICENSE, COMMUNITY_STANDARD}
```

### What is MISSING from Primitives

- **Dataset Manifest**: A structured description of a dataset including variables, units, collection method, sample sizes, and codebook. Artifact metadata is too generic.
- **Persistent Identifier (PID)**: DOIs, ORCIDs, RRIDs — globally unique identifiers that link to external registries. Cross-Reference handles links but not the specific semantics of persistent identifiers with resolver URLs.
- **Embargo**: A time-locked access restriction that automatically lifts on a specific date. No current primitive models time-triggered state transitions.
- **Data Lineage Graph**: Full provenance from raw collection through every transformation to published figure. Requires a DAG structure, not flat references.

### Data Management Roles

| Role | Responsibilities |
|---|---|
| **Data Manager** | DMP execution, repository deposits, metadata standards |
| **Data Steward** | Long-term preservation, access management, compliance |
| **Data Curator** | Metadata quality, format standardization, ontology alignment |
| **Research Librarian** | Repository selection, metadata standards, FAIR assessment |

---

## 9. Consolidated Gap Analysis: Missing Primitives

The following primitives are needed for scientific research but do NOT exist in the current set:

### 9.1 New Primitives Needed

| Proposed Primitive | Description | Used In |
|---|---|---|
| **Citation** | Bibliographic reference with DOI, authors, journal, year, bidirectional link to citing/cited works | Literature review, publication tracking, citation metrics |
| **Physical Resource** | A tangible item with location, quantity, condition, lot number, expiry, chain-of-custody | Samples, reagents, equipment, biological materials |
| **Instrument** | A shared schedulable resource with calibration records, maintenance state, usage logs | Lab equipment scheduling, usage tracking |
| **Agreement** | A bilateral/multilateral legal document with parties, terms, signatures, expiry, compliance tracking | DSAs, MTAs, subaward agreements, consortium agreements |
| **Approval** | A formal authorization from an external body with scope, conditions, expiry, and renewal requirements | IRB, IACUC, IBC, institutional sign-off |
| **Budget** | Structured financial tracking with categories, allocations, actuals, projections, and burn-down | Grant budgets, lab operational budgets |
| **Qualification** | A person-held credential with issuing body, issue date, expiry, and renewal requirements | Safety training, ethics certification, equipment authorization |
| **Lineage** | A directed acyclic graph of transformations showing provenance from source to derived products | Data provenance, computational reproducibility |
| **Bundle** | A composite entity that enforces co-presence of related artifacts (code + data + environment + results) | Reproducibility packages, submission packages, archival packages |
| **Embargo** | A time-locked access restriction with automatic lift date | Pre-publication data, patent-pending results |

### 9.2 Primitive Extensions Needed

These are not new primitives but extensions to existing ones:

| Existing Primitive | Extension Needed | Reason |
|---|---|---|
| **Actor** | Support for anonymous actors and institutional actors | Double-blind review, multi-institution collaboration |
| **Cross-Reference** | Support for external persistent identifiers (DOI, ORCID, RRID) with resolver URLs | Linking to external scholarly infrastructure |
| **State Machine** | Support for time-triggered transitions (not just event-triggered) | Embargo lifts, annual reviews, certification expiry |
| **Metric** | Support for structured multi-dimensional metrics with statistical metadata (CI, p-value, effect size) | Experimental results, statistical outputs |
| **Role** | Support for multiple fine-grained contribution types per person per work item (CRediT taxonomy) | Authorship attribution |
| **Schedule** | Support for hierarchical time structures (award period > budget year > reporting period) | Grant management timelines |
| **Constraint** | Support for constraint inheritance (institution-level constraints flow to all projects) | Institutional policies, funder requirements |

---

## 10. Complete Role Inventory

### Research-Specific Roles (Not Found in Software PM)

| Role | Category | Unique Aspects |
|---|---|---|
| **Principal Investigator** | Leadership | Legal responsibility for grant funds, academic freedom, tenure implications |
| **Co-PI** | Leadership | Shared authority, often at different institution |
| **Postdoctoral Researcher** | Execution | Fixed-term, high autonomy, career development needs |
| **Graduate Student** | Execution | Training component (thesis), evolving competence, committee oversight |
| **Lab Technician** | Execution | Protocol execution, equipment expertise, continuity across student generations |
| **Research Assistant** | Support | Often part-time/temporary, task-focused |
| **Statistician** | Specialist | Consulted at design and analysis phases, often shared across projects |
| **Research Librarian** | Specialist | Literature search, data management, repository selection |
| **Data Steward** | Specialist | Long-term data preservation, FAIR compliance |
| **Lab Manager** | Operations | Equipment, supplies, safety, onboarding |
| **Grants Administrator** | Operations | Budget tracking, compliance, reporting |
| **Safety Officer** | Compliance | Training, inspections, incident response |
| **IRB Coordinator** | Compliance | Protocol submission, amendment tracking |
| **Peer Reviewer** | External | Anonymous, episodic, quality evaluation |
| **Journal Editor** | External | Decision authority on publications |
| **Program Officer** | External | Funder representative, grant oversight |
| **Institutional Official** | External | Signs off on grants, assumes institutional responsibility |

---

## 11. Gap Analysis: Current "Research" Template vs. Scientific Research Needs

### What the Current Research Template Provides

Based on analysis of `/workspace/templates/research/`:

1. **PROGRESS.md** — Simple section-based completion tracking (not-started/in-progress/review/complete)
2. **experiments-README.md** — Experiment directory structure with comparison table (planned/in-progress/complete/abandoned, adopted/rejected/deferred)
3. **research-note.md** — Single research note template (objective/method/findings/conclusions)
4. **analysis/** — Empty directory for analysis artifacts
5. **research/** — Empty directory for research notes
6. **CLAUDE.md.template** — Basic project instructions

### What is Completely Missing

| Need | Gap Severity | Notes |
|---|---|---|
| **Full research lifecycle state machine** | CRITICAL | Current template has 4 statuses. Real research has 15+ states with gates between them |
| **Literature management** | CRITICAL | No citation tracking, no bibliography, no literature review structure |
| **Protocol management** | CRITICAL | No protocol versioning, validation, or compliance tracking |
| **IRB/ethics workflow** | CRITICAL | No concept of external approval gates that block data collection |
| **Grant management** | HIGH | No budget tracking, milestone tracking, or reporting cycle |
| **Publication pipeline** | HIGH | No manuscript lifecycle, no submission tracking, no review response management |
| **Sample/specimen tracking** | HIGH | No physical resource management |
| **Data management plan** | HIGH | No DMP template, no FAIR compliance tracking, no repository deposit workflow |
| **Collaboration structure** | HIGH | No multi-institution coordination, no authorship/contribution tracking |
| **Computational reproducibility** | MEDIUM | Experiments directory exists but no environment pinning, no data lineage, no compute scheduling |
| **Safety compliance** | MEDIUM | No safety training tracking, no incident reporting |
| **Equipment scheduling** | MEDIUM | No shared resource scheduling |
| **Conflict of interest** | LOW | No COI disclosure tracking |
| **Export control** | LOW | No technology control tracking |

### What Partially Exists but is Insufficient

| Feature | Current State | What is Needed |
|---|---|---|
| **Experiment tracking** | Directory structure + comparison table | Needs: protocol reference, parameter logging, statistical results, reproducibility metadata |
| **Progress tracking** | Section completion percentages | Needs: milestone-based tracking with grant deliverable alignment, Gantt-style dependencies |
| **Research notes** | Single-note template | Needs: typed notes (hypothesis, observation, meeting, literature review) with cross-linking |
| **Decision tracking** | Exists in product template (DECISIONS.md) but not in research template | Research template should include a decisions log for method choices, analysis decisions |

### Recommended Additions for a "Scientific Research" Process Template

**Tier 1 — Essential (must have for any research project):**
1. Research project lifecycle state machine (hypothesis through publication)
2. Literature review management with citation tracking
3. IRB/ethics approval workflow with blocking gates
4. Manuscript lifecycle state machine
5. Data management plan template
6. Role definitions (PI, postdoc, grad student, lab tech, collaborator)

**Tier 2 — Important (needed for lab-based or grant-funded research):**
1. Protocol management with versioning
2. Grant proposal and budget tracking
3. Sample/specimen tracking
4. Equipment scheduling
5. Safety compliance tracking
6. Multi-institution collaboration structure

**Tier 3 — Specialized (needed for computational or large-scale research):**
1. Computational experiment tracking with environment pinning
2. Data lineage/provenance graph
3. Compute resource scheduling
4. Reproducibility bundle management
5. FAIR compliance dashboard

---

## 12. Summary: Primitive Coverage Heat Map

How well the existing 17 primitives cover each research domain (1-5 scale):

| Domain | Coverage | Key Gap |
|---|---|---|
| Academic Workflow | 3/5 | Missing: Citation, Approval, Embargo |
| Lab Research | 2/5 | Missing: Physical Resource, Instrument, Chain of Custody |
| Computational | 3/5 | Missing: Lineage, Environment Spec, Compute Job |
| Grant Management | 2/5 | Missing: Budget, multi-year Schedule, Reporting Template |
| Collaboration | 2/5 | Missing: Agreement, Institution, Contribution Record |
| Compliance | 2/5 | Missing: Approval, Qualification, Audit Trail, Retention Policy |
| Publication Pipeline | 3/5 | Missing: Review Round, Response Map, Submission Record |
| Data Management | 3/5 | Missing: Dataset Manifest, Lineage, PID, Embargo |

**Overall assessment:** The existing 17 primitives provide approximately 40-50% coverage for scientific research project management. The primitives were designed for software product development and map well to the "work tracking" aspects of research (tasks, decisions, artifacts) but lack the domain-specific concepts that make research fundamentally different: external approvals, physical resources, bibliographic infrastructure, financial tracking, compliance workflows, and provenance chains.

The current "research" process template provides perhaps 10-15% of what a comprehensive scientific research PM system needs. It handles the most basic case — a solo researcher taking notes and tracking experiments — but has no support for the institutional, regulatory, collaborative, and publication-lifecycle aspects that define real scientific research.
