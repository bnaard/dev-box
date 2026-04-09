# Sector-Specific Process Structures Analysis

**Status:** Research complete
**Date:** 2026-03-27
**Relates to:** BACK-055 (process templates), process-ontology-primitives-2026-03.md, process-frameworks-research.md
**Purpose:** Identify what each professional sector needs from a project management tool beyond software-development defaults. Informs aibox process template library and potential sector-specific template packs.

---

## How to Read This Document

Each sector section covers seven dimensions:
1. Work items tracked
2. Roles
3. Compliance/regulatory constraints
4. Artifacts produced
5. Cadences/schedules
6. Gates/checkpoints
7. Key metrics

Each section ends with a **"What's UNIQUE"** box -- the things a professional in that sector would need that are NOT covered by generic project management primitives (backlogs, tasks, sprints, kanban boards, etc.).

The **universal primitives** identified in `process-ontology-primitives-2026-03.md` (Work Item, Log Entry, Decision Record, Artifact, Role, Process, State Machine, Checkpoint, Metric, Category, Scope, Schedule, Cross-reference, Constraint, Context) apply everywhere. This document focuses on what each sector needs ON TOP of those primitives.

---

## 1. Software Development (Agile, DevOps)

This is the baseline. All other sectors are compared against this.

### 1.1 Work Items
- Epic, Feature, User Story, Task, Bug, Tech Debt, Spike/Research
- Enablers (infrastructure work that enables future features)
- Incidents (production issues, severity-classified)

### 1.2 Roles
- Product Owner, Scrum Master, Developer, QA Engineer, DevOps/SRE, Tech Lead, Engineering Manager, Architect

### 1.3 Compliance/Regulatory
- Minimal in most contexts (SOC 2, ISO 27001 for enterprise)
- GDPR/privacy for data-handling features
- Accessibility standards (WCAG) for user-facing products
- Export controls for cryptography

### 1.4 Artifacts
- Source code, PRs, builds, releases, documentation, API specs, test reports, architecture decision records, runbooks, postmortems

### 1.5 Cadences
- Sprint (1-4 weeks), PI (8-12 weeks), release cycles, daily standups, weekly planning, quarterly roadmap review

### 1.6 Gates
- Code review approval, CI pipeline pass, staging deployment verification, security scan, performance benchmarks, release sign-off

### 1.7 Metrics
- Velocity, cycle time, lead time, deployment frequency, MTTR, change failure rate, test coverage, defect escape rate, WIP limits

---

## 2. Scientific Research (Academic, Lab-Based, Computational)

### 2.1 Work Items
- **Experiment** (the fundamental unit -- hypothesis, protocol, execution, analysis)
- **Literature review task** (systematic search, screening, extraction)
- Grant proposal / funding application
- Paper draft / manuscript
- Data collection campaign
- Instrument calibration task
- Peer review response
- IRB/ethics application
- Conference submission

### 2.2 Roles
- Principal Investigator (PI), Co-PI, Postdoctoral Researcher, Graduate Student, Lab Technician, Research Assistant, Statistician, Data Manager, IRB Officer, Department Chair, Journal Editor, Peer Reviewer

### 2.3 Compliance/Regulatory
- **IRB/Ethics board approval** (mandatory for human subjects research)
- **IACUC approval** (animal research)
- **Biosafety committee** (BSL levels for pathogen work)
- **Export controls** (ITAR, EAR for dual-use research)
- **Data management mandates** (NIH, NSF, EU Horizon require data management plans)
- **Reproducibility requirements** (pre-registration, open data, open code)
- **Conflict of interest disclosure**
- **Responsible conduct of research** (RCR training requirements)

### 2.4 Artifacts
- Lab notebooks (legally significant, timestamped, witnessed)
- Datasets (raw, processed, archived -- with metadata schemas like Dublin Core)
- Statistical analysis scripts and outputs
- Manuscripts (with tracked revision history per journal submission)
- Supplementary materials
- Pre-registration documents (ClinicalTrials.gov, OSF)
- Grant proposals and progress reports
- Poster presentations, slide decks
- Protocols (step-by-step experimental procedures)

### 2.5 Cadences
- **Grant cycles** (annual: NIH has 3 cycles/year, NSF varies by program)
- **Academic calendar** (semester/quarter boundaries, thesis deadlines)
- **Lab meetings** (weekly)
- **Journal submission cycles** (no fixed cadence -- deadline-driven for conferences, rolling for journals)
- **Progress reports** (annual or semi-annual to funders)
- **Thesis committee meetings** (6-12 month intervals)

### 2.6 Gates
- **Ethics/IRB approval** before any data collection
- **Pre-registration** before experiments begin
- **Data quality check** before analysis
- **Statistical review** before conclusions
- **Internal review** before submission
- **Peer review** (external, journal-managed)
- **Thesis defense** (oral examination)
- **Grant review panels** (funding decisions)

### 2.7 Metrics
- Publications (count, impact factor, citation count, h-index)
- Grant funding secured (dollars, success rate)
- Time from experiment to publication
- Reproducibility rate
- Student graduation rate and time-to-degree
- Data sharing compliance rate
- Conference presentations

### 2.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Experiment-as-work-item** with hypothesis/method/result structure | Software tasks don't have hypothesis-result pairs; experiments have a fundamentally different shape than user stories |
| **Lab notebook with legal witness requirements** | No analog in software; some institutions require physical countersignature or certified digital timestamps |
| **Pre-registration before execution** | Software has no concept of committing to a methodology before seeing results -- this is an anti-bias mechanism |
| **Multi-year timelines with funding cliffs** | Grants expire; work must align to funding periods that span 3-5 years |
| **Non-linear workflow** | Experiments fail and redirect; the "backlog" is more of a hypothesis tree than a prioritized list |
| **Citation and provenance tracking** | Every claim must trace to data, every dataset to a protocol, every protocol to an approval |
| **Dual submission tracking** | Submitting the same work to multiple venues has complex ethical rules |
| **Equipment/resource booking** | Shared instruments (NMR, electron microscope, compute cluster) need scheduling |

---

## 3. Financial Services (Banking, Insurance, Trading)

### 3.1 Work Items
- **Regulatory change implementation** (new rule from SEC, OCC, FINRA, FCA, etc.)
- **Model validation task** (validating risk models, pricing models)
- **Audit finding remediation**
- **Client onboarding/KYC case**
- **Trade processing exception**
- **Credit application/underwriting case**
- **Insurance claim**
- **Compliance review**
- **Product change request** (new financial product or modification)

### 3.2 Roles
- Portfolio Manager, Trader, Risk Analyst, Compliance Officer, Auditor (internal/external), Model Validator, Underwriter, Claims Adjuster, Actuary, Relationship Manager, Chief Risk Officer, Regulatory Affairs, Anti-Money Laundering (AML) Analyst, Data Governance Officer

### 3.3 Compliance/Regulatory
- **Basel III/IV** (capital adequacy, liquidity requirements)
- **Solvency II** (insurance capital requirements)
- **Dodd-Frank / MiFID II** (trading transparency, reporting)
- **SOX** (financial reporting controls)
- **GDPR/CCPA** (customer data protection)
- **BSA/AML** (anti-money laundering, Know Your Customer)
- **PCI DSS** (payment card data)
- **Model Risk Management (SR 11-7/SS 1/23)** (model validation requirements)
- **DORA** (Digital Operational Resilience Act -- EU, effective 2025)
- **Three Lines of Defense model** (1st: business, 2nd: risk/compliance, 3rd: audit)
- **Mandatory record retention** (7+ years for most records)

### 3.4 Artifacts
- Risk assessment reports, model validation documents, audit reports, regulatory filings, compliance attestations, trade confirmations, client suitability assessments, policy documents, actuarial reports, stress test results, board presentations, incident reports, data lineage documentation

