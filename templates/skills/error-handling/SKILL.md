---
name: error-handling
description: Error handling patterns across languages including Result types, exceptions, retry strategies, and circuit breakers. Use when designing error handling, reviewing error-prone code, or implementing resilience patterns.
---

# Error Handling

## When to Use

When the user asks to:
- Design error handling for a module or service
- Choose between Result types and exceptions
- Build error hierarchies or structured error responses
- Implement retry logic, exponential backoff, or circuit breakers
- Distinguish user-facing errors from internal errors
- Review code for missing or poor error handling

## Instructions

### 1. Result/Option Types

Use algebraic types where the language supports them -- they make error paths explicit in the type signature.

**Rust:** `Result<T, E>` for recoverable errors, `Option<T>` for missing values. Use `?` for propagation. Define domain error enums with `thiserror`.

**TypeScript:** Use discriminated unions or a `Result<T, E>` library. Avoid `throw` for expected failures (validation, not-found). Reserve exceptions for truly unexpected conditions.

**Python:** Use exceptions as the primary mechanism (idiomatic), but consider returning `None` or a result tuple for functions called in tight loops where try/except overhead matters.

**Java:** Use checked exceptions for recoverable conditions the caller must handle. Use unchecked (`RuntimeException`) for programming errors. Prefer specific exception types over `Exception`.

### 2. Error Hierarchies

Design errors in layers:

1. **Domain errors** -- Business rule violations (`InsufficientFunds`, `ItemOutOfStock`)
2. **Application errors** -- Workflow failures (`OrderNotFound`, `Unauthorized`)
3. **Infrastructure errors** -- External system failures (`DatabaseUnavailable`, `TimeoutError`)

Each layer should only expose errors meaningful at that level. Infrastructure errors should be mapped to application errors before reaching the domain.

### 3. User-Facing vs Internal Errors

Never leak internal details (stack traces, SQL queries, file paths) to end users.

- **Internal errors**: Full context for debugging -- stack trace, request ID, input values
- **User-facing errors**: Human-readable message, error code, optional field-level details
- Map internal errors to user-facing ones at the boundary (HTTP handler, CLI output)
- Log the internal error with a correlation ID; return the ID to the user for support

### 4. Error Codes

Use structured error codes for machine-readable error handling:

- Namespaced: `PAYMENT.DECLINED`, `AUTH.TOKEN_EXPIRED`, `VALIDATION.FIELD_REQUIRED`
- Documented in a central registry (enum, constants file, or docs page)
- Stable across versions -- never reuse or rename codes

### 5. Structured Error Responses

For APIs, return a consistent error shape:

```json
{
  "error": {
    "code": "VALIDATION.FIELD_REQUIRED",
    "message": "The 'email' field is required.",
    "details": [
      { "field": "email", "reason": "required" }
    ],
    "request_id": "req-abc123"
  }
}
```

### 6. Retry Strategies

Use retries only for transient failures (network timeouts, 503s, lock contention).

**Exponential backoff with jitter:**
- Base delay: 100-500ms
- Multiply by 2 each attempt: 200ms, 400ms, 800ms, ...
- Add random jitter (0-100% of delay) to prevent thundering herd
- Set a max retry count (3-5) and a max total timeout

**Never retry:**
- 4xx client errors (except 429 Too Many Requests)
- Validation failures
- Authentication errors (retry after re-auth, not blindly)

### 7. Circuit Breakers

Prevent cascading failures when a downstream service is unhealthy:

- **Closed** (normal): Requests flow through. Track failure rate.
- **Open** (tripped): Requests fail immediately without calling the downstream. Return a fallback or error.
- **Half-open** (probing): After a timeout, allow one request through. If it succeeds, close the circuit. If it fails, reopen.

Configure: failure threshold (e.g., 5 failures in 30s), open duration (e.g., 60s), half-open probe count.

### 8. Error Reporting

- Log errors with structured fields: `level`, `error_code`, `message`, `stack`, `request_id`, `user_id`
- Send unhandled exceptions to an error tracker (Sentry, Datadog, Honeybadger)
- Alert on error rate spikes, not individual errors
- Group errors by root cause, not by message string

## Examples

### Example 1: Domain Error Enum (Rust)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },

    #[error("card declined: {reason}")]
    CardDeclined { reason: String },

    #[error("payment provider unavailable")]
    ProviderUnavailable(#[source] reqwest::Error),
}

pub fn charge(amount: u64, balance: u64) -> Result<Receipt, PaymentError> {
    if amount > balance {
        return Err(PaymentError::InsufficientFunds {
            required: amount,
            available: balance,
        });
    }
    Ok(Receipt { amount })
}
```

### Example 2: Retry with Exponential Backoff (Python)

```python
import time
import random
import httpx

def fetch_with_retry(url: str, max_retries: int = 4) -> httpx.Response:
    base_delay = 0.3
    for attempt in range(max_retries + 1):
        try:
            resp = httpx.get(url, timeout=5.0)
            if resp.status_code == 429:
                retry_after = float(resp.headers.get("Retry-After", base_delay))
                time.sleep(retry_after + random.uniform(0, 0.5))
                continue
            resp.raise_for_status()
            return resp
        except httpx.TransportError:
            if attempt == max_retries:
                raise
            delay = base_delay * (2 ** attempt) + random.uniform(0, base_delay)
            time.sleep(delay)
    raise httpx.HTTPError(f"Failed after {max_retries} retries")
```

### Example 3: Structured API Error Response (TypeScript)

```typescript
class AppError extends Error {
  constructor(
    public code: string,
    message: string,
    public statusCode: number = 500,
    public details?: { field: string; reason: string }[],
  ) {
    super(message);
  }
}

// In route handler
app.post("/orders", async (req, res) => {
  try {
    const order = await createOrder(req.body);
    res.status(201).json(order);
  } catch (err) {
    if (err instanceof AppError) {
      res.status(err.statusCode).json({
        error: {
          code: err.code,
          message: err.message,
          details: err.details,
          request_id: req.id,
        },
      });
    } else {
      console.error("Unhandled error", { request_id: req.id, err });
      res.status(500).json({
        error: {
          code: "INTERNAL.UNEXPECTED",
          message: "An unexpected error occurred.",
          request_id: req.id,
        },
      });
    }
  }
});
```
