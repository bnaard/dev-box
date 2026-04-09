# Personas and Validation Scenarios for aibox Context System

**Status:** Draft — in discussion (DISC-001)
**Date:** 2026-03-28
**Relates to:** DISC-001 §2.38-2.39, PRD.md
**Purpose:** Define personas and user stories to validate the new context system design
against real-world usage patterns.

---

## 1. Personas

### Persona 1: Alex — Solo Developer

```yaml
apiVersion: aibox/v1
kind: Actor
metadata:
  id: ACTOR-bold-crane
  labels:
    type: persona
spec:
  name: "Alex Chen"
  type: persona
  background: |
    Freelance full-stack developer, 5 years experience. Works on 2-3 client
    projects simultaneously. Uses Claude Code as primary AI assistant. Comfortable
    with terminal workflows. Values speed over ceremony. Has no team — works alone
    with AI agents.
  expertise: [python, typescript, react, docker, postgresql]
  goals:
    - "Ship client features fast with minimal overhead"
    - "Keep a clean record of decisions for client handoffs"
    - "Never lose context between work sessions"
  frustrations:
    - "Setting up project structure from scratch every time"
    - "Forgetting why I made a decision 3 weeks ago"
    - "Losing track of what's done vs in progress across projects"
  process_needs: managed
  typical_project_size: "20-100 work items, 1-3 month duration"
  agent_usage: "One Claude Code agent, daily interaction"
```

### Persona 2: Dr. Priya — Research Scientist

```yaml
apiVersion: aibox/v1
kind: Actor
metadata:
  id: ACTOR-calm-heron
  labels:
    type: persona
spec:
  name: "Dr. Priya Sharma"
  type: persona
  background: |
    Computational biologist at a university lab. Leads a team of 2 PhD students
    and 1 postdoc. Manages research projects involving data analysis pipelines,
    paper writing, and grant reporting. Uses Python for analysis, LaTeX for papers.
    Needs reproducibility and audit trails for publications.
  expertise: [python, R, bioinformatics, statistics, latex]
  goals:
    - "Track experiments and their outcomes reproducibly"
    - "Manage paper writing pipeline from draft to publication"
    - "Keep grant milestones on track"
    - "Coordinate work across lab members without heavy tooling"
  frustrations:
    - "No structured way to track which experiments led to which findings"
    - "Paper revisions are chaotic — losing track of reviewer comments"
    - "Grant reporting requires digging through months of unstructured notes"
  process_needs: research (possibly full-product for grant management)
  typical_project_size: "50-200 items per research project, 6-24 month duration"
  agent_usage: "AI assistant for literature review, data analysis, writing"
```

### Persona 3: Maria — Small Team Lead

```yaml
apiVersion: aibox/v1
kind: Actor
metadata:
  id: ACTOR-swift-finch
  labels:
    type: persona
spec:
  name: "Maria Torres"
  type: persona
  background: |
    Engineering lead at a 15-person startup. Manages a team of 4 developers.
    Responsible for release planning, code quality, and technical decisions.
    Uses GitHub for code, but wants structured context for AI-assisted development.
    Needs RBAC — developers shouldn't change release processes.
  expertise: [go, kubernetes, system-design, team-management]
  goals:
    - "Consistent development processes across the team"
    - "Clear ownership and accountability for work items"
    - "Decisions documented so new hires can understand why things are the way they are"
    - "Metrics to show engineering health to leadership"
  frustrations:
    - "Every developer has a different workflow"
    - "Decisions made in Slack disappear into history"
    - "No visibility into engineering throughput"
  process_needs: software or full-product
  typical_project_size: "200-500 work items, ongoing"
  agent_usage: "Multiple agents — one per developer, plus CI/review agents"
```

### Persona 4: Sam — Consultant / Contractor

```yaml
apiVersion: aibox/v1
kind: Actor
metadata:
  id: ACTOR-keen-wren
  labels:
    type: persona
spec:
  name: "Sam Rodriguez"
  type: persona
  background: |
    Independent consultant specializing in data engineering. Takes 4-6 short
    engagements per year (2-8 weeks each). Needs to spin up project environments
    fast, work intensively, then hand off everything cleanly to the client.
    Context and decisions must be transferable.
  expertise: [python, spark, airflow, dbt, cloud-infrastructure]
  goals:
    - "Zero-to-productive in minutes for each new engagement"
    - "Clean handoff packages for clients"
    - "Reusable project templates across engagements"
    - "Track time and deliverables for invoicing"
  frustrations:
    - "Each client has different tooling — re-setup every time"
    - "Handoff documents are painful to create retroactively"
    - "Losing track of what was decided vs what was just discussed"
  process_needs: managed
  typical_project_size: "10-50 items, 2-8 week duration"
  agent_usage: "One AI agent, intensive daily use"
```

