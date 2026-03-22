---
name: estimation-planning
description: Software estimation and planning including story points, velocity tracking, scope negotiation, and technical debt budgeting. Use when estimating work, planning sprints, or negotiating project scope.
---

# Estimation and Planning

## When to Use

When the user is estimating work effort, planning sprints, negotiating scope with
stakeholders, budgeting for technical debt, or asks "how should I estimate this project?"
or "how do I improve our sprint planning?".

## Instructions

### 1. Story Points vs Time Estimates

- **Story points** measure relative complexity, not hours. They account for uncertainty,
  effort, and risk combined into a single number.
- Use the Fibonacci sequence (1, 2, 3, 5, 8, 13) — the gaps reflect estimation uncertainty at larger sizes
- A "1" is the simplest well-understood task on the team. Everything else is relative to it.
- If a story is 13+, it needs to be broken down before it can be estimated
- **Time estimates** (hours/days) are appropriate for: fixed-scope contracts, individual task tracking,
  and teams that prefer them — neither approach is universally superior
- Never convert story points to hours — it defeats the purpose of relative sizing

### 2. Planning Poker

- Each estimator independently selects a card (prevents anchoring bias)
- Discuss outliers: the highest and lowest estimators explain their reasoning
- After discussion, re-vote. Converge within 2 rounds or take the higher estimate.
- Limit sessions to 60-90 minutes; estimate no more than 15-20 stories per session
- The person doing the work does not get extra weight — estimation is a team activity
- Use async estimation (each person submits independently) for remote teams, then discuss outliers

### 3. Cone of Uncertainty

- Early estimates are inherently inaccurate: 4x range at project start, 1.25x at late stages
- Communicate estimates as ranges, not single numbers: "3-5 sprints" not "4 sprints"
- Re-estimate as uncertainty decreases: after design, after spike, after first iteration
- Use the cone to set stakeholder expectations: "our confidence increases as we progress"
- Never promise a single date at the start of a project — provide a range with confidence level

### 4. Velocity Tracking

- Velocity = story points completed per sprint (only count fully done stories)
- Track over at least 5-6 sprints before using velocity for forecasting
- Use the average of the last 3-5 sprints, not the best sprint, for planning
- Velocity is a planning tool, not a performance metric — never compare teams
- When velocity drops, investigate: scope changes, interruptions, technical debt, team changes
- Adjust planned work when velocity trends downward; do not assume it will recover on its own

### 5. Scope Negotiation

- Present scope as a menu with trade-offs: "We can deliver A+B by March, or A+B+C by April"
- Use MoSCoW prioritization: Must-have, Should-have, Could-have, Won't-have (this time)
- Fixed date? Negotiate scope. Fixed scope? Negotiate date. Never fix both.
- Identify the MVP: what is the smallest delivery that provides value?
- Document what was explicitly descoped and why — prevents scope creep later
- Revisit scope decisions at sprint boundaries, not mid-sprint

### 6. Technical Debt Budgeting

- Allocate 15-20% of each sprint to technical debt and maintenance
- Track technical debt items in the same backlog as features — they compete for priority
- Categorize debt: critical (blocks future work), important (slows development), minor (cosmetic)
- Pay down critical debt immediately; schedule important debt each sprint
- Frame debt for stakeholders in business terms: "this refactoring will reduce bug rate by ~30%
  and speed up feature delivery in this area"
- Never let debt accumulate silently — make it visible in planning

### 7. Three-Point Estimation

- For each task, estimate: Optimistic (O), Most Likely (M), Pessimistic (P)
- PERT estimate: `(O + 4*M + P) / 6`
- Standard deviation: `(P - O) / 6`
- Use for project-level estimates where individual task uncertainty compounds
- The pessimistic estimate should assume things go wrong (but not catastrophically)
- Sum the PERT estimates for project total; sum the variances and take the square root for project uncertainty

### 8. Monte Carlo Simulation Basics

- Input: historical velocity data (points per sprint) and total backlog size
- Run 1000+ simulations: randomly sample a velocity for each sprint, count sprints to complete
- Output: probability distribution of completion dates
- Report as: "80% chance of completing by Sprint 12, 95% chance by Sprint 14"
- More credible than single-point estimates because it uses actual historical variation
- Re-run as the backlog changes or new velocity data is available

## Examples

**User:** "The stakeholder wants all 50 stories done by end of quarter. Is that realistic?"
**Agent:** Calculates based on the team's average velocity (last 5 sprints) and remaining
sprints in the quarter. If velocity is 25 points/sprint and 50 stories total 180 points,
that requires 7.2 sprints — but only 5 remain. Presents options: descope to 120 points
of must-haves (fits in 5 sprints), extend the deadline by 2 sprints, or add capacity.
Uses MoSCoW to identify which stories are must-have vs could-have.

**User:** "Our estimates are always wrong. How do we improve?"
**Agent:** Reviews recent sprints to identify patterns: stories consistently under-estimated
(8-point stories taking two sprints), scope creep mid-sprint, or missing tasks not
captured in estimates. Recommends: break stories to 5 points or less, add explicit
time for code review and testing in estimates, track estimate-vs-actual for calibration,
and use planning poker to surface hidden complexity.

**User:** "When will this project be done? We have 400 points in the backlog."
**Agent:** Runs a three-point estimate: with velocity ranging 20-35 points/sprint
(O=12, M=16, P=25 sprints), PERT estimate is 17 sprints. Presents as a range:
"14-20 sprints with 80% confidence." Offers to run a Monte Carlo simulation
using the team's actual sprint velocity history for a more precise forecast.
