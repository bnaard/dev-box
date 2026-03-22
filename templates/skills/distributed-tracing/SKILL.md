---
name: distributed-tracing
description: Distributed tracing with OpenTelemetry including spans, traces, context propagation, and sampling strategies. Use when instrumenting distributed systems, debugging request flows, or setting up tracing infrastructure.
allowed-tools: Bash Read Write
---

# Distributed Tracing

## When to Use

When the user is instrumenting a distributed system with tracing, debugging a request that spans multiple services, setting up OpenTelemetry, choosing a sampling strategy, or trying to understand latency across service boundaries.

## Instructions

### Core Concepts

1. **Trace** — The entire journey of a request through the system. Identified by a `trace_id` (128-bit hex string). A single trace contains all work triggered by one user action.
2. **Span** — A single unit of work within a trace. Has a name, start time, duration, status, and optional attributes. Spans form a tree: every span (except the root) has a parent span.
3. **Context** — The carrier of trace information (`trace_id`, `span_id`, `trace_flags`) across process boundaries. Propagated via HTTP headers, message metadata, or gRPC metadata.
4. **Baggage** — Key-value pairs propagated alongside trace context. Use for cross-cutting concerns (tenant ID, feature flags) but keep it small — baggage is sent with every outgoing request.

### OpenTelemetry Architecture

OpenTelemetry (OTel) is the standard for distributed tracing. Key components:

- **SDK** — In-process library that creates and exports spans
- **API** — Vendor-neutral interface your code instruments against
- **Exporter** — Sends spans to a backend (Jaeger, Zipkin, Tempo, Datadog)
- **Collector** — Optional proxy that receives, processes, and exports telemetry data. Recommended for production (decouples apps from backends)
- **Auto-instrumentation** — Libraries that automatically create spans for common frameworks (HTTP clients, database drivers, messaging)

### Instrumentation Strategy

1. **Start with auto-instrumentation** — most languages have OTel auto-instrumentation that covers HTTP, gRPC, database, and messaging libraries with zero code changes
2. **Add manual spans for business logic** — auto-instrumentation covers infrastructure; add custom spans for domain operations (process order, validate payment, generate report)
3. **Set meaningful span names** — use `<component>.<operation>` format: `OrderService.createOrder`, `PaymentGateway.charge`, `DB.query`
4. **Add span attributes** for debugging context:
   - `http.method`, `http.url`, `http.status_code` (HTTP spans)
   - `db.system`, `db.statement`, `db.operation` (database spans)
   - `order.id`, `user.tier`, `payment.method` (business context)
5. **Record errors properly** — set span status to ERROR and record the exception as a span event
6. **Keep spans focused** — a span should represent one logical operation. If a span covers "process order + send email + update inventory", break it into child spans.

### Context Propagation

Context propagation is what connects spans across service boundaries:

- **W3C Trace Context** (recommended) — standard headers: `traceparent` and `tracestate`
  - `traceparent: 00-<trace_id>-<parent_span_id>-<trace_flags>`
  - Example: `traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`
- **B3 Propagation** (Zipkin) — headers: `X-B3-TraceId`, `X-B3-SpanId`, `X-B3-Sampled`
- **Message queues** — propagate context in message headers/metadata, not in the message body
- **Async operations** — pass context explicitly when spawning background work; it does not propagate automatically across thread/goroutine boundaries in most languages

### Sampling Strategies

In high-traffic systems, tracing every request is expensive. Use sampling:

1. **Head-based sampling** — decide at the trace root whether to sample (before any spans are created)
   - **Probabilistic**: sample X% of traces (e.g., 10%). Simple but can miss rare errors.
   - **Rate-limiting**: sample N traces per second. Prevents overload but biases toward high-traffic endpoints.
   - Best for: most production systems as a starting point

2. **Tail-based sampling** — decide after the trace is complete (requires the OTel Collector)
   - **Error-based**: always keep traces that contain errors
   - **Latency-based**: always keep traces above a latency threshold (e.g., > 2s)
   - **Policy-based**: combine rules (keep all errors + keep 5% of successful requests)
   - Best for: systems where you need to capture rare but important events

3. **Always-on sampling** — sample 100% (low-traffic or critical-path services only)

Recommendation: start with head-based probabilistic (10-20%), add tail-based error/latency sampling in the Collector as traffic grows.

### Trace Analysis for Debugging

When debugging with traces:

1. **Start from the symptom** — find traces matching the error, high latency, or anomaly
2. **Identify the critical path** — the longest chain of sequential spans determines total latency
3. **Look for gaps** — time between a parent span and its children indicates queueing or scheduling delay
4. **Check span attributes** — look at error messages, SQL queries, cache hit/miss indicators
5. **Compare fast vs slow traces** — overlay a normal trace with a slow one to spot divergence
6. **Follow the waterfall** — the trace timeline shows parallel vs sequential execution, revealing optimization opportunities

### Common Pitfalls

- **Missing context propagation** — if you see disconnected traces, a service is not forwarding the `traceparent` header
- **Over-instrumentation** — creating a span for every function call generates noise and overhead; span creation should be meaningful
- **Sensitive data in attributes** — never put PII, credentials, or full SQL parameters in span attributes
- **Not sampling in production** — 100% sampling at scale will overwhelm your tracing backend and increase costs dramatically
- **Ignoring async boundaries** — traces break when context is not passed to background jobs, message consumers, or thread pools

## Examples

**User:** "We have 5 microservices in Python and no tracing. How do we set up distributed tracing?"
**Agent:** Recommends OpenTelemetry with auto-instrumentation as the fastest path. Installs `opentelemetry-distro` and `opentelemetry-instrumentation` packages. Configures the OTel SDK to export spans via OTLP to an OTel Collector. Sets up W3C Trace Context propagation (default in OTel). Deploys the Collector with a Jaeger exporter for visualization. Adds manual spans for key business operations. Configures head-based sampling at 20% with a Collector tail-sampling policy that keeps all error traces.

**User:** "A request to our checkout endpoint sometimes takes 10 seconds but usually takes 200ms. How do I debug this?"
**Agent:** Searches the tracing backend for traces on the checkout endpoint with duration > 5s. Examines the trace waterfall and identifies that 9 seconds are spent in a span calling the inventory service. Drills into the inventory service trace and finds it is making a sequential database query for each item instead of a batch query. The slow traces correlate with large carts (> 20 items). Recommends: batch the DB query, add a span attribute for `cart.item_count` to make this pattern visible in future traces.

**User:** "Our tracing costs are too high, we're sending 50 million spans per day."
**Agent:** Analyzes span sources and finds that health check endpoints generate 40% of spans and an over-instrumented logging middleware creates 3 spans per request instead of 1. Recommends: (1) exclude health checks from tracing via sampler config, (2) collapse the middleware spans into one, (3) deploy an OTel Collector with tail-based sampling that keeps 100% of errors, 100% of slow requests (> 2s), and 5% of successful requests. Estimates 80% reduction in span volume while retaining all interesting traces.
