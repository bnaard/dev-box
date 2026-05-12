---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260414_1100-NobleStag-team-composition-and-model-mix
  created: 2026-04-14 11:00:00+00:00
spec:
  title: Permanent 8-role AI team with model-tier mapping and Opus/Sonnet/Haiku target mix
  state: accepted
  decision: "This project operates with a permanent 8-role AI-agent team, defined as processkit\nRole + Actor + Binding entities under context/{roles,actors,bindings}/:\n\n  1. project-manager          (Opus)   \u2014 owner-facing lead; intake, strategy, routing, review, devil's advocate\n  2. senior-architect         (Opus)   \u2014 large features, complex bugs, cross-cutting design\n  3. junior-architect         (Sonnet) \u2014 small/medium design, architectural questions, bugfix scoping (default architect)\n  4. developer                (Sonnet) \u2014 implementation from plans (default execution role)\n  5. senior-researcher        (Opus)   \u2014 deep research with synthesis and judgement\n  6. junior-researcher        (Sonnet) \u2014 bounded research and lookups (default researcher)\n  7. junior-developer         (Haiku)  \u2014 mechanical edits, bulk patterns, simple bugfixes\n  8. assistant                (Haiku)  \u2014 secretary: briefings, summaries, scheduling, indexing, handovers\n\
    \nTarget orientation mix (task count, not hard budget):\n  - Opus   ~5%   \u2014 reserved for PM, senior design, and deep research where synthesis is load-bearing\n  - Sonnet ~85%  \u2014 the default tier for architecture, implementation, and bounded research\n  - Haiku  ~10%  \u2014 mechanical and administrative work where Haiku is safe\n\nClone policy:\n  - Default clone cap per role: 5\n  - Owner approves any request to exceed the cap\n  - Clones are created on demand, not pre-allocated\n  - Clone actor IDs follow KIND-YYYYMMDD_HHMM-PascalCaseWord-slug (e.g. ACTOR-20260420_0900-SilverOtter-developer-agent-02)\n  - Clones bind to their role via a new Binding; never rename or reuse template actor IDs\n\nIntermediate schema extension:\n  - Roles and Actors carry project-local fields under `spec.x_aibox.*` (model_tier, model,\n    default_clone_cap, escalate_cap_to, is_template, role_ref, clone_of, schema_note)\n  - This namespace is explicitly provisional; processkit is working on a\
    \ canonical team schema\n  - When processkit ships the canonical schema, these entities migrate via a Migration entity\n    (lift `x_aibox.*` fields into their canonical equivalents, preserve IDs)\n"
  context: |
    Owner directive to establish a permanent, token-efficient AI team. The owner acts as
    approver of plans; the project-manager role is the team lead and only direct owner
    interlocutor. The team is sized for a single developer + consultant workload on an
    Anthropic Max 5x subscription ($100/mo), which imposes tight per-session and weekly
    usage limits.

    processkit has Role/Actor/Binding primitives but no canonical team schema yet. Rather
    than wait, this project establishes a minimal local extension (`x_aibox.*`) with a
    commitment to migrate to the canonical schema when released.
  rationale: "- Sonnet-first by default: matches model-recommender guidance that Sonnet is the right\n  choice for most architectural and implementation work, reserving Opus for tasks where\n  reasoning-synthesis is the binding dimension.\n- Haiku for mechanical and administrative work keeps the Opus/Sonnet budget for work\n  that actually needs it.\n- Distinct senior/junior architect and researcher roles (instead of a single tiered role)\n  let PM route each task at its real complexity without second-guessing a model-tier field.\n- A dedicated PM role concentrates the owner-facing and devil's-advocate responsibilities\n  in one place, so delegation to specialist roles stays clean.\n- Assistant role absorbs high-volume, low-stakes work (briefings, summaries, indexing) that\n  would otherwise fragment across other roles and consume higher-tier budget.\n\nBudget-vs-task-count caveat: Opus queries cost roughly 5\xD7 Sonnet per equivalent token,\nHaiku roughly 1/5. A 5%/85%/10% task-count mix\
    \ therefore lands closer to ~20%/75%/5%\nof actual token spend. The numbers in this DEC are task-count orientation, not\nbudget-enforced \u2014 PM watches actual usage and escalates to owner if Opus share creeps up.\n"
  alternatives:
  - option: Single generic 'developer' / 'architect' role with a model-tier flag; PM picks the model per invocation
    rejected_because: Loses the forcing function of 'is this really senior work'. A per-invocation tier flag drifts upward over time because every task feels important in the moment.
  - option: Wait for processkit canonical team schema before establishing anything
    rejected_because: The owner needs the team operational now. `x_aibox.*` namespace with a documented migration commitment captures the intent without blocking on external release.
  - option: Skip the Role/Actor/Binding triplet and define the team inline in AGENTS.md
    rejected_because: Loses queryability via index-management; decays into stale prose; makes clone tracking impossible.
  - option: Use a lower Haiku share and higher Sonnet share (e.g. 0/95/5)
    rejected_because: Under-uses Haiku for the clear high-volume low-stakes work (summaries, indexing, mechanical edits) where it is safe and materially cheaper.
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
