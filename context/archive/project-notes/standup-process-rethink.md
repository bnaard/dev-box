# Standup Process: Rethink for AI-Native Workflows

**Date:** 2026-03-23
**Context:** During product process dogfooding (DEC-015), we debated whether to include STANDUPS.md.

## Observation

In human agile teams, standups are scheduled sync rituals — forcing information sharing across people with different contexts. With AI agents, the dynamic is different:

- The agent has full session context — no need to "catch up"
- The human can ask for status on-demand, not on a schedule
- Session handovers (`project-notes/session-*.md`) already capture full state

## Two Distinct Use Cases

1. **"What's the status?"** — lightweight, in-session checkpoint. This *is* a standup, just on-demand rather than scheduled.
2. **"Write all to context"** — comprehensive session handover for persistence and continuity.

These are different in weight and purpose. Standups are forward-looking (what's next, what's blocked), handovers are retrospective (everything that changed).

## Key Insight: Status Requests Should Be Recorded

When the user asks "what's the status?", the response is currently ephemeral (conversation only). But recording it would be useful in all scenarios — solo dev or mixed team. The act of recording a status response effectively *makes* it a standup entry. This means standups don't need to be a separate ritual; they emerge naturally from status requests.

## When STANDUPS.md Becomes Valuable Again

Mixed teams: multiple humans and agents needing a shared running log that persists across conversations. Agent A writes a standup entry, Human B reads it from their phone hours later, Agent C picks up from there. That's genuine multi-party sync — closer to the agile original.

Long-running remote sessions: with Telegram/Signal access, sessions may span days. Multiple lightweight checkpoints accumulate into one eventual session handover.

## Decision (for now)

Skip STANDUPS.md in aibox's own process (DEC-015). The current solo workflow doesn't need the file. But don't redesign the standup skill or template — the concept is sound for the mixed-team future. When that use case becomes real, the right shape will be clear from lived friction.

## Future Consideration

Consider making the standup skill auto-record whenever the user asks for status. This removes the ceremony ("write a standup") and makes it a natural side-effect of the status query. The skill could append a timestamped entry to STANDUPS.md whenever it runs, whether triggered explicitly or via a status request.