### 3.5 Cadences
- **Daily:** Trade reconciliation, risk reporting, P&L reporting
- **Monthly:** Regulatory reporting (many jurisdictions), risk committee meetings
- **Quarterly:** Earnings reporting, board risk reviews, regulatory examinations
- **Annual:** Annual report, external audit, stress testing (CCAR/DFAST), model revalidation
- **Ad-hoc:** Regulatory examination responses (tight deadlines, often 2-4 weeks)

### 3.6 Gates
- **Four-eyes principle** (dual approval for trades, transfers, model changes)
- **Model validation sign-off** (independent review before production use)
- **Compliance review** before product launch
- **Risk assessment** before new business activity
- **Audit committee approval** for material changes
- **Change Advisory Board** for IT changes (ITIL-style)
- **Regulatory pre-notification** for certain activities

### 3.7 Metrics
- Risk-adjusted return, Value at Risk (VaR), capital adequacy ratios, loss ratios, combined ratio (insurance), trade error rate, SLA compliance for settlement, KYC completion time, audit finding closure rate, regulatory finding aging, model performance (back-testing accuracy)

### 3.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Three Lines of Defense model** | Work items have mandatory review chains: business owner -> risk/compliance -> audit. Not just "reviewer" but legally distinct defense lines |
| **Four-eyes / dual control** | Certain actions MUST have two independent approvers -- not optional code review, but legally mandated dual sign-off |
| **Regulatory calendar as primary driver** | Deadlines are externally imposed and non-negotiable; the backlog is often driven by regulatory change, not user needs |
| **Audit trail with legal retention** | Every action, decision, and approval must be retained for 7+ years with tamper-evident logging |
| **Model lifecycle management** | Financial models (risk, pricing, credit scoring) have their own lifecycle: develop -> validate -> approve -> monitor -> revalidate -> retire |
| **Segregation of duties** | The person who builds something CANNOT approve it; role assignments have hard constraints |
| **Materiality thresholds** | Work items have financial impact classifications that determine approval authority levels |
| **Regulatory examination management** | Ad-hoc projects created by regulator requests with non-negotiable deadlines and specific deliverable formats |

---

## 4. Healthcare (Clinical, Pharmaceutical, Health IT)

### 4.1 Work Items
- **Clinical order** (lab test, imaging, medication, procedure)
- **Patient case / encounter**
- **Clinical trial protocol task** (site setup, patient enrollment, data monitoring)
- **Regulatory submission** (FDA 510(k), NDA, BLA, CE marking)
- **CAPA** (Corrective and Preventive Action)
- **Change control** (validated system changes)
- **Quality event / deviation report**
- **Patient safety report** (adverse event, near miss)

### 4.2 Roles
- Physician/Surgeon, Nurse, Pharmacist, Clinical Research Coordinator, Data Safety Monitoring Board (DSMB), Quality Manager, Regulatory Affairs Specialist, Clinical Data Manager, Biostatistician, Principal Investigator (clinical trials), IRB/Ethics Committee, Patient/Subject

### 4.3 Compliance/Regulatory
- **FDA 21 CFR Part 11** (electronic records, electronic signatures)
- **EU MDR / IVDR** (medical device regulation)
- **GxP** (Good Clinical Practice, Good Manufacturing Practice, Good Laboratory Practice, Good Distribution Practice)
- **HIPAA** (patient data privacy and security)
- **Clinical trial regulations** (ICH-GCP E6(R3), informed consent requirements)
- **ISO 13485** (medical device quality management)
- **ISO 14971** (risk management for medical devices)
- **Computer System Validation (CSV)** / Computer Software Assurance (CSA)
- **Pharmacovigilance regulations** (mandatory adverse event reporting timelines)
- **CLIA** (lab certification)

### 4.4 Artifacts
- Clinical protocols, informed consent forms, case report forms (CRFs), statistical analysis plans, clinical study reports, regulatory submission dossiers (eCTD format), design history files, risk management files (ISO 14971), validation protocols and reports, SOPs, training records, batch records, device master records, labeling, IFU (instructions for use)

### 4.5 Cadences
- **Real-time:** Adverse event reporting (serious: 24 hours, fatal: immediate)
- **Per-visit:** Patient encounter documentation
- **Monthly:** Data monitoring committee reviews, quality metrics
- **Quarterly:** Regulatory progress reports, DSMB meetings
- **Annual:** IND annual reports, quality management reviews, device registration renewals
- **Phase-driven:** Clinical trials have phases (I, II, III, IV) spanning 2-15 years

### 4.6 Gates
- **IRB/Ethics approval** before any patient contact
- **FDA IND/IDE approval** before clinical investigation
- **DSMB safety review** at interim analysis points
- **Design review** (at each design phase per ISO 13485)
- **Validation protocol approval** before execution
- **Regulatory submission readiness review**
- **Release to market** (requires documented evidence package)
- **Change control board** for validated system changes

### 4.7 Metrics
- Patient enrollment rate, protocol deviation rate, adverse event rate, query resolution time, data completion rate, time to regulatory approval, CAPA closure time, audit finding trends, device complaint rate, manufacturing yield, process capability (Cpk)

### 4.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Patient safety as overriding constraint** | Work can be halted at ANY point if safety signal detected; no concept of "must ship by Friday" when lives are at stake |
| **21 CFR Part 11 / electronic signatures** | Electronic records must have validated systems, audit trails, and legally binding e-signatures -- not just "git log" |
| **Validated system changes** | Any change to a production system requires pre-approved protocols, IQ/OQ/PQ testing, and documented evidence. A "hotfix" is essentially impossible |
| **Design History File** | Medical devices require a complete traceable record from user needs -> design inputs -> design outputs -> verification/validation |
| **Regulatory submission as a deliverable** | The submission dossier (eCTD) has a rigid format mandated by regulators; it IS the product in a regulatory sense |
| **Mandatory timelines for safety reporting** | Serious adverse events: 15 days. Fatal: 7 days. These are legal obligations, not SLAs |
| **Blinding and randomization** | Clinical trials require that certain data is hidden from certain roles -- role-based access control with enforced information barriers |
| **Phase transitions as multi-year gates** | Phase I -> II -> III -> IV, each requiring regulatory approval and spanning years |

---

## 5. Legal (Law Firms, Compliance, Contracts)

### 5.1 Work Items
- **Matter** (the fundamental unit -- a legal engagement or case)
- **Task** (research, drafting, review, filing)
- **Filing / motion** (court submission with deadline)
- **Discovery request / response**
- **Contract review / negotiation cycle**
- **Compliance assessment**
- **Regulatory filing**
- **Due diligence item** (M&A, real estate)

### 5.2 Roles
- Partner, Associate, Paralegal, Legal Secretary, Client, Opposing Counsel, Judge, Clerk of Court, Expert Witness, Mediator/Arbitrator, Compliance Officer, General Counsel, Contract Manager, Legal Operations Manager

### 5.3 Compliance/Regulatory
- **Attorney-client privilege** (communications must be tracked and protected)
- **Conflict of interest checks** (mandatory before accepting new matters)
- **Court filing deadlines** (jurisdictional rules, often non-extendable)
- **Statute of limitations** (absolute deadlines for bringing claims)
- **Legal hold / litigation hold** (must preserve ALL potentially relevant documents)
- **Bar association ethics rules** (competence, confidentiality, conflicts)
- **Trust accounting rules** (client funds must be segregated)
- **e-Discovery rules** (FRCP, proportionality requirements)
- **Regulatory compliance calendars** (filing deadlines vary by jurisdiction)

