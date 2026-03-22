---
name: metrics-monitoring
description: Application metrics and monitoring using RED/USE methods, Prometheus patterns, and SLO-based alerting. Use when instrumenting applications, designing dashboards, or setting up monitoring.
allowed-tools: Bash Read Write
---

# Metrics & Monitoring

## When to Use

When the user is instrumenting an application with metrics, designing monitoring dashboards, setting up alerting, choosing between metric types, defining SLIs/SLOs, or asking about observability methodology (RED, USE, golden signals).

## Instructions

### The Four Golden Signals (Google SRE)

These four metrics cover the health of any request-serving system:

1. **Latency** — Time to serve a request. Track both successful and failed requests separately (a fast 500 is not a healthy request).
2. **Traffic** — Demand on the system. HTTP requests/sec, messages consumed/sec, active connections.
3. **Errors** — Rate of failed requests. Explicit failures (HTTP 5xx), implicit failures (200 with wrong content), and policy violations (responses > SLA threshold).
4. **Saturation** — How "full" the system is. CPU utilization, memory pressure, queue depth, thread pool usage. The most useful saturation metric predicts exhaustion before it happens.

### RED Method (for Request-Driven Services)

Focused on the request path — ideal for APIs, web services, and microservices:

- **Rate** — Requests per second
- **Errors** — Failed requests per second (or error percentage)
- **Duration** — Latency distribution (p50, p95, p99, not just average)

Apply RED to every service boundary: API gateways, backend services, database proxies.

### USE Method (for Resources)

Focused on infrastructure resources — ideal for CPUs, memory, disks, network, queues:

- **Utilization** — Percentage of time the resource is busy (or percentage of capacity used)
- **Saturation** — Work that is queued or waiting (queue depth, swap usage)
- **Errors** — Count of error events (disk errors, NIC drops, OOM kills)

Apply USE to every physical or logical resource: each CPU, each disk, each connection pool.

### Prometheus Metric Types

Choose the right type for each measurement. See `references/metric-types.md` for code examples.

| Type        | Use For                                    | Example                           |
|-------------|--------------------------------------------|------------------------------------|
| **Counter** | Monotonically increasing values            | `http_requests_total`              |
| **Gauge**   | Values that go up and down                 | `temperature_celsius`              |
| **Histogram**| Distribution of values in buckets         | `http_request_duration_seconds`    |
| **Summary** | Pre-calculated quantiles on the client     | `rpc_duration_seconds`             |

Guidelines:
- Use counters for anything you want to rate() over: requests, errors, bytes transferred
- Use gauges for current state: queue depth, active connections, temperature
- Use histograms over summaries in almost all cases (histograms are aggregatable, summaries are not)
- Include a `_total` suffix on counters, a `_seconds` or `_bytes` suffix for units

### Dashboard Design

1. **Start with the golden signals** — every service dashboard should show latency, traffic, errors, saturation
2. **Use p50/p95/p99 for latency** — averages hide outliers; the p99 experience is often 10x worse than p50
3. **Layer dashboards**: overview (all services) -> service detail -> instance detail
4. **Time range defaults**: last 1 hour for debugging, last 24 hours for trend analysis
5. **Include deployment markers** — vertical annotations showing when deploys happened
6. **RED row per dependency**: if service A calls B and C, show RED metrics for each downstream call

### Alerting Thresholds

1. **Alert on symptoms, not causes** — alert on "error rate > 1%" not "CPU > 80%"
2. **Use SLO-based alerting** (burn rate) — see the alerting-oncall skill for details
3. **Set thresholds from baselines** — measure normal behavior for 2 weeks before setting thresholds
4. **Use multi-window alerting** — fire only when both a short window (5min) and long window (1hr) exceed threshold
5. **Avoid static thresholds on gauges** — CPU at 85% is fine for batch workloads, critical for latency-sensitive ones

### SLIs, SLOs, and SLAs

- **SLI (Service Level Indicator)** — a quantitative measure of service quality. Example: "proportion of requests served in < 200ms"
- **SLO (Service Level Objective)** — a target value for an SLI. Example: "99.9% of requests served in < 200ms over 30 days"
- **SLA (Service Level Agreement)** — a business contract with consequences for missing the SLO. Example: "99.9% availability or customer receives credit"

Define SLOs by:
1. Pick 3-5 SLIs that reflect user experience (availability, latency, correctness)
2. Set targets based on user expectations, not system capability
3. Use error budgets: if SLO is 99.9%, you have 0.1% error budget (43 minutes/month)
4. When error budget is exhausted, freeze feature work and focus on reliability

### Instrumentation Checklist

For any new service, instrument at minimum:
- [ ] Request rate, error rate, and latency at every API endpoint
- [ ] Dependency call rate, error rate, and latency for each downstream service/database
- [ ] Queue depth and processing lag for any async work
- [ ] Connection pool utilization (active/idle/max)
- [ ] Application-specific business metrics (orders/sec, signups/day)
- [ ] Runtime metrics (GC pauses, thread count, heap usage)

## Examples

**User:** "I need to add monitoring to our new Go microservice, where do I start?"
**Agent:** Walks through the RED method for the service's HTTP endpoints. Adds Prometheus client library with a histogram for request duration, a counter for requests total (partitioned by method, path, status), and a gauge for in-flight requests. Sets up a `/metrics` endpoint. Creates a Grafana dashboard with RED panels. Adds USE metrics for the database connection pool (utilization = active/max, saturation = waiting requests). Refers to `references/metric-types.md` for Go code examples.

**User:** "Our p99 latency is 3 seconds but our average is 200ms, what's going on?"
**Agent:** Explains that averages mask tail latency and that 1% of users experience 15x worse performance. Suggests using a histogram to inspect the latency distribution. Identifies whether the tail is caused by a specific endpoint, a downstream dependency timeout, or GC pauses. Recommends adding p50/p95/p99 panels to the dashboard and setting an SLO (e.g., p99 < 500ms) with burn-rate alerting.

**User:** "How should we define SLOs for our e-commerce platform?"
**Agent:** Proposes three SLIs: (1) Availability: proportion of non-5xx responses > 99.95%, (2) Latency: p99 of checkout requests < 1s at 99.9%, (3) Correctness: proportion of orders with matching inventory > 99.99%. Sets 30-day rolling windows. Calculates error budgets (availability: 22 minutes/month, latency: 43 minutes/month). Recommends multi-window burn-rate alerts that page when 2% of the monthly budget is consumed in 1 hour.