### Persona 5: kaits — The Company Simulator

```yaml
apiVersion: aibox/v1
kind: Actor
metadata:
  id: ACTOR-sharp-eagle
  labels:
    type: persona
spec:
  name: "kaits Orchestrator"
  type: persona
  background: |
    Not a human — kaits is the multi-agent company simulator that uses aibox
    as infrastructure. It creates repos, initializes aibox projects, assigns
    agent teams, and monitors progress across projects. It needs programmatic
    access to all aibox commands and primitives.
  expertise: [all — kaits manages agents with any expertise]
  goals:
    - "Programmatically create and manage dozens of aibox projects"
    - "Assign agent teams to projects with correct roles"
    - "Monitor progress across all projects via metrics"
    - "Apply company-level processes (SAFe, OKRs) across projects"
  frustrations:
    - "aibox must be fully scriptable — no interactive prompts"
    - "Need predictable file structures for cross-project queries"
    - "Schema changes must be migratable programmatically"
  process_needs: full-product (per project)
  typical_project_size: "50K+ items across all projects"
  agent_usage: "Dozens of agents across multiple projects"
```

### Persona 6: Jordan — Creative / Content Producer

```yaml
apiVersion: aibox/v1
kind: Actor
metadata:
  id: ACTOR-bright-dove
  labels:
    type: persona
spec:
  name: "Jordan Lee"
  type: persona
  background: |
    Technical writer and content strategist at a SaaS company. Manages
    documentation, blog posts, tutorial videos, and marketing content. Works
    with AI for drafting and editing. Needs editorial workflow: draft → review
    → approval → publish.
  expertise: [technical-writing, markdown, content-strategy, seo]
  goals:
    - "Track content from idea through publication"
    - "Editorial review workflow with approval gates"
    - "Manage content calendar (schedules/cadences)"
    - "Link content to product features and releases"
  frustrations:
    - "Content ideas get lost in Slack"
    - "No structured review/approval workflow"
    - "Can't see what content is in flight vs published"
  process_needs: managed + custom editorial process
  typical_project_size: "100-300 content items, ongoing"
  agent_usage: "AI assistant for drafting, editing, SEO optimization"
```

---

## 2. Validation Scenarios as User Stories

### Scenario 1: Solo Dev Starts a New Project

**Persona:** Alex (Solo Developer)

**User Story:**
> As Alex, I want to run a single command to set up a fully configured project
> with work tracking and AI context, so that I can start coding without spending
> time on infrastructure setup.

**Preconditions:**
- Alex has aibox installed on the host machine
- Alex has Docker running
- No existing project directory

**Flow:**
1. Alex runs: `aibox init myproject --image python --process managed`
2. aibox creates the project directory and `aibox.toml`
3. aibox scaffolds `context/` with the "managed" primitives:
   - `context/items/` directories for work items, decisions, actors, roles
   - `context/events/` for event log
   - `context/schemas/` for state machines and categories
   - `context/.aibox/templates/v1.0.0/` with originals
4. aibox creates the Actor entity for Alex (from git config or interactive prompt)
5. aibox creates a default Admin role and assigns Alex to it
6. aibox installs managed skills: `event-log-management`, `workitem-management`,
   `decision-record-management`, `actor-profile-management`, `role-management`,
   `context-archiving`, `standup-context`, `session-handover`
7. aibox builds/pulls the container image
8. Alex enters the container: `aibox up`
9. Agent reads CLAUDE.md → context/ → skills → is immediately productive

**Validation questions:**
- What does the agent see on first session? Is there enough context to be useful?
- How does Alex add the first work item? Does the skill guide the agent?
- What files exist after init? Is it overwhelming or intuitive?
- Is 5 minutes from `aibox init` to "working on my first task" realistic?

---

### Scenario 2: User Has an Idea for a Feature

**Persona:** Alex (Solo Developer)

**User Story:**
> As Alex, I want to tell my AI agent about a feature idea and have it properly
> tracked, so that I don't lose ideas and can prioritize them later.

**Preconditions:**
- Project is running with "managed" package
- Alex is in an active session with the agent

**Flow:**
1. Alex says: "I have an idea — we should add dark mode toggle to the settings page"
2. Agent recognizes this as a feature idea → uses `workitem-management` skill
3. Agent generates word-ID: `aibox id generate --kind WorkItem` → `BACK-calm-lark`
4. Agent creates `context/items/work/BACK-calm-lark.md` with:
   - Frontmatter: kind: WorkItem, state: draft, subtype: feature, priority: medium
   - Level 1 intro: one-line summary
   - Level 2: description of what dark mode means
   - Level 3: (empty for now — details to be fleshed out later)
