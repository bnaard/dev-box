---
name: webhook-integration
description: Webhook design and consumption including signature verification, idempotency, retry handling, and security. Use when implementing webhooks, designing event notification systems, or debugging webhook deliveries.
allowed-tools: Bash(curl:*) Read Write
---

# Webhook Integration

Design, implement, and consume webhooks reliably — payload format, signature
verification, idempotency, retry handling, and security.

## When to Use

- Designing a webhook system for event notifications.
- Implementing a webhook consumer for a third-party service.
- Adding HMAC-SHA256 signature verification.
- Debugging failed deliveries or duplicate processing.
- Setting up dead letter queues for undeliverable events.

## Instructions

### Payload Design

```json
{
  "id": "evt_a1b2c3d4",
  "type": "order.shipped",
  "created_at": "2026-03-22T10:30:00Z",
  "data": { "order_id": "ord_xyz789", "carrier": "ups" }
}
```

- Unique event `id` for idempotency. Dotted `type` for categorization (`resource.action`).
- ISO 8601 timestamp. Domain payload nested under `data` for a stable envelope.

### Signature Verification (HMAC-SHA256)

**Sender:** compute HMAC over raw body, send as `X-Webhook-Signature: sha256=<hex>`.

**Consumer:**
```python
expected = hmac.new(secret.encode(), raw_body, hashlib.sha256).hexdigest()
if not hmac.compare_digest(expected, received_signature):
    return 401
```

- Sign the **raw request body**, not parsed JSON (parsing may reorder fields).
- Use constant-time comparison to prevent timing attacks.
- Include a timestamp to prevent replay attacks; reject events older than 5 minutes.
- Support multiple active secrets during rotation.

### Idempotency

Webhooks deliver **at least once** — consumers must handle duplicates:
- Store processed event IDs in a database/cache with unique constraint.
- Check before processing; use `INSERT ... ON CONFLICT DO NOTHING`.
- TTL on the idempotency store (e.g., 7 days) to bound storage.

### Retry Handling

**As sender:** retry with exponential backoff (1min, 5min, 30min, 2hr, 8hr, 24hr).
Treat 2xx as success, 4xx (except 429) as permanent failure, 5xx/timeouts as transient.
Include `X-Webhook-Retry-Count` header.

**As consumer:** return 200/202 immediately after signature validation. Process
asynchronously — never do heavy work in the handler or the sender will time out.

### Dead Letter Queues

Move events that exhaust retries to a DLQ with full context (payload, headers, error).
Provide tools to inspect and replay. Alert on DLQ growth. Retain for 30+ days.

### Ordering

Webhooks do **not** guarantee ordering. Use `created_at` to detect out-of-order events.
For state changes, include a version/sequence number; discard stale events.

### Testing

- **Local dev:** `ngrok` or `cloudflared tunnel` to expose local endpoints.
- **Inspection:** `webhook.site` or `requestbin` for payload inspection.
- **Automated:** test with invalid signatures, duplicate IDs, out-of-order events, malformed payloads.

### Security

- Always verify HMAC signatures. Restrict by sender IP ranges if published.
- TLS only — never accept webhooks over plain HTTP.
- Enforce payload size limits (e.g., 1MB). Rate-limit the consumer endpoint.
- Store secrets in a vault, not in code or environment variables.

## Examples

### Example 1: Implement a Stripe webhook consumer

```
User: Handle Stripe payment webhooks in our Node.js service.

Agent: Implements POST /webhooks/stripe: reads raw body for signature
  verification, stores event ID in Redis (7-day TTL) for idempotency,
  queues for async processing, returns 200 immediately. Handles
  payment_intent.succeeded, charge.refunded, invoice.payment_failed.
```

### Example 2: Design a webhook system for a SaaS platform

```
User: Notify customers when resources change in our platform.

Agent: Designs registration API (POST /webhooks with url, events[], auto
  secret), envelope format, HMAC-SHA256 signing, exponential retry up to
  24hr then DLQ, admin dashboard for delivery status and replay.
```

### Example 3: Debug webhook delivery failures

```
User: Consumer processes some events twice and misses others.

Agent: Finds two issues: no idempotency check (retries reprocessed) and
  synchronous heavy processing causing sender timeouts. Adds event ID
  dedup with DB unique constraint, moves processing to background queue,
  returns 202 immediately. Success rate rises from 74% to 99.8%.
```
