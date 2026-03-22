---
name: alerting-oncall
description: Alert design and on-call practices including severity levels, runbooks, SLO-based alerting, and escalation policies. Use when designing alerts, writing runbooks, or improving on-call processes.
---

# Alerting & On-Call

## When to Use

When the user is designing alerts for a service, writing runbooks, reducing alert fatigue, setting up escalation policies, implementing SLO-based alerting, or improving on-call processes.

## Instructions

### Alert Severity Levels

Define clear severity levels so responders know how urgently to act:

| Severity | Response Time | Action          | Example                                      |
|----------|---------------|-----------------|----------------------------------------------|
| **P1 / Critical** | Immediate (< 5 min) | Page on-call, wake people up | Total service outage, data loss, security breach |
| **P2 / High**     | Within 30 min        | Page during business hours   | Degraded service, error rate > SLO, partial outage |
| **P3 / Medium**   | Within 4 hours       | Ticket, fix today            | Elevated latency, disk 80% full, cert expiring in 7 days |
| **P4 / Low**      | Within 1 business day| Ticket, fix this sprint      | Deprecated API usage, non-critical background job failures |

Rules:
- Only P1 and P2 should page (send push notifications / wake people up)
- P3 and P4 create tickets — they must never page
- Every alert must have exactly one severity; if you cannot decide, it is P3

### Page vs Ticket

The most important decision in alert design is whether an alert pages or creates a ticket:

- **Page** when: users are currently impacted, data is at risk, or the situation will get worse without human intervention
- **Ticket** when: the issue can wait hours or days, automatic recovery is likely, or it is a warning about future risk
- If an alert pages but the responder's action is always "acknowledge and wait," it should be a ticket
- If you page more than twice a week for non-incidents, your alerts need tuning

### SLO-Based Alerting (Burn Rate)

Traditional threshold alerts ("error rate > 1%") are noisy. SLO-based alerting fires only when you are consuming your error budget too fast:

1. Define your SLO: e.g., 99.9% availability over 30 days (error budget = 43 minutes)
2. Calculate burn rate: `burn_rate = actual_error_rate / (1 - SLO_target)`
   - Burn rate 1 = consuming budget at exactly the expected rate
   - Burn rate 14.4 = will exhaust the entire monthly budget in 2 hours
3. Use multi-window alerts to avoid false positives:
   - **Page (P1)**: burn rate > 14.4 in both the last 5 min AND last 1 hour
   - **Page (P2)**: burn rate > 6 in both the last 30 min AND last 6 hours
   - **Ticket (P3)**: burn rate > 3 in both the last 2 hours AND last 24 hours

This approach alerts early on severe problems and late on slow burns, matching human response capabilities.

### Runbook Template

Every alert that pages must have a runbook. Minimum sections:

```
# Alert: <alert_name>
## What it means
One sentence explaining the condition and its user impact.

## Likely causes
- Cause 1 (most common)
- Cause 2
- Cause 3

## Diagnosis steps
1. Check <dashboard_link> for the current state
2. Run <command> to verify the symptom
3. Check recent deploys: <deploy_log_link>

## Mitigation
- Quick fix: <rollback command or toggle>
- If that doesn't work: <escalation path>

## Escalation
- Team: <team_name> (#slack-channel)
- Secondary: <person or team>
```

### Alert Fatigue Prevention

Alert fatigue is the number one on-call quality issue. Combat it by:

1. **Track alert volume** — measure pages per week per person; target < 2 pages per on-call shift
2. **Review every alert weekly** — if an alert fired and no action was taken, delete or downgrade it
3. **Merge related alerts** — 10 alerts for 10 instances of the same service should be 1 grouped alert
4. **Set proper thresholds from data** — use percentile baselines, not guesses
5. **Auto-resolve** — if the condition clears within 5 minutes, do not page (it was a transient spike)
6. **Silence during maintenance** — proactively suppress known alerts during deploys and planned work

### Escalation Policies

1. **Primary on-call** responds within the target time for the severity
2. **Auto-escalate** to secondary if not acknowledged within 10 minutes (P1) or 30 minutes (P2)
3. **Escalate to engineering lead** if not resolved within 1 hour (P1) or 4 hours (P2)
4. **Declare an incident** if multiple P1 alerts fire simultaneously or the issue affects multiple services

### On-Call Handoff Practices

- Handoff at a consistent time (e.g., Monday 10:00 local time)
- Outgoing on-call writes a summary: active incidents, flaky alerts, known issues, anything to watch
- Incoming on-call reviews the summary and acknowledges
- Shadow rotations for new team members (paired with experienced on-call for 1-2 cycles)

### Incident Communication

During a P1/P2 incident:
1. Open an incident channel (Slack, Teams) immediately
2. Post an initial summary: what is broken, who is affected, current severity
3. Update every 15-30 minutes, even if the update is "still investigating"
4. Post to a status page if external users are affected
5. After resolution: timeline, root cause, follow-up actions within 48 hours

## Examples

**User:** "We get paged 15 times a week and most are false alarms. How do we fix this?"
**Agent:** Audits the last 30 days of alerts. Categorizes each by action taken: mitigated (keep), auto-resolved (convert to ticket or delete), no action (delete). Identifies the top 3 noisy alerts and either raises their thresholds, adds auto-resolve windows, or converts them to tickets. Implements alert grouping so per-instance alerts become per-service. Sets a target of < 2 pages per on-call shift and schedules a weekly alert review.

**User:** "How should I set up alerting for our new payment service with a 99.95% availability SLO?"
**Agent:** Calculates the error budget: 0.05% over 30 days = 22 minutes of downtime. Configures three burn-rate alerts: P1 pages when burn rate > 14.4 (5min AND 1hr windows), P2 pages when burn rate > 6 (30min AND 6hr windows), P3 tickets when burn rate > 3 (2hr AND 24hr windows). Creates a runbook for each alert with diagnosis steps specific to the payment service. Adds a latency SLO alert (p99 < 500ms) with the same multi-window approach.

**User:** "I need a runbook template for our database connection pool exhaustion alert."
**Agent:** Writes a runbook: explains that the alert fires when active connections exceed 90% of the pool max for 5 minutes. Lists likely causes: connection leak (query not closing connections), traffic spike, slow queries holding connections. Diagnosis steps: check `db_pool_active` metric, look for long-running queries (`pg_stat_activity`), check recent deploys. Mitigation: restart the affected pods to release connections (immediate), increase pool max if traffic is legitimate, identify and fix the leaking query. Escalation: database team in #db-oncall.