5. Agent logs creation event via `event-log-management` skill
6. Agent confirms to Alex: "Created BACK-calm-lark — 'Dark mode toggle'. Priority medium, state draft."
7. Agent updates INDEX.md for the work items directory

**Validation questions:**
- Is the flow natural? Does the agent need to ask Alex for details or can it infer?
- What if Alex says "add to backlog" vs "create a ticket" vs "I have an idea"?
  Does the "When to Use" section cover all these phrasings?
- Does the agent know to set priority medium by default, or should it ask?
- Is the file structure correct? Does the three-level rule produce a useful document?

---

### Scenario 3: User Asks "What Am I Working On?"

**Persona:** Alex (Solo Developer)

**User Story:**
> As Alex, I want to ask my agent for a status overview and get an accurate
> picture of my work, so that I can decide what to focus on today.

**Preconditions:**
- Project has 15 work items in various states
- No SQLite index (fresh session, agent reads files directly)

**Flow:**
1. Alex says: "What am I working on? What's the status?"
2. Agent uses `workitem-management` skill → reads `context/items/work/` directory
3. Agent scans all .md files, reads frontmatter (kind, state, priority, owner)
4. Agent groups by state and presents:
   ```
   In Progress (3):
   - BACK-calm-lark — Dark mode toggle (medium)
   - BACK-swift-oak — API auth refactor (high)
   - BACK-bold-fox — Fix login timeout (critical)

   Ready (5): ...
   Blocked (1): BACK-keen-jay — DB migration (blocked by BACK-swift-oak)
   Draft (6): ...
   ```
5. Agent suggests: "BACK-bold-fox is critical — should we focus there?"

**Validation questions:**
- With 15 items, scanning files is fine. What about 500? 5000? At what point
  does the agent need the SQLite index?
- Does the agent read full files or just frontmatter? (Frontmatter-only scan
  would be much faster)
- How does the agent know which items are assigned to Alex vs someone else?
- Should the agent check the event log for recent activity too?

---

### Scenario 4: Work Item Lifecycle Across Multiple Sessions

**Persona:** Alex (Solo Developer)

**User Story:**
> As Alex, I want my work items to maintain continuity across sessions with
> different agents, so that context is never lost when I resume work.

**Preconditions:**
- BACK-swift-oak is in-progress, assigned to Alex
- Alex ends session (agent A does handover)
- Alex starts new session (agent B picks up)

**Flow — Session 1 (Agent A):**
1. Alex works on BACK-swift-oak for 2 hours
2. Agent A logs events: started work, discovered dependency, made design decision
3. Alex says "let's wrap up" → agent A uses `session-handover` skill
4. Agent A creates handover document referencing BACK-swift-oak, events logged,
   open questions, and next steps

**Flow — Session 2 (Agent B):**
1. Agent B starts, reads CLAUDE.md → context/ → latest handover document
2. Agent B reads BACK-swift-oak entity file + recent events in event log
3. Agent B says: "Welcome back. Last session you were working on BACK-swift-oak
   (API auth refactor). You discovered a dependency on BACK-keen-jay and made a
   decision to use JWT instead of session tokens. Next steps were: implement the
   token refresh endpoint."
4. Alex confirms, work continues seamlessly

**Validation questions:**
- Does the handover document contain enough for Agent B to be immediately useful?
- What if there were 20 events in the session — is the handover too verbose?
- Does Agent B know to look at the handover, or does it need instructions in CLAUDE.md?
- What if Alex doesn't explicitly say "wrap up" — are events still useful on their own?

---

### Scenario 5: Two Humans Collaborate with RBAC

