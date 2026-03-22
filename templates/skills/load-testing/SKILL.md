---
name: load-testing
description: Load testing methodology including test types, scenario design, and capacity planning. Use when planning load tests, analyzing test results, or setting up performance testing in CI.
allowed-tools: Bash Read Write
---

# Load Testing

## When to Use

When the user asks to:
- Plan or run a load test for an API, service, or website
- Choose a load testing tool or design test scenarios
- Analyze load test results and identify bottlenecks
- Set up performance testing in a CI pipeline
- Estimate capacity or plan for scaling based on test data

## Instructions

### 1. Choose the Right Test Type

Each type answers a different question:

| Type | VUs / Load | Duration | Question It Answers |
|---|---|---|---|
| **Smoke** | 1-5 VUs | 1-2 min | Does it work at all under minimal load? |
| **Load** | Expected traffic | 10-30 min | Can it handle normal production traffic? |
| **Stress** | 2-5x expected | 10-30 min | Where does it break? What fails first? |
| **Spike** | Sudden burst (10x) | 5-10 min | Can it handle sudden traffic surges? |
| **Soak / Endurance** | Normal traffic | 2-12 hours | Are there memory leaks or degradation over time? |

Always start with a smoke test before running heavier tests.

### 2. Design Realistic Scenarios

A good load test simulates real user behavior, not just raw request throughput:

1. **Identify key user journeys**: login -> browse -> search -> checkout (not just GET /health)
2. **Use realistic data**: randomize user IDs, search terms, product IDs from a dataset
3. **Include think time**: real users pause between actions (1-5 seconds)
4. **Model traffic distribution**: 60% browse, 25% search, 10% add-to-cart, 5% checkout
5. **Include authentication**: token refresh, session management
6. **Test dependent services**: if your API calls a payment provider, account for that latency

```javascript
// k6 example: realistic scenario with stages
import http from 'k6/http';
import { sleep, check } from 'k6';

export const options = {
  stages: [
    { duration: '2m', target: 50 },   // ramp up
    { duration: '5m', target: 50 },   // hold steady
    { duration: '2m', target: 200 },  // stress
    { duration: '5m', target: 200 },  // hold stress
    { duration: '2m', target: 0 },    // ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1500'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const res = http.get('https://api.example.com/products');
  check(res, { 'status 200': (r) => r.status === 200 });
  sleep(Math.random() * 3 + 1);  // 1-4s think time
}
```

### 3. Capture the Right Metrics

Essential metrics to record during every load test:

**Latency** (most important):
- p50 (median), p95, p99, and max — averages hide problems
- Track per-endpoint, not just aggregate

**Throughput**: requests per second (RPS) successfully processed

**Error rate**: percentage of failed requests (HTTP 5xx, timeouts, connection refused)

**Resource utilization** (capture from monitoring):
- CPU usage per service
- Memory RSS and heap usage
- Database connection pool usage
- Queue depth and consumer lag
- Network I/O and open connections

### 4. Tools Overview

| Tool | Language | Strengths |
|---|---|---|
| **k6** | JavaScript | Developer-friendly scripting, good CLI output, Grafana integration |
| **Locust** | Python | Easy to write complex scenarios, distributed mode, web UI |
| **Gatling** | Scala/Java | High performance, detailed HTML reports, CI-friendly |
| **wrk** | Lua (config) | Extremely high throughput for simple URL hammering |
| **hey** | Go | Simple CLI for quick benchmarks, no scripting needed |
| **vegeta** | Go | Constant-rate load generation, good for precise RPS targets |

Choose **k6** or **Locust** for realistic scenario testing. Use **wrk** or **hey** for quick single-endpoint benchmarks.

### 5. Identify Bottlenecks from Results

Analyze results systematically:

- **Latency climbs linearly with load**: resource saturation (CPU, DB connections, thread pool)
- **Latency spikes periodically**: GC pauses, cron jobs, log rotation, cache expiration
- **Errors start at specific VU count**: resource exhaustion (file descriptors, connection pool, memory)
- **Throughput plateaus while latency grows**: the system is at capacity — find the saturated resource
- **Errors only under concurrency**: race conditions, deadlocks, connection pool starvation

Correlate load test metrics with infrastructure monitoring (Grafana, Datadog, CloudWatch) to pinpoint which component saturates first.

### 6. Capacity Planning from Results

Use load test data to estimate scaling needs:

1. Find the **throughput ceiling**: the RPS where p99 latency exceeds SLA
2. Calculate **headroom**: production should run at 50-70% of the ceiling for safety
3. Scale estimation: if 1 instance handles 500 RPS at ceiling, and you need 2000 RPS, plan for 6 instances (2000 / 500 * 1.5 safety factor)
4. Verify by running the load test against the scaled setup

### 7. CI Integration

Add performance gates to prevent regressions:

```yaml
# GitHub Actions example with k6
- name: Run load test
  run: |
    k6 run --out json=results.json load-test.js
- name: Check thresholds
  run: |
    # k6 exits non-zero if thresholds fail
    # Thresholds defined in the test script
```

Guidelines for CI load tests:
- Run smoke or light load tests on every PR (1-2 minutes)
- Run full load tests nightly or before release
- Use **relative thresholds** (compare to baseline) rather than absolute values
- Test against a dedicated environment, never production
- Store results historically to detect gradual degradation

## Examples

**User:** "We need to load test our API before Black Friday"
**Agent:** Designs a test plan: smoke test first to validate the setup, then a load test at 2x normal traffic (based on last year's peak), then a stress test at 5x to find the breaking point. Uses k6 with scenarios modeling the top 5 user journeys weighted by actual traffic distribution. Configures thresholds at p95 < 500ms and error rate < 0.1%.

**User:** "Our load test shows p99 latency spiking every 30 seconds"
**Agent:** Correlates the spikes with infrastructure metrics and identifies GC pauses in the JVM service. The heap is undersized, causing frequent full GCs under load. Recommends increasing heap from 2GB to 4GB and switching to G1GC. Re-runs the test to confirm p99 drops from 3200ms to 180ms.

**User:** "Set up performance testing in our CI pipeline"
**Agent:** Creates a k6 smoke test that runs on every PR (5 VUs, 30 seconds) with thresholds on p95 latency and error rate. Adds a nightly full load test job (100 VUs, 10 minutes) that posts results to a Grafana dashboard. Configures the PR test to fail if p95 latency regresses by more than 20% compared to the main branch baseline.