### 5.4 Artifacts
- Legal briefs, motions, complaints, answers, discovery requests/responses, depositions (transcripts), contracts (with redline versions), legal opinions, memoranda, court orders, settlement agreements, compliance reports, corporate filings, board resolutions, regulatory submissions

### 5.5 Cadences
- **Court-driven:** Scheduling orders set deadlines for discovery, motions, trial
- **Statute-driven:** Limitation periods, filing deadlines, regulatory calendars
- **Transaction-driven:** M&A deals have closing dates; contract negotiations have rounds
- **Billing cycles:** Monthly billing with detailed time entries
- **Regular:** Weekly case status reviews, monthly client reporting

### 5.6 Gates
- **Conflict check** before engagement
- **Partner review** before filing
- **Client approval** before major filings or settlement
- **Privilege review** before document production in discovery
- **Signature / execution** of contracts (often with notarization or witnessing)
- **Court approval** of settlements
- **Regulatory clearance** for transactions (antitrust, CFIUS)

### 5.7 Metrics
- Billable hours, realization rate (billed vs. collected), matter profitability, time to resolution, court deadline compliance, contract cycle time, discovery volume and cost, client satisfaction, win/loss rate, regulatory finding rate

### 5.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Time tracking as a primary function** | Billable hours are the revenue model; every task needs 6-minute increment time entries linked to matter codes |
| **Conflict of interest engine** | Before ANY work begins, the system must check all parties against all existing and past clients/matters |
| **Court-imposed scheduling** | Deadlines are set by judges/courts, not the team; missing a filing deadline can mean losing a case |
| **Statute of limitations tracking** | Absolute drop-dead dates that, if missed, permanently extinguish legal rights |
| **Privilege classification** | Documents must be tagged as privileged/non-privileged; producing a privileged document in discovery can waive privilege permanently |
| **Redline / version comparison** | Contract negotiation requires side-by-side comparison of every change between drafts with attribution |
| **Matter-centric (not project-centric) organization** | Everything revolves around the "matter" -- a client engagement that may span years and multiple sub-projects |
| **Ethical walls / information barriers** | Certain team members MUST be blocked from certain matters to prevent conflicts; enforced access control |
| **Jurisdictional rules engine** | Filing requirements, deadlines, and procedures vary by jurisdiction (state, federal, international) |

---

## 6. Manufacturing (Production, Quality, Supply Chain)

### 6.1 Work Items
- **Work order / production order** (make X units of product Y)
- **Quality inspection** (incoming, in-process, final)
- **Engineering Change Order (ECO)** / Engineering Change Notice (ECN)
- **Non-conformance report (NCR)**
- **CAPA** (Corrective and Preventive Action)
- **Maintenance work order** (preventive, corrective)
- **Purchase order**
- **Inventory adjustment**

### 6.2 Roles
- Plant Manager, Production Supervisor, Operator, Quality Inspector, Quality Manager, Process Engineer, Manufacturing Engineer, Maintenance Technician, Supply Chain Planner, Purchasing Agent, Warehouse Manager, EHS (Environment Health Safety) Officer

### 6.3 Compliance/Regulatory
- **ISO 9001** (quality management systems)
- **ISO 14001** (environmental management)
- **ISO 45001** (occupational health and safety)
- **IATF 16949** (automotive quality)
- **AS9100** (aerospace quality)
- **GMP** (Good Manufacturing Practice -- pharma, food, cosmetics)
- **OSHA regulations** (workplace safety)
- **EPA regulations** (environmental compliance)
- **RoHS, REACH** (hazardous substances in products)
- **UL, CE, FCC** (product safety and emissions certifications)
- **Customs and trade compliance** (tariffs, country of origin, export controls)

### 6.4 Artifacts
- Bills of Material (BOM), routing sheets, work instructions, control plans, inspection records, test certificates, certificates of conformance, SPC (Statistical Process Control) charts, FMEA (Failure Mode and Effects Analysis) documents, PPAP (Production Part Approval Process) packages, first article inspection reports, material safety data sheets (SDS), calibration records, training records

### 6.5 Cadences
- **Per-shift:** Production reporting, quality checks
- **Daily:** Production scheduling, scrap/yield review, safety briefings
- **Weekly:** Production planning, quality review, maintenance scheduling
- **Monthly:** OEE review, supplier scorecard, inventory review
- **Quarterly:** Management review, customer quality reviews
- **Annual:** ISO surveillance audits, strategic planning, capital budgeting

### 6.6 Gates
- **First Article Inspection (FAI)** before production run
- **PPAP approval** before shipping to automotive customers
- **Design review** at each development phase
- **Process validation** (IQ/OQ/PQ) before production
- **Material incoming inspection**
- **Final inspection / release** before shipment
- **Engineering change board approval**
- **Customer source inspection** (for critical components)

### 6.7 Metrics
- OEE (Overall Equipment Effectiveness), yield/scrap rate, on-time delivery, cycle time, takt time, first pass yield, defects per million opportunities (DPMO), cost per unit, inventory turns, supplier quality (PPM defect rate), MTBF/MTTR (equipment), safety incident rate (TRIR)

### 6.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Bill of Materials (BOM) as central artifact** | Hierarchical product structure (assemblies -> sub-assemblies -> parts -> raw materials) with revision control; no software analog |
| **Physical resource tracking** | Machines, tools, materials, floor space -- physical constraints that don't exist in software |
| **Statistical Process Control (SPC)** | Real-time charting of process measurements against control limits; automated out-of-control detection |
| **Takt time / line balancing** | Work is paced by customer demand rate; tasks must be balanced across workstations to match takt |
| **Lot/batch traceability** | Every unit must trace back to material lots, operators, machines, and timestamps for recall capability |
| **Engineering Change Order workflow** | Changes to products require formal impact analysis across BOM, tooling, documentation, customer notification |
| **Supplier management as a core workflow** | Qualifying, auditing, scoring, and managing suppliers is a major process stream |
| **Physical inspection with measurement data** | Quality checks produce dimensional measurements, not pass/fail -- data feeds SPC charts |

---

## 7. Construction / Engineering

### 7.1 Work Items
- **RFI (Request for Information)** -- questions between parties
- **Submittal** (material/equipment approval before procurement)
- **Change order** (scope/cost/schedule change)
- **Punch list item** (deficiency to fix before acceptance)
- **Inspection request**
- **Pay application** (contractor payment request)
- **Safety observation / incident report**
- **Drawing revision**

### 7.2 Roles
- Owner/Client, Architect, General Contractor, Subcontractor, Project Manager (owner-side and contractor-side), Superintendent, Project Engineer, Estimator, Inspector, Safety Officer, Building Official, Design Engineer, Commissioning Agent

### 7.3 Compliance/Regulatory
- **Building codes** (IBC, local amendments -- vary by jurisdiction)
- **Zoning and permitting** (land use, environmental impact)
- **OSHA (construction-specific)** (fall protection, trenching, scaffolding)
- **Environmental regulations** (stormwater, demolition waste, asbestos)
- **ADA compliance** (accessibility)
- **Fire codes** (NFPA)
- **Prevailing wage / Davis-Bacon** (for government projects)
- **Bonding and insurance requirements**
- **Professional licensing** (PE stamps, architect seals required on drawings)

### 7.4 Artifacts
- Drawings (plans, elevations, sections, details -- with revision clouds), specifications (CSI MasterFormat divisions), submittals, RFI logs, daily reports, inspection reports, as-built drawings, O&M manuals, commissioning reports, cost estimates, schedules (CPM/Gantt), pay applications with schedule of values, lien waivers, warranties

### 7.5 Cadences
- **Daily:** Daily reports, toolbox safety talks
- **Weekly:** OAC (Owner-Architect-Contractor) meetings, schedule updates, subcontractor coordination
- **Monthly:** Pay applications, schedule narrative, progress photos
- **Phase-based:** Design phases (schematic -> design development -> construction documents -> bidding -> construction -> closeout)
- **Milestone-driven:** Foundation, structure, envelope, MEP rough-in, finishes, substantial completion, final completion