**Persona:** Maria (Team Lead) + Bob (Developer on Maria's team)

**User Story:**
> As Maria, I want my team's agents to respect role-based access control, so
> that developers can't accidentally change release processes or role assignments.

**Preconditions:**
- Project with "software" package
- Maria has Admin role; Bob has Developer role
- Release process defined as PROC-bold-hawk

**Flow — Bob tries to modify a process:**
1. Bob says to his agent: "Change the release process to skip the staging deploy"
2. Agent reads Bob's actor profile → sees Developer role
3. Agent reads Developer role → sees restriction: "Cannot modify kind: Process,
   StateMachine, Gate, Constraint, Role"
4. Agent responds: "I can't modify the release process — your Developer role
   restricts process changes. Talk to Maria (Admin) if you need this changed."
5. Agent logs an escalation event

**Flow — Maria modifies the process:**
1. Maria says: "Remove the staging deploy step from the release process"
2. Agent reads Maria's actor profile → sees Admin role → no restriction on Process
3. Agent edits PROC-bold-hawk.md, removes the staging step
4. Agent logs the modification event
5. Agent confirms: "Updated PROC-bold-hawk. Removed staging deploy step."

**Validation questions:**
- Is the RBAC check natural, or does it feel like a blocker?
- What if Bob has a legitimate urgent need? Does escalation feel reasonable?
- How does the agent know who is currently speaking? (git config? explicit identity?)
- What if Bob says "I got Maria's approval verbally"? How strict is the agent?

---

### Scenario 6: Design Discussion Leading to a Decision

**Persona:** Alex (Solo Developer)

**User Story:**
> As Alex, I want to have a structured discussion with my AI agent about a
> design choice and record the decision, so that future me (or a client)
> understands why I chose PostgreSQL over SQLite.

**Preconditions:**
- Project is running, Alex is working on a database choice

**Flow:**
1. Alex says: "Let's discuss whether to use PostgreSQL or SQLite for this project"
2. Agent uses `discussion-management` skill → creates DISC-bright-owl
3. Agent structures the discussion: context, arguments for each option, trade-offs
4. Alex and agent go back and forth, adding arguments
5. Agent records each point in the discussion body (§2.1, §2.2, etc.)
6. Alex says: "OK, let's go with PostgreSQL"
7. Agent uses `decision-record-management` → creates DEC-keen-fox
8. Agent links DEC-keen-fox to DISC-bright-owl
9. Agent logs decision event
10. Discussion state → concluded

**Validation questions:**
- Does the discussion entity add value over just chatting and recording a decision?
- When should the agent suggest creating a formal discussion vs just answering?
- Is the discussion → decision flow smooth or too bureaucratic for a solo dev?
- How does this discussion survive across sessions? (It's a file — naturally persistent)

---

### Scenario 7: Setting Up a Weekly Review Cadence

**Persona:** Dr. Priya (Research Scientist)

**User Story:**
> As Priya, I want to set up a weekly review cadence where my agent reminds me
> to review lab progress, so that experiments and papers stay on track.

**Preconditions:**
- Project with "research" package (or managed + schedule skill added)
- Priya's lab has 3 active research streams

**Flow:**
1. Priya says: "I want a weekly review every Friday afternoon"
2. Agent uses `schedule-management` skill → creates SCHED-warm-lark
3. Agent creates the schedule entity: pattern "weekly(friday)", cadence type,
   links to a review process template
4. Following Friday, Priya starts a session
5. Agent checks active schedules → sees SCHED-warm-lark is due
6. Agent says: "Weekly review is due. Last review was 7 days ago. Shall I
   prepare a summary of this week's progress?"
7. Agent reads recent events, groups by research stream, presents status
8. Priya reviews, makes decisions, agent logs everything

**Validation questions:**
- The agent checks schedules at session start — but only if it knows to. Is this
  instruction in CLAUDE.md or in the skill?
- What if Priya doesn't start a session on Friday? Does Saturday's session still
  remind? (Yes — "overdue by 1 day")
- How much preparation can the agent do automatically vs needing to ask?
- Does the "research" package include schedule-management? (Currently: no — gap!)

---

### Scenario 8: aibox Schema Migration (v1 → v2)

**Persona:** Alex (Solo Developer)

**User Story:**
> As Alex, I want aibox version updates to be safe and guided, so that my
> project context isn't broken by schema changes.

**Preconditions:**
- Project initialized with aibox v1 (apiVersion: aibox/v1)
- Alex updates aibox CLI to v2
- v2 adds a required `metadata.labels.priority` field to WorkItem

**Flow:**
1. Alex runs `aibox update` → CLI updates to v2
2. aibox stores new templates in `context/.aibox/templates/v2.0.0/`
3. aibox generates `context/.aibox/migration/v1-to-v2.md`:
   ```markdown
   # Migration: aibox/v1 → aibox/v2

   ## Changes
   - WorkItem: `metadata.labels.priority` is now required (was optional in spec)
   - Event schema: added `correlation_id` field (optional)
   - New kind: Override (for template customization)

   ## Instructions for your agent
   - Scan all WorkItem files. For any without `metadata.labels.priority`,
     add it based on the existing `spec.priority` field.
   - Update `apiVersion: aibox/v1` to `apiVersion: aibox/v2` in all entity files.
   - Never edit files without asking the user first.
   ```
4. Alex's agent reads the migration document in next session
5. Agent says: "aibox updated to v2. There's a migration to apply — 23 work items
   need a `labels.priority` field added. Shall I proceed?"
6. Alex approves, agent migrates files one by one, logging events

**Validation questions:**
- Is the migration prompt clear enough for any agent to follow?
- What if the agent makes a mistake during migration? (Git provides rollback)
- Should `aibox lint` detect pre-migration state and warn?
- How does this work for large projects (5000 files to migrate)?

---

### Scenario 9: Project Grows to 5,000 Work Items

**Persona:** Maria (Team Lead)

**User Story:**
> As Maria, I want the context system to remain performant as my project grows,
> so that my team's agents aren't slowed down by scale.

**Preconditions:**
- Project has been running for 18 months
- 5,000 work items (3,500 done, 800 archived, 700 active)
- Event log has 50,000+ entries

**Flow:**
1. Maria's agent starts a session
2. Agent needs to find active work items → scans `context/items/work/`
3. With monthly sharding: `context/items/work/2024/06/` through `2026/03/`
   — 24 month directories, ~200 files each
4. Agent reads only the most recent months (active items are recent)
5. For historical queries: agent suggests `aibox sync` to rebuild the SQLite index,
   then queries the index
6. For metrics: agent reads events from recent months' JSONL files, computes
   lead time, throughput, etc.

**Validation questions:**
- Can the agent efficiently find "active items only" without reading 5,000 files?
  (State in frontmatter → agent must read frontmatter of each file, or use INDEX.md)
- Should INDEX.md list items by state? That would let the agent find active items
  without scanning every file.
- When does `aibox sync` become necessary vs optional?
- Is archiving aggressive enough? (3,500 done items still on disk is wasteful)

---

### Scenario 10: kaits Spins Up a Simulated Company Project

**Persona:** kaits (Company Simulator)

**User Story:**
> As kaits, I want to programmatically create an aibox project with a full team
> structure, so that I can simulate a software development team working on a
> product.

**Preconditions:**
- kaits has aibox CLI available
- kaits has a product brief to implement
- kaits needs to create: a project, 5 agent actors, roles, initial backlog

**Flow:**
1. kaits runs: `aibox init simco-webapp --image node --process full-product --non-interactive`
2. aibox creates the project with all primitives active
3. kaits programmatically creates Actor entities for 5 simulated agents:
   - ACTOR-brave-lynx: "Senior Developer" (expertise: react, node)
   - ACTOR-swift-tern: "Junior Developer" (expertise: css, html)
   - ACTOR-calm-crow: "QA Engineer" (expertise: testing, automation)
   - ACTOR-bold-hawk: "Product Owner" (expertise: product-strategy, ux)
   - ACTOR-keen-dove: "Tech Lead" (expertise: architecture, system-design)
4. kaits creates Role entities: Developer, QA, PO, Tech Lead
5. kaits assigns actors to roles
6. kaits creates a Scope entity for "v1.0 Launch"
7. kaits creates 20 initial work items from the product brief
8. kaits creates process definitions (code-review, release) customized for this team
9. kaits spawns agents and assigns them initial work items
10. Agents start working, logging events, making decisions, following processes

**Validation questions:**
- Can all of this be done non-interactively via CLI commands?
- Does `aibox id generate` work in batch mode for creating many entities?
- Is the file-per-entity format efficient for programmatic bulk creation?
- How does kaits monitor progress across the project? (Read event log? Metrics?)

---

## 3. Coverage Analysis

### Which primitives are exercised by each scenario?

| Primitive | S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10 |
|---|---|---|---|---|---|---|---|---|---|---|
| Work Item | x | x | x | x | x | | | x | x | x |
| Event | x | x | | x | x | x | x | x | x | x |
| Decision | | | | x | | x | | | | x |
| Artifact | | | | | | | | | | |
| Actor | x | | | | x | | | | | x |
| Role | x | | | | x | | | | | x |
| Process | | | | | x | | | | | x |
| State Machine | | x | | x | | | | | | |
| Category | | x | | | | | | | | x |
| Cross-Reference | | | | | | x | | | | x |
| Gate | | | | | x | | | | | x |
| Metric | | | | | | | | | x | x |
| Schedule | | | | | | | x | | | |
| Scope | | | | | | | | | | x |
| Constraint | | | | | x | | | | | |
| Context | x | | | x | | | | x | | |
| Discussion | | | | | | x | | | | |

### Gaps identified:
- **Artifact** is not exercised by any scenario — need a scenario for research output
  or build/release tracking
- **Schedule** is only in scenario 7 — could add to scenario 10 (kaits sets up cadences)
- **Constraint** only in scenario 5 (RBAC) — could add WIP limit scenario
