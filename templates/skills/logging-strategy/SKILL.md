---
name: logging-strategy
description: Structured logging strategy including log levels, correlation IDs, context propagation, and PII avoidance. Use when designing logging, reviewing log statements, or setting up log aggregation.
allowed-tools: Bash Read Write
---

# Logging Strategy

## When to Use

When the user is designing a logging approach for an application, reviewing existing log statements for quality, setting up log aggregation (ELK, Loki, CloudWatch), adding correlation IDs to a distributed system, or asking about what to log and what to avoid logging.

## Instructions

### Log Levels

Use log levels consistently and intentionally:

1. **TRACE** — Very fine-grained diagnostic detail (loop iterations, variable state). Never enable in production.
2. **DEBUG** — Information useful during development (SQL queries, cache hits/misses, config values loaded). Off by default in production.
3. **INFO** — Normal operational events that confirm the system is working (service started, request handled, job completed). This is the default production level.
4. **WARN** — Something unexpected that the system can handle but that deserves attention (retry succeeded, deprecated API called, approaching rate limit). Not page-worthy.
5. **ERROR** — A failure that prevented an operation from completing (unhandled exception, downstream service unreachable, write failed). Should trigger an alert or ticket.
6. **FATAL** — The process cannot continue and is shutting down (missing required config, port already bound, unrecoverable state). Extremely rare.

Rule of thumb: if you're unsure between two levels, pick the lower one. Over-logging at DEBUG is better than missing context at ERROR.

### Structured vs Unstructured Logging

Always prefer structured logging (JSON or key-value pairs) over unstructured text:

- Structured logs are machine-parseable, searchable, and filterable
- Unstructured logs require regex parsing and break when message formats change
- Every log line should include: `timestamp`, `level`, `service`, `message`, and a `request_id` or `trace_id`
- Use the reference file `references/structured-logging.md` for language-specific libraries and patterns

### Correlation IDs and Context Propagation

1. Generate a unique `request_id` (UUID or ULID) at the system boundary (API gateway, message consumer)
2. Propagate it through all downstream calls via headers (`X-Request-ID`) or message metadata
3. Include the `request_id` in every log line so all logs for a single request can be correlated
4. In distributed systems, also propagate `trace_id` and `span_id` (W3C Trace Context format)
5. Use MDC (Mapped Diagnostic Context) or equivalent to avoid passing IDs through every function signature

### What to Log

- Request/response summaries: method, path, status code, duration
- State transitions: order placed, payment processed, user activated
- Decision points: why a branch was taken, which cache was hit
- Error details: stack trace, input that caused the failure, downstream error message
- Performance data: query duration, queue depth, batch size
- Lifecycle events: service started, config loaded, graceful shutdown initiated
- Retry attempts: which operation, attempt number, backoff duration
- External calls: downstream service name, response time, response status

### What NOT to Log

- **PII**: names, emails, phone numbers, addresses, IP addresses (unless required and compliant)
- **Secrets**: passwords, API keys, tokens, session IDs, credit card numbers
- **Sensitive business data**: salary figures, health records, financial details
- **High-cardinality user input**: full request bodies (log a hash or truncation instead)
- When in doubt, redact or mask: `email=b***@example.com`, `card=****1234`

### Log Aggregation and Rotation

1. Ship logs to a centralized system (ELK stack, Grafana Loki, CloudWatch Logs)
2. Use structured format (JSON) for ingestion; avoid multi-line logs where possible
3. Set retention policies: hot storage (7-30 days searchable), warm (90 days), cold archive (1+ year if required)
4. Rotate local log files by size (100MB) or time (daily) to prevent disk exhaustion
5. In containers, log to stdout/stderr and let the orchestrator handle collection

### Performance Considerations

- Logging has real cost: I/O, serialization, memory allocation
- Use lazy evaluation for expensive log arguments (don't compute a debug string if debug is off)
- Avoid logging inside tight loops; aggregate and log summaries instead
- Sample high-volume logs (e.g., log 1 in 100 health check requests)
- Async log appenders prevent log I/O from blocking request processing

### Common Anti-Patterns

- **Log and throw**: logging an error and then throwing it causes duplicate log entries up the call stack
- **Logging everything at INFO**: makes production logs noisy and important events invisible
- **String concatenation in log calls**: wastes CPU when the level is disabled; use parameterized messages
- **Missing context**: `"Error occurred"` with no details about what, where, or why
- **Inconsistent formats**: mixing structured and unstructured logs in the same service
- **Logging in a loop**: prefer aggregating counts and logging a summary after the loop
- **Timestamps without timezone**: always use UTC with ISO 8601 format to avoid ambiguity
- **Swallowing exceptions**: catching an exception and logging only a generic message loses the stack trace

## Examples

**User:** "We have a REST API in Python and no structured logging. How should we set it up?"
**Agent:** Recommends replacing the stdlib `logging` module's default formatter with `structlog`. Configures a processor chain that adds `timestamp`, `level`, `service`, `request_id`, and outputs JSON. Adds middleware to generate and propagate a `request_id` for every incoming request. Sets up log level via environment variable (`LOG_LEVEL=INFO` for production, `DEBUG` for development). Refers to `references/structured-logging.md` for the structlog setup pattern.

**User:** "I'm logging user data for debugging, is this okay?"
**Agent:** Reviews the log statements and identifies PII exposure (email addresses, IP addresses logged at INFO level). Recommends masking emails (`u***@domain.com`), removing IP logging unless legally required, and moving detailed user data logging to DEBUG level with a note that DEBUG must never be enabled in production without a data processing agreement.

**User:** "Our logs are huge and costing us a fortune in storage."
**Agent:** Analyzes log volume by level and source. Identifies that health check endpoints generate 80% of INFO logs. Recommends: (1) reduce health check logging to DEBUG or sample at 1%, (2) move verbose middleware logs from INFO to DEBUG, (3) set retention to 14 days hot / 90 days cold, (4) add a `log.skip` tag for high-frequency low-value events. Estimates 60-70% cost reduction.