### 7.6 Gates
- **Permitting** (building permit before construction)
- **Design phase approvals** (owner sign-off at each design phase)
- **Submittal approval** (architect approval before material procurement)
- **Inspection approvals** (building inspector sign-off at defined milestones)
- **Substantial completion** (the building is usable; punch list may remain)
- **Final completion / closeout** (all punch list items resolved, all documentation submitted)
- **Commissioning acceptance** (systems perform per specification)
- **Certificate of Occupancy** (government approval to use the building)

### 7.7 Metrics
- Schedule variance (CPM-based, not agile-based), cost variance (budget vs. actual by cost code), RFI response time, submittal turnaround time, change order volume and cost, safety incident rate (EMR), punch list closure rate, subcontractor performance ratings

### 7.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Critical Path Method (CPM) scheduling** | Construction uses predecessor/successor dependency chains with float calculation, not sprints or kanban |
| **Multi-party contractual structure** | Owner, architect, GC, subcontractors all have separate contracts and different views of the same project |
| **RFI / Submittal workflows** | Formal question-and-answer and approval processes between separate companies, with contractual time limits |
| **Cost code / WBS structure** | CSI MasterFormat 50-division structure for organizing ALL construction knowledge and costs |
| **Drawing-centric collaboration** | Drawings are the primary deliverable; markup, revision clouding, and drawing set management are core functions |
| **Weather and site condition dependencies** | Schedules depend on external physical conditions that software projects never encounter |
| **Pay application / earned value by trade** | Monthly payment requests mapped against a schedule of values by CSI division |
| **Retainage** | A percentage of each payment is withheld until project completion -- a financial mechanism with no software analog |
| **Lien rights and mechanics liens** | Legal claims against property if contractors are not paid; time-sensitive legal filings |

---

## 8. Education (Course Development, Curriculum Design)

### 8.1 Work Items
- **Course / module development task**
- **Learning objective** (the fundamental unit of design -- what students will be able to DO)
- **Assessment item** (quiz, exam, assignment, rubric)
- **Content creation** (lecture, video, reading, interactive element)
- **Curriculum review / accreditation task**
- **Student case / advising task** (for student services)
- **Research project** (student or faculty)

### 8.2 Roles
- Instructional Designer, Subject Matter Expert (SME), Faculty/Instructor, Curriculum Committee, Program Director, Department Chair, Dean, Academic Advisor, Learning Technologist, Assessment Specialist, Accreditation Coordinator, Teaching Assistant, Librarian

### 8.3 Compliance/Regulatory
- **Accreditation standards** (regional accreditors, discipline-specific like ABET for engineering, AACSB for business)
- **FERPA** (student privacy)
- **ADA / Section 508** (accessibility of learning materials)
- **Title IX** (non-discrimination)
- **State education regulations** (vary by state/country)
- **Credit hour regulations** (Carnegie Unit, seat time vs. competency)
- **Transfer articulation agreements**

### 8.4 Artifacts
- Syllabi, course outlines, learning objectives mapped to program outcomes, assessment rubrics, course materials (slides, videos, readings), grade books, program self-study documents (for accreditation), curriculum maps, student learning outcome reports, course evaluation results, faculty development plans

### 8.5 Cadences
- **Academic calendar:** Semester/quarter/trimester cycles
- **Course development:** Typically 6-12 months before first offering
- **Curriculum review:** 3-5 year program review cycles
- **Accreditation:** 5-10 year cycles with interim reports
- **Assessment:** Annual assessment of program learning outcomes
- **Weekly:** Teaching schedule, office hours

### 8.6 Gates
- **Curriculum committee approval** for new courses or changes
- **Department review** of course design
- **Accessibility review** of materials
- **Accreditation self-study** submission and site visit
- **Program review** at defined intervals
- **Student evaluation threshold** (courses below threshold trigger review)

### 8.7 Metrics
- Student learning outcomes achievement rates, course completion rates, student satisfaction scores, graduation rates, retention rates, time to degree, faculty-student ratio, credit hour production, assessment score distributions, employer satisfaction (for professional programs), licensure exam pass rates

### 8.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Learning objective alignment** | Every piece of content and every assessment must map to specific learning objectives, which map to program outcomes, which map to accreditation standards -- a multi-level alignment matrix |
| **Academic calendar as immovable constraint** | Course delivery dates are fixed years in advance; you cannot "slip" a semester |
| **Bloom's taxonomy as a design framework** | Learning objectives are classified by cognitive level (remember, understand, apply, analyze, evaluate, create) -- this drives assessment design |
| **Backward design (Understanding by Design)** | Start with desired outcomes, then design assessments, then design learning activities -- the opposite of incremental feature development |
| **Accreditation evidence collection** | Continuous collection of artifacts that prove students are achieving outcomes, compiled into massive self-study documents every 5-10 years |
| **Faculty governance** | Curriculum changes require committee votes, not management decisions -- consensus-driven and slow by design |
| **Rubric-based assessment** | Multi-dimensional scoring guides that define what "proficient" looks like for each learning outcome |
| **Accessibility as a mandatory requirement** | All materials must be accessible (captioned videos, screen-reader-compatible documents, etc.) -- not a nice-to-have |

---

## 9. Media / Content Creation (Journalism, Marketing, Video Production)

### 9.1 Work Items
- **Story / article** (pitch, draft, edit, publish)
- **Campaign** (multi-channel coordinated effort)
- **Asset** (photo, video, graphic, audio clip)
- **Editorial calendar entry**
- **Brand guideline item**
- **Social media post**
- **Video production task** (pre-production, production, post-production)
- **Content brief**

### 9.2 Roles
- Editor-in-Chief, Managing Editor, Reporter/Writer, Photographer, Videographer, Graphic Designer, Content Strategist, Social Media Manager, Copy Editor, Fact Checker, Producer, Director, Sound Engineer, Marketing Manager, Brand Manager, SEO Specialist, Creative Director

### 9.3 Compliance/Regulatory
- **Defamation / libel law**
- **Copyright and fair use**
- **FTC disclosure requirements** (sponsored content, endorsements)
- **Press freedom protections** (shield laws, source protection)
- **Privacy laws** (right to be forgotten, consent for imagery)
- **Advertising standards** (ASA, NAD -- truthfulness, substantiation)
- **Music licensing** (ASCAP, BMI, sync licenses for video)
- **Broadcast regulations** (FCC for broadcast; less for digital)
- **GDPR** (consent for marketing communications, cookie compliance)

### 9.4 Artifacts
- Published articles, video packages (raw footage, rough cut, fine cut, final), graphics and designs (with source files), editorial calendars, brand guidelines, style guides, media kits, analytics reports, content briefs, storyboards, scripts, shot lists, social media calendars, press releases

### 9.5 Cadences
- **Publication schedule:** Daily (news), weekly (features), monthly (magazines), quarterly (reports)
- **Campaign cycles:** Planning (4-8 weeks), execution (2-12 weeks), analysis (1-2 weeks)
- **Editorial meetings:** Daily (newsrooms), weekly (content teams)
- **Content calendar:** Monthly or quarterly planning
- **Real-time:** Breaking news has no cadence -- immediate response
- **Production schedule:** Pre-production -> shoot days -> post-production (varies by project)

### 9.6 Gates
- **Editorial review / approval** before publication
- **Legal review** for sensitive content
- **Fact-checking** before publication
- **Brand review** for marketing materials
- **Client approval** for agency work (often multiple rounds)
- **Rough cut / fine cut approval** for video
- **Final proof / sign-off** before print

