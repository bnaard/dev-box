---
name: postmortem-writing
description: Blameless postmortem writing with timeline, root cause analysis, and corrective actions. Use when writing incident postmortems, conducting post-incident reviews, or creating postmortem templates.
---

# Postmortem Writing

## When to Use

When the user needs to write an incident postmortem, conduct a post-incident review,
create a postmortem template, or asks "how do I write a blameless postmortem?" or
"what should a postmortem include?".

## Instructions

### 1. Blameless Postmortem Structure

Every postmortem must include these sections in order:

1. **Title and Metadata**: Incident name, date, severity, duration, author, reviewers
2. **Summary**: 2-3 sentences. What happened, what was the impact, how was it resolved.
3. **Impact**: Who was affected, for how long, quantified (users, revenue, SLA budget consumed)
4. **Timeline**: Chronological events from first signal to full resolution
5. **Root Cause Analysis**: Why it happened, using 5 Whys or fault tree
6. **Contributing Factors**: What made the incident worse or delayed resolution
7. **What Went Well**: What worked during the response (detection, communication, tooling)
8. **Corrective Actions**: Specific, owned, time-bound tasks to prevent recurrence
9. **Lessons Learned**: What the team now knows that it did not before

### 2. Writing the Timeline

- Use UTC timestamps consistently
- Format: `HH:MM UTC — Event description (who/what system)`
- Include: first alert, escalation points, key diagnostic steps, mitigation applied, full resolution
- Separate detection time (first signal) from response time (first human action)
- Note any delays and why they occurred (e.g., "15 min delay — on-call was in a meeting")
- The timeline is factual, not interpretive — save analysis for the root cause section

### 3. Root Cause Analysis with 5 Whys

- Start with the observable failure and ask "why?" iteratively
- Stop when you reach a systemic or process issue, not a human mistake
- Example:
  - Why did the site go down? The database ran out of connections.
  - Why did it run out? A query was holding connections open for 30+ seconds.
  - Why was the query slow? It was doing a full table scan on a 50M row table.
  - Why was there no index? The migration that added the column did not include an index.
  - Why was that not caught? There is no review step for migration performance impact.
- The root cause is the process gap (no migration performance review), not the person who wrote the migration
- If the 5 Whys produce multiple branches, document all of them

### 4. Corrective Actions

- Each action must be: specific, assigned to an owner, and have a due date
- Categorize as: prevent (stop it from happening), detect (catch it faster), mitigate (reduce impact)
- Format: `[P/D/M] Action description — @owner — due YYYY-MM-DD`
- Limit to 5-8 actions — too many means none get done
- Include at least one action from each category (prevent, detect, mitigate)
- Track completion in the team's issue tracker, not just the postmortem document

### 5. Blameless Culture

- Focus on systems and processes, not individuals
- Use passive voice for human errors: "the config was deployed without review" not "Alice deployed without review"
- Ask "what about our system made this mistake easy to make?" not "who made this mistake?"
- Assume everyone acted with the best information available at the time
- The goal is learning, not accountability — accountability happens in separate processes
- If someone feels blamed, the postmortem has failed regardless of the wording

### 6. Facilitation Tips

- Schedule the review within 3-5 business days of the incident (memories fade)
- Invite all responders plus 1-2 people who were not involved (fresh perspective)
- Time-box to 60 minutes: 10 min timeline walk-through, 20 min root cause, 20 min actions, 10 min wrap-up
- The facilitator is not the author — separate roles prevent defensiveness
- Start by reading the timeline aloud as a group to establish shared understanding
- End with: "What is the single most important thing we should change?"

### 7. Template

```markdown
# Postmortem: [Incident Title]

**Date:** YYYY-MM-DD
**Severity:** SEV-1 / SEV-2 / SEV-3
**Duration:** HH:MM (from detection to resolution)
**Author:** [Name]
**Reviewers:** [Names]

## Summary
[2-3 sentences: what happened, impact, resolution]

## Impact
- Users affected: [number or percentage]
- Duration of user impact: [time]
- Revenue impact: [estimate if applicable]
- SLA budget consumed: [percentage]

## Timeline (UTC)
- HH:MM — [First signal / alert fired]
- HH:MM — [Escalation / response began]
- HH:MM — [Key diagnostic finding]
- HH:MM — [Mitigation applied]
- HH:MM — [Full resolution confirmed]

## Root Cause Analysis
### 5 Whys
1. Why? ...
2. Why? ...
3. Why? ...
4. Why? ...
5. Why? ...

## Contributing Factors
- [Factor that made it worse or delayed resolution]

## What Went Well
- [Thing that worked during response]

## Corrective Actions
| Type | Action | Owner | Due Date | Status |
|------|--------|-------|----------|--------|
| Prevent | [Action] | @name | YYYY-MM-DD | Open |
| Detect | [Action] | @name | YYYY-MM-DD | Open |
| Mitigate | [Action] | @name | YYYY-MM-DD | Open |

## Lessons Learned
- [Key takeaway]
```

## Examples

**User:** "We had a production outage last night. Help me write the postmortem."
**Agent:** Asks for the timeline of events, collects facts (what failed, when it was
detected, who responded, how it was fixed), then structures the postmortem using the
template. Guides the 5 Whys analysis from the symptom to the systemic root cause.
Proposes corrective actions in prevent/detect/mitigate categories, each with a
suggested owner and due date.

**User:** "Our postmortems always end with 20 action items that never get done"
**Agent:** Reviews the corrective actions section and helps prioritize: keep the top
5 highest-impact actions, defer the rest to the backlog with explicit justification.
Ensures each action is specific enough to be a single ticket, has a clear owner,
and a realistic due date. Suggests tracking completion in the team's sprint board
rather than the document.

**User:** "How do I make our postmortem process more blameless?"
**Agent:** Reviews the language in recent postmortems for blame patterns (naming
individuals in failure descriptions, using "should have" language). Rewrites examples
using systems-focused language. Recommends separating the facilitator from the author
role, starting reviews with the timeline (shared facts) before analysis, and adding
a "what about our system made this easy?" question to the template.
