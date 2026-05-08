---
apiVersion: processkit.projectious.work/v1
kind: Process
metadata:
  id: PROC-20260414_1100-team-task-distribution
  created: 2026-04-14T11:00:00Z
spec:
  name: team-task-distribution
  owner: ROLE-20260414_1100-CalmHawk-project-manager
  related_decisions:
    - DEC-20260414_1100-NobleStag-team-composition-and-model-mix
---

# Team Task Distribution

How the project-manager role takes an owner request, routes it to the right
team member, supervises execution, and reports back. This is the operational
companion to `DEC-20260414_1100-NobleStag-team-composition-and-model-mix`.

## Inputs

- The owner's request, in whatever form (free text, follow-up, correction, or
  a task that expands a prior commitment).
- The current entity state (open workitems, pending migrations, prior decisions)
  — checked via `index-management` / `query_entities`, never by filesystem walks.

## Step 1 — Clarify and frame

Before routing, PM must be able to state the request as a verb-noun task with
a success criterion. If that is not possible from the input as given, PM asks
the owner a targeted question (one at a time — see `owner-profiling` anti-pattern
on bulk-asking).

## Step 2 — Route via `task-router`

Call `route_task(task_description)`. The router returns a skill + process
override + recommended MCP tool. PM reads this before picking a role —
processkit skill conventions override general judgement.

If `confidence < 0.5`, surface the candidate tools to the owner or escalate
rather than guessing.

## Step 3 — Size via `model-recommender` Workflow B

For multi-step or ambiguous work, PM runs the model-recommender's task-router
workflow against the task list. The output maps each work cluster to a
dimension profile (R/E/S/B/L/G) which picks the role:

| Dominant profile        | Default role               | Model  |
|-------------------------|----------------------------|--------|
| R+E (novel design)      | senior-architect           | Opus   |
| E (routine implementation) | developer               | Sonnet |
| E (mechanical bulk)     | junior-developer           | Haiku  |
| R+E (bounded design)    | junior-architect           | Sonnet |
| R (deep research)       | senior-researcher          | Opus   |
| R (bounded research)    | junior-researcher          | Sonnet |
| S (high-volume admin)   | assistant                  | Haiku  |
| Meta (strategy, review) | project-manager (self)     | Opus   |

PM deviates from the default only with a stated reason logged in the handoff
or a DecisionRecord — not silently.

## Step 4 — Decide solo or fan-out

- **Solo** (single role, single actor) — default.
- **Fan-out** — only after confirming subtask independence (different files,
  different modules) per `agent-management`. Cap at 3–4 parallel clones
  initially even though the role cap is 5; coordination overhead grows faster
  than expected.
- **Pipeline** — researcher → architect → developer → assistant (review/summary)
  when the stages genuinely depend on each other.

When spawning a clone, create a new `Actor` with a fresh ID
(`ACTOR-<timestamp>-<PascalCaseWord>-<role-slug>-NN`) and a new `Binding` to
the role. Do not reuse template actor IDs. Mark `spec.templated_from`
pointing at the template actor.

## Step 5 — Issue the handoff

Every delegation uses the structured handoff block from `agent-management`:

```
## Handoff: PM -> <role>
**Status:** assigned
**Inputs:** <files, context, prior outputs with paths and line numbers>
**Task:** <verb-noun, one sentence>
**Success criteria:** <verifiable>
**Constraints:** <non-negotiables: provider neutrality, zero warnings, etc.>
**Budget hint:** <expected model tier and rough scope>
**Next:** <what you return to PM>
```

## Step 6 — Review

PM reviews every returned output against the success criteria before returning
to owner. Self-review by the executing role is required but not sufficient —
the PM's review is the quality gate.

Acceptance checks PM always runs:

- Output matches the stated success criteria.
- No silent scope expansion.
- Project quality gates passed where applicable (tests, lint, fmt).
- Entity-layer side effects recorded (workitem transitions, decision records,
  index updates).
- Relevant sources cited for research output.

## Step 7 — Report to owner

PM reports back with: what changed, what remains, any blockers, and any
decisions that need owner approval. No trailing summaries of trivial work
(per owner working-style).

## Devil's-advocate pass

Before any decision or plan leaves PM — outbound to owner or inbound to a
role — PM asks:

- What would a reasonable critic say is wrong with this?
- What is the second-best option and why are we not taking it?
- What assumption is load-bearing and under-examined?
- Is there a cheaper model tier that would still meet the criteria?

Surface the answer when non-obvious. Do not perform this as theater — only
when it changes the decision or the handoff.

## Budget awareness

The target orientation is ~5%/85%/10% Opus/Sonnet/Haiku by task count. Opus
actually consumes ~5× the tokens of Sonnet per equivalent output, so the
same mix is closer to ~20%/75%/5% by budget. PM watches:

- If Opus share (by count or by budget) trends upward across a session,
  pause and ask whether the work genuinely needs it.
- If the session is approaching the Max 5x 5-hour window cap, shift
  pending work to Sonnet/Haiku where it fits, or checkpoint and defer
  the rest.

## Escalation rules

- Clone cap exceeded (5 per role) → owner approves before spawning more.
- Senior tier invoked twice in a short window → surface to owner (may signal
  routing error or genuine complexity bump).
- Role boundary unclear (task could be either junior or senior architect) →
  default to junior; escalate on demonstrated need.
- Research finding contradicts a prior DecisionRecord → do not proceed on
  action; flag to owner.

## Gotchas

- **Routing to Opus because the owner asked a hard question.** The owner
  expects PM to push back on over-use of Opus. A hard-sounding question often
  has a simple answer; junior researcher first, senior only on escalation.
- **Skipping the handoff block for "quick" tasks.** Quick tasks fail the same
  way as slow ones when the spec is vague. Use the block every time.
- **Pre-creating clones "just in case".** Clones are created on demand.
  Entity-layer inflation is worse than a 30-second spawn delay.
- **Losing the owner's original intent in translation.** The final report to
  owner should answer the owner's actual question, not the PM's internal
  reframe of it.