### 9.7 Metrics
- Pageviews, unique visitors, engagement rate, time on page, social shares, conversion rate, SEO rankings, subscriber growth, audience reach, ad revenue, production cost per asset, turnaround time (pitch to publish), content freshness, brand consistency score

### 9.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Editorial calendar as the primary planning tool** | Content is planned around publication dates, themes, and audience segments -- not sprints or features |
| **Multi-stage creative review** | Content goes through ideation -> draft -> editorial review -> legal review -> design -> final approval, with subjective quality judgments at each stage |
| **Asset management with version variants** | A single photo might need 10 crops/sizes for different channels; video has proxy, rough cut, fine cut, master |
| **Embargo and publish scheduling** | Content is prepared but must not be visible until a specific date/time |
| **Source protection and confidentiality** | Journalism requires protecting source identity -- access control with legal backing |
| **Rights and licensing management** | Every asset has usage rights (owned, licensed, stock, user-generated) with expiration dates and territory restrictions |
| **Real-time / breaking workflow** | Some work has zero planning time -- newsrooms need immediate-response processes alongside planned editorial |
| **Multi-channel adaptation** | One piece of content spawns multiple derivatives for different platforms (web, social, email, print) |

---

## 10. Consulting / Professional Services

### 10.1 Work Items
- **Engagement / project** (the billable unit)
- **Deliverable** (report, analysis, presentation, recommendation)
- **Workstream** (parallel tracks within an engagement)
- **Finding / recommendation** (the atomic output of analysis)
- **Proposal / bid**
- **Staffing request**
- **Knowledge asset** (reusable template, methodology, case study)

### 10.2 Roles
- Partner, Principal, Manager/Engagement Manager, Senior Consultant, Consultant, Analyst, Subject Matter Expert, Client Sponsor, Client Project Manager, Business Development Lead, Knowledge Manager, Practice Leader

### 10.3 Compliance/Regulatory
- **Independence requirements** (audit firms cannot consult for audit clients -- SOX)
- **Conflict of interest** (competing clients in same industry)
- **Confidentiality / NDA** (client information must not leak between engagements)
- **Professional licensing** (CPA, CFA, PE for certain advisory work)
- **Data handling requirements** (client data must be isolated)
- **Anti-bribery** (FCPA, UK Bribery Act)
- **Engagement letter / contract requirements**

### 10.4 Artifacts
- Proposals, statements of work, engagement letters, status reports, deliverable documents (analyses, recommendations, implementation plans), presentation decks, workshop materials, data analysis outputs, project plans, change management plans, training materials, knowledge articles

### 10.5 Cadences
- **Weekly:** Client status meetings, internal team check-ins
- **Milestone-based:** Deliverable review cycles
- **Monthly:** Steering committee presentations, billing
- **Phase-based:** Discovery -> Analysis -> Recommendation -> Implementation -> Handoff
- **Quarterly:** Practice reviews, utilization reviews, business development pipeline

### 10.6 Gates
- **Proposal approval** (internal review before submitting to client)
- **Engagement kick-off** (scope, team, timeline confirmed)
- **Phase-end review** (deliverable quality check before presenting to client)
- **Client acceptance** of deliverables
- **Partner review** of key outputs
- **Quality review** (independent review for high-risk engagements)
- **Knowledge capture** before engagement close

### 10.7 Metrics
- Utilization rate (billable hours / available hours), realization rate, revenue per consultant, client satisfaction (NPS), proposal win rate, engagement profitability, on-time delivery rate, knowledge reuse rate, repeat business rate, consultant development (promotion rate, training hours)

### 10.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Utilization as the primary business metric** | Every hour must be classified as billable, non-billable, or investment; this drives staffing, pricing, and profitability |
| **Proposal / bid management** | Pre-engagement work (proposals, RFP responses) is a major process with its own pipeline and win/loss tracking |
| **Staffing / resource allocation across engagements** | Consultants are shared resources across multiple concurrent engagements; staffing optimization is a core workflow |
| **Client-facing deliverable quality** | Outputs go directly to senior executives; formatting, polish, and narrative quality matter in ways that internal documents don't |
| **Knowledge management / reuse** | Consulting firms live or die by reusing methodologies, templates, and case studies; knowledge capture is a formal process |
| **Phase-gated with client sign-off** | Each phase requires explicit client acceptance before proceeding; scope changes require formal change orders |
| **Pyramid staffing model** | Work is deliberately structured so junior staff do data gathering, mid-level does analysis, senior does synthesis and client management |
| **Intellectual property boundaries** | What belongs to the client vs. the firm's methodology is a constant negotiation |

---

## 11. Non-Profit / NGO (Grant Management, Program Delivery)

### 11.1 Work Items
- **Grant application**
- **Program / project** (funded initiative)
- **Activity** (specific intervention within a program)
- **Beneficiary case** (individual or community served)
- **Monitoring visit**
- **Evaluation task**
- **Fundraising campaign**
- **Volunteer coordination task**
- **Advocacy initiative**

### 11.2 Roles
- Executive Director, Program Manager, Program Officer (funder-side), Grant Writer, M&E (Monitoring and Evaluation) Specialist, Field Officer, Country Director, Finance Officer, Fundraiser/Development Officer, Volunteer Coordinator, Board Member, Beneficiary Representative

### 11.3 Compliance/Regulatory
- **Donor/funder requirements** (USAID, World Bank, EU, private foundations each have different rules)
- **501(c)(3) / charity regulations** (restrictions on political activity, unrelated business income)
- **OMB Uniform Guidance** (US federal grants -- cost principles, audit requirements)
- **IATI** (International Aid Transparency Initiative -- publishing spending data)
- **Safeguarding policies** (protection of vulnerable populations, especially children)
- **Anti-terrorism / sanctions compliance** (cannot fund designated entities)
- **Local country regulations** (NGO registration requirements vary dramatically)
- **Sarbanes-Oxley-like requirements** for large nonprofits

### 11.4 Artifacts
- Grant proposals, logframes (Logical Framework Approach), theory of change documents, M&E frameworks, indicator tracking tables, progress reports to donors (narrative + financial), audit reports, program evaluations (mid-term, final, impact), beneficiary data, case studies, annual reports, financial statements, board meeting minutes

### 11.5 Cadences
- **Grant cycles:** Application deadlines, reporting periods (quarterly, semi-annual, annual)
- **Program cycles:** Multi-year (typically 3-5 years per grant)
- **Reporting:** Quarterly to funders, annual to regulators
- **Board meetings:** Quarterly
- **Monitoring visits:** Monthly or quarterly to field sites
- **Evaluation:** Mid-term and end-of-program
- **Fundraising:** Annual campaigns, year-end giving season

### 11.6 Gates
- **Grant approval** (internal decision to pursue, then funder decision)
- **Program design review** (theory of change, logframe validation)
- **Baseline data collection** before program starts
- **Mid-term evaluation** (is the program working? adjust?)
- **Final evaluation** (what was achieved?)
- **Financial audit** (are funds being used properly?)
- **Board approval** for major decisions
- **Donor approval** for budget reallocations above threshold

### 11.7 Metrics
- **Impact metrics:** Lives improved, services delivered, outcomes achieved (sector-specific: literacy rates, vaccination coverage, income levels)
- **Output metrics:** Number of beneficiaries reached, trainings conducted, materials distributed
- **Financial:** Burn rate, cost per beneficiary, overhead ratio, fundraising efficiency
- **Operational:** Donor reporting compliance, activity completion rate, volunteer retention

