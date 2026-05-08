# Archive — `context/actors/` (v1 legacy)

This directory was moved here on 2026-05-09 as part of the v0.25.6
stale-process / v1-legacy cleanup
(BACK-20260508_2241-QuietLark, DEC-20260508_2247-FierceQuail).

## Why archived

The v1 `Actor` entity type was superseded by the v2 `TeamMember`
primitive per **DEC-20260422_0233-SpryTulip** (2026-04-22), and the
`actor-profile` skill was deprecated in favour of `team-manager`.
The skill catalog now lists `actor-profile` only as a transitional
marker; live identity work goes through `team-manager`.

The 9 entities in this archive are:

| File | Was | Live successor |
|---|---|---|
| `ACTOR-20260411_0000-SnappyFrog-bernhard.md` | The human owner | `TEAMMEMBER-thrifty-otter` (Bernhard) |
| `ACTOR-20260414_1100-BrightEagle-senior-architect-agent.md` | Role template — senior-architect | obsolete; team is now named TeamMembers (Cora/Sage/Avery/Robin/Jordan) |
| `ACTOR-20260414_1100-CalmHawk-pm-agent.md` | Role template — PM | superseded by `TEAMMEMBER-cora` |
| `ACTOR-20260414_1100-DeepWhale-senior-researcher-agent.md` | Role template — senior-researcher | obsolete |
| `ACTOR-20260414_1100-NimbleMouse-junior-developer-agent.md` | Role template — junior-developer | superseded by `TEAMMEMBER-robin` |
| `ACTOR-20260414_1100-QuickFalcon-junior-architect-agent.md` | Role template — junior-architect | obsolete |
| `ACTOR-20260414_1100-SteadyOtter-developer-agent.md` | Role template — developer | superseded by `TEAMMEMBER-avery` |
| `ACTOR-20260414_1100-SwiftFox-junior-researcher-agent.md` | Role template — junior-researcher | obsolete |
| `ACTOR-20260414_1100-TidyBee-assistant-agent.md` | Role template — assistant | obsolete |

The 8 role-template Actors created on 2026-04-14 were placeholders
that predate the actual named-agent team. They were never instantiated
into real agent identities; the v2 team was composed directly via
`team-manager.create_team_member` instead.

## Cross-references

Historical workitems and decisions in `context/workitems/` and
`context/decisions/` still reference these ACTOR- IDs. Those references
live in immutable historical records and are intentionally left as-is —
the audit trail stays internally consistent for the period when the
references were authored. The active routing surfaces (`find_skill`,
`task-router`, `team-manager.list_team_members`) only see the v2
TeamMember entities going forward.

## Why kept (vs `git rm`)

Same reasoning as `archive/processes-v1/README.md`: the v1 entities are
part of the project's history, useful as the concrete reference when
authoring the v1→v2 Actor migration that aibox was supposed to emit
when DEC-20260422_0233-SpryTulip was accepted.
