---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260414_1100-NobleStag-team-composition-and-model-mix
  created: 2026-04-14 11:00:00+00:00
spec:
  title: Permanent 8-role AI team with model-tier mapping and Opus/Sonnet/Haiku target
    mix
  state: accepted
  decision: |
    This project operates with a permanent 8-role AI-agent team, defined as processkit
    Role + Actor + Binding entities under context/{roles,actors,bindings}/:

      1. project-manager          (Opus)   — owner-facing lead; intake, strategy, routing, review, devil's advocate
      2. senior-architect         (Opus)   — large features, complex bugs, cross-cutting design
      3. junior-architect         (Sonnet) — small/medium design, architectural questions, bugfix scoping (default architect)
      4. developer                (Sonnet) — implementation from plans (default execution role)
      5. senior-researcher        (Opus)   — deep research with synthesis and judgement
      6. junior-researcher        (Sonnet) — bounded research and lookups (default researcher)
      7. junior-developer         (Haiku)  — mechanical edits, bulk patterns, simple bugfixes
      8. assistant                (Haiku)  — secretary: briefings, summaries, scheduling, indexing, handovers

    Target orientation mix (task count, not hard budget):
      - Opus   ~5%   — reserved for PM, senior design, and deep research where synthesis is load-bearing
      - Sonnet ~85%  — the default tier for architecture, implementation, and bounded research
      - Haiku  ~10%  — mechanical and administrative work where Haiku is safe

    Clone policy:
      - Default clone cap per role: 5
      - Owner approves any request to exceed the cap
      - Clones are created on demand, not pre-allocated
      - Clone actor IDs follow KIND-YYYYMMDD_HHMM-PascalCaseWord-slug (e.g. ACTOR-20260420_0900-SilverOtter-developer-agent-02)
      - Clones bind to their role via a new Binding; never rename or reuse template actor IDs

    Intermediate schema extension:
      - Roles and Actors carry project-local fields under `spec.x_aibox.*` (model_tier, model,
        default_clone_cap, escalate_cap_to, is_template, role_ref, clone_of, schema_note)
      - This namespace is explicitly provisional; processkit is working on a canonical team schema
      - When processkit ships the canonical schema, these entities migrate via a Migration entity
        (lift `x_aibox.*` fields into their canonical equivalents, preserve IDs)
  context: |
    Owner directive to establish a permanent, token-efficient AI team. The owner acts as
    approver of plans; the project-manager role is the team lead and only direct owner
    interlocutor. The team is sized for a single developer + consultant workload on an
    Anthropic Max 5x subscription ($100/mo), which imposes tight per-session and weekly
    usage limits.

    processkit has Role/Actor/Binding primitives but no canonical team schema yet. Rather
    than wait, this project establishes a minimal local extension (`x_aibox.*`) with a
    commitment to migrate to the canonical schema when released.
  rationale: |
    - Sonnet-first by default: matches model-recommender guidance that Sonnet is the right
      choice for most architectural and implementation work, reserving Opus for tasks where
      reasoning-synthesis is the binding dimension.
    - Haiku for mechanical and administrative work keeps the Opus/Sonnet budget for work
      that actually needs it.
    - Distinct senior/junior architect and researcher roles (instead of a single tiered role)
      let PM route each task at its real complexity without second-guessing a model-tier field.
    - A dedicated PM role concentrates the owner-facing and devil's-advocate responsibilities
      in one place, so delegation to specialist roles stays clean.
    - Assistant role absorbs high-volume, low-stakes work (briefings, summaries, indexing) that
      would otherwise fragment across other roles and consume higher-tier budget.

    Budget-vs-task-count caveat: Opus queries cost roughly 5× Sonnet per equivalent token,
    Haiku roughly 1/5. A 5%/85%/10% task-count mix therefore lands closer to ~20%/75%/5%
    of actual token spend. The numbers in this DEC are task-count orientation, not
    budget-enforced — PM watches actual usage and escalates to owner if Opus share creeps up.
  alternatives:
  - option: Single generic 'developer' / 'architect' role with a model-tier flag;
      PM picks the model per invocation
    rejected_because: Loses the forcing function of 'is this really senior work'.
      A per-invocation tier flag drifts upward over time because every task feels
      important in the moment.
  - option: Wait for processkit canonical team schema before establishing anything
    rejected_because: The owner needs the team operational now. `x_aibox.*` namespace
      with a documented migration commitment captures the intent without blocking
      on external release.
  - option: Skip the Role/Actor/Binding triplet and define the team inline in AGENTS.md
    rejected_because: Loses queryability via index-management; decays into stale prose;
      makes clone tracking impossible.
  - option: Use a lower Haiku share and higher Sonnet share (e.g. 0/95/5)
    rejected_because: Under-uses Haiku for the clear high-volume low-stakes work (summaries,
      indexing, mechanical edits) where it is safe and materially cheaper.
  consequences: |
    - Every work request now starts with PM routing to a role via task-router + model-recommender
    - AGENTS.md gains a short Team section pointing at context/roles/, context/actors/, and
      context/processes/team-task-distribution.md
    - The index (index-management) must be regenerated after this change so the new entities are
      queryable from the next session onward
    - When processkit ships the canonical team schema, a Migration entity will lift `x_aibox.*`
      fields into their canonical equivalents; this DEC is the referenced-from anchor
  related:
  - ROLE-20260414_1100-CalmHawk-project-manager
  - ROLE-20260414_1100-BrightEagle-senior-architect
  - ROLE-20260414_1100-QuickFalcon-junior-architect
  - ROLE-20260414_1100-SteadyOtter-developer
  - ROLE-20260414_1100-DeepWhale-senior-researcher
  - ROLE-20260414_1100-SwiftFox-junior-researcher
  - ROLE-20260414_1100-NimbleMouse-junior-developer
  - ROLE-20260414_1100-TidyBee-assistant
---