### 11.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Logframe / Theory of Change as planning framework** | Programs are structured as: inputs -> activities -> outputs -> outcomes -> impact, with indicators at each level -- not backlogs or feature lists |
| **Multi-donor co-funding** | A single program may be funded by 5 donors, each with different reporting requirements, budget formats, and compliance rules |
| **Beneficiary tracking with privacy constraints** | Must track who is served and outcomes achieved, often in contexts with limited data infrastructure and high vulnerability |
| **Impact measurement** | The ultimate metric is long-term societal change, which is hard to attribute and takes years to manifest |
| **Restricted vs. unrestricted funding** | Donor money is legally restricted to specific purposes; every expense must be allocated to the correct funding source |
| **Field context challenges** | Programs operate in environments with limited connectivity, local language requirements, and physical access constraints |
| **Adaptive management** | Programs must adjust to changing contexts (conflict, natural disaster, political change) while maintaining funder accountability |
| **Overhead ratio pressure** | Strong incentive to minimize management/overhead costs, which creates tension with process rigor |

---

## 12. Government / Public Sector

### 12.1 Work Items
- **Policy initiative**
- **Regulation / rulemaking** (proposed rule, comment period, final rule)
- **Budget request / appropriation**
- **Procurement / acquisition** (RFP, evaluation, award, contract management)
- **Legislative item** (bill, amendment, committee report)
- **Constituent service request / case**
- **Audit finding / remediation**
- **Program performance measure**

### 12.2 Roles
- Agency Head, Program Manager, Contracting Officer, Contracting Officer Representative (COR), Policy Analyst, Legislative Liaison, Budget Analyst, Inspector General, Chief Information Officer, Chief Financial Officer, Civil Servant (by grade level -- GS system), Elected Official, Political Appointee

### 12.3 Compliance/Regulatory
- **Federal Acquisition Regulation (FAR)** (procurement rules)
- **FISMA / FedRAMP** (information security)
- **Section 508** (accessibility)
- **FOIA** (Freedom of Information Act -- records must be producible)
- **Records Management** (NARA requirements for federal records)
- **Anti-Deficiency Act** (cannot spend money not appropriated)
- **Government Performance and Results Act (GPRA)** (performance measurement)
- **Administrative Procedure Act** (rulemaking process requirements)
- **Ethics regulations** (gift rules, revolving door, financial disclosure)
- **Classification and security clearance** requirements

### 12.4 Artifacts
- Policy documents, Federal Register notices, budget justifications, acquisition plans, Statements of Work, performance work statements, evaluation criteria, contract documents, performance reports, audit reports, FOIA responses, congressional testimony, regulations (proposed and final), environmental impact statements, privacy impact assessments

### 12.5 Cadences
- **Fiscal year** (Oct 1 - Sep 30 for US federal)
- **Budget cycle** (formulation -> justification -> appropriation -> execution, 18+ months)
- **Rulemaking:** Proposed rule -> comment period (30-180 days) -> final rule (months to years)
- **Congressional calendar:** Sessions, recesses, election cycles
- **Quarterly:** Performance reporting (GPRA)
- **Annual:** Financial audit, performance review, strategic plan update
- **Acquisition cycles:** 6-18 months for major acquisitions

### 12.6 Gates
- **Budget approval** (multiple levels: agency -> OMB -> Congress)
- **Acquisition approval** (milestone decision authority at defined thresholds)
- **Policy review** (interagency clearance, OMB review for significant rules)
- **Public comment period** (mandatory for rulemaking)
- **Inspector General review**
- **Congressional notification** (for major programs)
- **Authority to Operate (ATO)** for IT systems (FISMA)
- **Privacy Impact Assessment** before collecting PII

### 12.7 Metrics
- Program performance measures (GPRA), budget execution rate (obligated vs. appropriated), acquisition lead time, FOIA response time, constituent satisfaction, employee engagement (FEVS), IT security metrics, audit finding closure rate, cost savings/avoidance, contract performance (CPARS)

### 12.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Appropriations-based funding** | Money is legally restricted by Congress to specific purposes and time periods; spending outside authorization is a criminal offense |
| **FAR-based procurement** | Buying anything follows an extraordinarily detailed regulatory process with mandatory competition, protest rights, and documentation |
| **Public accountability and transparency** | FOIA means nearly all records are potentially public; this affects how information is recorded |
| **Rulemaking process** | Creating regulations is a multi-year formal process with mandatory public participation and judicial review |
| **Multi-year budget cycles** | Budget is planned 2 years out, appropriated annually, and executed over 1-5 years depending on color of money |
| **Inspector General / audit culture** | Independent auditors can review ANY program at ANY time; documentation must support retroactive scrutiny |
| **Political calendar overlay** | Administration changes, congressional priorities, and election cycles create discontinuity in project sponsorship |
| **Authority to Operate (ATO)** | IT systems cannot go live without a formal security authorization that can take 6-18 months |
| **Interagency coordination** | Major initiatives require coordination across multiple agencies with different authorities, budgets, and cultures |

---

## 13. Data Science / ML / AI Research

### 13.1 Work Items
- **Experiment** (model training run with hyperparameters, dataset version, results)
- **Data pipeline task** (ingestion, cleaning, transformation, feature engineering)
- **Model development task** (architecture selection, training, evaluation)
- **Model deployment task** (serving, monitoring, A/B testing)
- **Dataset curation** (labeling, augmentation, versioning)
- **Evaluation / benchmark task**
- **Responsible AI review**

### 13.2 Roles
- Data Scientist, ML Engineer, Data Engineer, Research Scientist, MLOps Engineer, Product Manager (ML), Domain Expert (for labeling/validation), Ethics/Responsible AI Reviewer, Data Annotator, Platform Engineer

### 13.3 Compliance/Regulatory
- **EU AI Act** (risk classification, transparency requirements, conformity assessments)
- **GDPR** (right to explanation for automated decisions, data minimization)
- **CCPA** (automated decision-making disclosure)
- **Model Risk Management (SR 11-7)** (if in financial services)
- **FDA** (if ML is used in medical devices -- SaMD regulations)
- **NIST AI Risk Management Framework**
- **Algorithmic bias and fairness requirements** (various emerging regulations)
- **Data licensing** (training data usage rights)
- **Export controls** (for certain AI models)

### 13.4 Artifacts
- Datasets (versioned, with data cards), trained models (versioned, with model cards), experiment logs, evaluation reports, feature importance analyses, data quality reports, model documentation (model cards), fairness/bias audits, deployment configurations, monitoring dashboards, A/B test results, research papers

### 13.5 Cadences
- **Continuous:** Model monitoring, data pipeline execution, drift detection
- **Per-experiment:** Varies (minutes for small experiments, weeks for large training runs)
- **Sprint-aligned:** Feature engineering, pipeline development (when embedded in product teams)
- **Quarterly:** Model retraining evaluation, responsible AI review
- **Annual:** Model revalidation, data retention review

### 13.6 Gates
- **Data quality validation** before model training
- **Experiment review** (peer review of methodology before conclusions)
- **Model evaluation** against benchmark thresholds
- **Fairness/bias audit** before deployment
- **Shadow mode / canary deployment** before full production
- **A/B test statistical significance** before rollout
- **Model monitoring threshold** (performance degradation triggers retraining)
- **Responsible AI review** for high-risk applications

### 13.7 Metrics
- Model performance (accuracy, F1, AUC, RMSE, BLEU -- task-dependent), inference latency, training cost, data quality score, feature importance/drift, fairness metrics (demographic parity, equalized odds), A/B test impact on business metrics, model staleness, pipeline reliability

### 13.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Experiment tracking as a core workflow** | Each experiment has parameters, code version, data version, and results; reproducing and comparing experiments is fundamental |
| **Dataset versioning** | Data is a first-class artifact that changes over time; model provenance requires knowing exactly which data was used |
| **Model lifecycle (distinct from code lifecycle)** | Models have their own versioning, evaluation, deployment, monitoring, and retirement lifecycle separate from the code that creates them |
| **Compute resource management** | GPU/TPU time is expensive and must be budgeted; experiment queuing and prioritization is a real constraint |
| **Reproducibility requirements** | Must be able to recreate any past result from code + data + environment + random seeds |
| **Drift detection and monitoring** | Deployed models degrade over time as data distributions shift; automated monitoring triggers human review |
| **Fairness and bias as quality dimensions** | A model can be "accurate" but discriminatory; fairness metrics are mandatory quality checks |
| **Non-deterministic outputs** | Unlike software tests (pass/fail), ML results are probabilistic; "good enough" thresholds replace binary success criteria |

---

## 14. Product Design / UX

### 14.1 Work Items
- **Design exploration** (concept development, moodboard)
- **User research task** (interview, survey, usability test)
- **Design artifact** (wireframe, mockup, prototype, design system component)
- **Design review feedback item**
- **Accessibility audit item**
- **Design system contribution** (new component, pattern, token)
- **User journey / flow**

### 14.2 Roles
- UX Researcher, UX Designer, UI Designer, Interaction Designer, Visual Designer, Design System Lead, Content Designer/UX Writer, Creative Director, Design Manager, Accessibility Specialist, Information Architect, Service Designer

### 14.3 Compliance/Regulatory
- **WCAG 2.1/2.2** (web accessibility -- increasingly legally mandated)
- **ADA / Section 508** (US accessibility law)
- **EN 301 549** (EU accessibility standard)
- **GDPR dark patterns** (deceptive design prohibitions)
- **California's consent requirements** (clear opt-in/opt-out patterns)
- **Industry-specific UX requirements** (e.g., FDA for medical device UI, aviation for cockpit interfaces)

### 14.4 Artifacts
- User personas, journey maps, empathy maps, wireframes, mockups (with design tool source files), interactive prototypes, design system documentation, component libraries, style guides, usability test reports, research synthesis documents, design specifications (redlines/handoff), accessibility audit reports, motion/animation specifications

### 14.5 Cadences
- **Discovery sprints:** 1-2 weeks of focused research
- **Design sprints:** 1 week (Google Ventures format) or 2-week design iterations
- **Usability testing:** Per feature or monthly cadence
- **Design system review:** Monthly or quarterly
- **Design critique:** Weekly
- **Ahead-of-development:** Design typically runs 1-2 sprints ahead of engineering

### 14.6 Gates
- **Research synthesis approval** (team agrees on insights before designing)
- **Design concept review** (explore multiple directions before committing)
- **Usability test pass** (task completion rate above threshold)
- **Accessibility audit pass** (WCAG AA compliance)
- **Design system review** (component meets system standards)
- **Design-to-engineering handoff** (specifications complete, all states covered)
- **Design QA** (built product matches design intent)

### 14.7 Metrics
- Task completion rate, time on task, error rate, SUS (System Usability Scale) score, NPS, conversion rate, accessibility compliance rate, design system adoption rate, research utilization rate, design iteration count before approval

### 14.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Diverge-converge cycles** | Design explores MULTIPLE solutions before converging; tracking "exploration breadth" is unique |
| **Visual artifact versioning** | Designs are images/prototypes, not code; versioning requires visual diff, not text diff |
| **Design system as living artifact** | A shared component library that evolves; design system health/adoption is a meta-concern |
| **Research-driven rather than stakeholder-driven** | What to build emerges from user research, not requirements documents; the "backlog" is a research insight board |
| **Dual-track workflow** | Research/design runs in parallel with (and ahead of) engineering; two tracks with different cadences sharing dependencies |
| **Subjective quality assessment** | "Is this design good?" involves aesthetic judgment, brand alignment, and emotional response -- not testable in CI |
| **State and interaction coverage** | Every design must account for empty states, error states, loading states, edge cases -- a completeness dimension absent in traditional task tracking |
| **Handoff as a formal deliverable** | The specification package (spacing, colors, interactions, responsive behavior) must be exhaustive for engineering to implement faithfully |

---

## 15. Startup / Venture (Lean Startup, MVP Development)

### 15.1 Work Items
- **Hypothesis** (the fundamental unit: "We believe [X] will result in [Y]")
- **Experiment** (how we test the hypothesis -- landing page, prototype, survey, concierge MVP)
- **Pivot decision** (strategic direction change)
- **Fundraising task** (pitch deck, investor meetings, due diligence responses)
- **Customer development interview**
- **MVP feature**
- **Growth experiment** (A/B test, channel test)

### 15.2 Roles
- Founder/CEO, CTO, Product Lead, Growth Lead, Designer, Engineer, Advisor, Board Member, Investor, Early Customer / Design Partner

### 15.3 Compliance/Regulatory
- **Securities law** (fundraising is regulated -- Reg D, Reg CF, SAFE notes)
- **Corporate governance** (board meetings, minutes, cap table management)
- **IP protection** (patents, trade secrets, NDAs)
- **Industry-specific** (varies wildly based on what the startup does)
- **Employment law** (equity compensation, contractor vs. employee classification)
- **Privacy** (GDPR, CCPA if handling user data)

### 15.4 Artifacts
- Business Model Canvas, Lean Canvas, pitch decks, financial models, customer interview notes, experiment results, MVP builds, competitive analyses, investor updates, board decks, cap table, term sheets, SAFE/convertible note agreements, product roadmap (light-weight)

### 15.5 Cadences
- **Weekly:** Team standup, customer development calls, experiment reviews
- **Bi-weekly or monthly:** Investor updates, board updates
- **Quarterly:** Board meetings (once board is formed), OKR reviews
- **Fundraising cycles:** 3-6 months of active fundraising every 12-24 months
- **Build-Measure-Learn loops:** As fast as possible (days to weeks)

### 15.6 Gates
- **Problem-solution fit validation** (do customers have this problem?)
- **Product-market fit** (do customers want THIS solution?)
- **Go/no-go on pivot** (is the current approach working?)
- **Fundraising readiness** (metrics, deck, story ready for investors)
- **Hiring decisions** (each early hire is a strategic decision)
- **Launch / public release**

### 15.7 Metrics
- Customer acquisition cost (CAC), lifetime value (LTV), LTV:CAC ratio, monthly recurring revenue (MRR), churn rate, activation rate, retention curves, runway (months of cash remaining), burn rate, NPS, experiment velocity (hypotheses tested per week), pirate metrics (AARRR: acquisition, activation, revenue, retention, referral)

### 15.8 What's UNIQUE

| Need | Why Software Tools Miss It |
|---|---|
| **Hypothesis as the atomic work unit** | Everything starts with a testable belief; the backlog is a hypothesis board, not a feature list |
| **Build-Measure-Learn as the core loop** | The process is explicitly iterative and designed to INVALIDATE ideas quickly, not just build features |
| **Runway as an existential constraint** | Cash remaining is the ultimate project deadline; every decision is filtered through "does this extend our runway or bring revenue?" |
| **Pivot as a first-class process event** | Completely changing direction is expected and healthy, not a project failure |
| **Investor reporting and fundraising** | A parallel workstream that competes for founder time and has its own pipeline (investor CRM) |
| **Extreme velocity over process rigor** | Process overhead is the enemy; anything that adds structure must earn its keep immediately |
| **Vanity metrics vs. actionable metrics** | Strong need to distinguish between metrics that look good and metrics that drive decisions |
| **Customer development as a core activity** | Talking to customers is not "research" that happens before "building" -- it is the primary activity |

---

## 16. Additional Sectors Worth Noting

### 16.1 Real Estate Development
- **Unique needs:** Deal pipeline management, due diligence checklists (environmental, title, zoning), financing structure tracking, entitlement process management (government approvals that can take years), tenant lease management, draw schedule (construction loan disbursement)
- **Key differentiator:** Each project is a separate legal entity (SPV/LLC); financial structure IS the project structure

### 16.2 Agriculture / Farming
- **Unique needs:** Seasonal crop planning, weather-dependent scheduling, compliance with agricultural regulations (USDA, organic certification), supply chain from farm to market, equipment maintenance scheduling, variable-rate input management (fertilizer, water, seed per zone)
- **Key differentiator:** Biological timelines that cannot be accelerated; weather as an uncontrollable variable

### 16.3 Energy / Utilities
- **Unique needs:** Regulatory rate cases, grid reliability management, outage management, generation/transmission planning (20+ year horizons), environmental compliance (emissions tracking), NERC reliability standards, nuclear regulatory (NRC) compliance
- **Key differentiator:** Monopoly/regulated business model where most work is driven by regulatory requirements, not market demand

### 16.4 Logistics / Transportation
- **Unique needs:** Route optimization, fleet management, customs documentation, bill of lading management, load planning, carrier performance tracking, last-mile delivery coordination, hazmat compliance (DOT regulations)
- **Key differentiator:** Time-critical physical movement with complex multi-party handoffs and regulatory border crossings

### 16.5 Events / Hospitality
- **Unique needs:** Venue management, vendor coordination (catering, AV, decor), attendee management, event day run-of-show, health/safety compliance, entertainment licensing, countdown-based planning (all work converges on one date)
- **Key differentiator:** Immovable deadline (the event date), physical logistics, and extreme parallelism (everything must converge simultaneously)

---

## 17. Cross-Sector Synthesis

### 17.1 Universal Primitives That Cover All Sectors

The 15 primitives from `process-ontology-primitives-2026-03.md` provide the foundation. Every sector uses:
- Work Items (though they call them different things)
- State Machines (though the states differ)
- Roles (though the role definitions differ dramatically)
- Artifacts (though the types vary)
- Checkpoints/Gates (though what is checked varies)
- Metrics (though what is measured varies)
- Constraints (the most sector-specific primitive)
- Schedules/Cadences (though the rhythms differ)

### 17.2 Sector-Specific Extensions Needed

The following capabilities go BEYOND universal primitives and would need sector-specific template packs:

| Extension | Sectors That Need It | Implementation Approach |
|---|---|---|
| **Hypothesis-result work item type** | Scientific research, Data science/ML, Startups | Work item subtype with hypothesis, method, result, conclusion fields |
| **Time tracking / billable hours** | Legal, Consulting, Professional services | Time entry linked to work items with matter/engagement codes |
| **Regulatory calendar integration** | Financial services, Government, Healthcare, Construction | External calendar feed with auto-generated compliance tasks |
| **Multi-party contractual workflows** | Construction, Government procurement, Legal | Work items visible to specific parties with role-based access per organization |
| **Physical resource scheduling** | Manufacturing, Scientific research, Construction, Events | Resource calendar with booking, conflicts, and utilization tracking |
| **Audit trail with legal retention** | Financial services, Healthcare, Government, Legal | Immutable event log with configurable retention policies and tamper-evident hashing |
| **BOM / hierarchical product structure** | Manufacturing, Construction | Nested artifact structure with revision control and impact analysis |
| **Impact/outcome measurement** | Non-profit, Government, Education | Logframe-style indicator tracking with baseline -> target -> actual |
| **Dual-track parallel workflows** | Product design/UX, Data science | Linked work streams with different cadences and handoff points |
| **Financial structure tracking** | Startups, Non-profit, Government, Real estate | Budget allocation, burn tracking, restricted/unrestricted funds, runway calculation |
| **Editorial/publication calendar** | Media, Education, Marketing | Date-driven content planning with embargo and publish scheduling |
| **Experiment tracking with parameters** | Data science/ML, Scientific research, Startups | Experiment work item with parameters, environment, results, comparison views |
| **Segregation of duties enforcement** | Financial services, Healthcare, Government | Hard constraints on role assignments (builder cannot approve, etc.) |
| **Jurisdictional rules engine** | Legal, Government, Construction | Rule sets that vary by jurisdiction affecting deadlines, requirements, and processes |
| **Creative asset management** | Media, Product design, Marketing | Visual assets with variant management, rights tracking, and visual diffing |

### 17.3 Complexity Spectrum

Sectors ordered by process formality requirements (most formal first):

1. **Healthcare (pharmaceutical/device)** -- human safety + regulatory burden
2. **Government / Public sector** -- public accountability + procurement law
3. **Financial services** -- systemic risk + regulatory density
4. **Construction / Engineering** -- safety + multi-party contracts + permitting
5. **Manufacturing** -- quality systems + supply chain + safety
6. **Legal** -- jurisdictional rules + ethical obligations + audit trails
7. **Scientific research** -- ethics review + reproducibility + funding compliance
8. **Non-profit / NGO** -- donor accountability + impact measurement
9. **Education** -- accreditation + academic governance
10. **Consulting** -- client accountability + utilization tracking
11. **Data science / ML** -- reproducibility + fairness + model lifecycle
12. **Product design / UX** -- moderate formality, dual-track complexity
13. **Media / Content** -- moderate formality, speed-focused
14. **Software development** -- moderate formality, well-tooled
15. **Startup / Venture** -- minimal formality, maximum speed

### 17.4 Implications for aibox Template Library

**Tier 1 -- Universal (ship with aibox core):**
- Generic project (current `minimal` template)
- Software development (current `product` template)
- Research (current `research` template)
- Managed/team (current `managed` template)

**Tier 2 -- Sector packs (installable add-ons):**
- Healthcare/Pharma pack (adds: regulatory submission tracking, CAPA workflow, validated change control, 21 CFR Part 11 audit trail)
- Financial services pack (adds: three lines of defense, model lifecycle, regulatory calendar, segregation of duties)
- Construction/Engineering pack (adds: RFI/submittal workflows, CPM scheduling, pay application, multi-party views)
- Legal pack (adds: matter management, time tracking, conflict check, court calendar, privilege tagging)
- Non-profit pack (adds: logframe, donor reporting, impact measurement, grant lifecycle)
- Government pack (adds: acquisition workflow, appropriations tracking, ATO process, FOIA readiness)

**Tier 3 -- Community contributed:**
- Manufacturing, Education, Media, Consulting, Startup, Data Science, Product Design

### 17.5 Key Design Principle

The analysis confirms the ontology primitives approach: **aibox should NOT build 15 different project management tools**. Instead:

1. The 15 universal primitives handle 70-80% of every sector's needs
2. Sector-specific CONSTRAINTS (regulatory, safety, legal) are the primary differentiator
3. Sector-specific WORK ITEM TYPES (hypothesis, experiment, matter, claim, order) are the secondary differentiator
4. Sector-specific CADENCES (grant cycles, academic calendar, fiscal year, regulatory calendar) are the tertiary differentiator

The composable template pack model -- where sector packs add work item types, constraint definitions, process templates, and cadence configurations on top of universal primitives -- is the right architecture.

---

## Sources and Domain Knowledge

This analysis is based on direct domain knowledge of professional practice in each sector, cross-referenced with:
- PMBOK 7th Edition performance domains (applicable across all sectors)
- ISO 9001:2015 (quality management, applicable to manufacturing, healthcare, services)
- ICH-GCP E6(R3) (clinical trial management)
- Federal Acquisition Regulation (government procurement)
- CSI MasterFormat (construction industry standard)
- Lean Canvas / Business Model Canvas (startup methodology)
- CRISP-DM (data science process model)
- Design Council Double Diamond (product design)
- Logical Framework Approach (non-profit/development sector)
- ABA Model Rules of Professional Conduct (legal profession)
- Basel III / Solvency II frameworks (financial services)
- EU AI Act (AI/ML regulatory framework)
