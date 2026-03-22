---
name: api-design
description: REST API design including resource naming, HTTP methods, status codes, pagination, versioning, and OpenAPI specs. Use when designing APIs, reviewing API contracts, or writing OpenAPI/Swagger documentation.
allowed-tools: Bash Read Write
---

# API Design

Design and review RESTful APIs following industry conventions. Produce consistent,
well-documented API contracts using OpenAPI 3.1.

## When to Use

- Designing a new REST API or extending an existing one.
- Reviewing API contracts for consistency and correctness.
- Writing or generating OpenAPI / Swagger documentation.
- Choosing pagination, filtering, or versioning strategies.
- Defining error response formats.

## Instructions

### Resource Naming

- Use plural nouns for collections: `/users`, `/orders`.
- Nest sub-resources to express ownership: `/users/{id}/orders`.
- Avoid verbs in URIs — the HTTP method conveys the action.
- Use kebab-case for multi-word segments: `/line-items`.
- Keep URIs shallow (max 3 levels of nesting).

See `references/rest-conventions.md` for the full naming and method reference.

### HTTP Methods and Status Codes

Map CRUD operations to methods consistently:

| Operation | Method | Success Code | Notes                  |
|-----------|--------|--------------|------------------------|
| List      | GET    | 200          | Collection endpoint    |
| Read      | GET    | 200          | Single resource        |
| Create    | POST   | 201          | Return Location header |
| Replace   | PUT    | 200          | Full replacement       |
| Patch     | PATCH  | 200          | Partial update         |
| Delete    | DELETE | 204          | No body returned       |

Always return the correct 4xx/5xx status codes. Never return 200 for errors.

### Pagination

Choose cursor-based pagination for large or real-time datasets. Use offset-based
pagination only for small, stable datasets where page-jumping is needed.

Return pagination metadata in the response body:

```json
{
  "data": [...],
  "pagination": {
    "next_cursor": "abc123",
    "has_more": true
  }
}
```

### Filtering and Sorting

- Filter via query parameters: `GET /orders?status=shipped&created_after=2025-01-01`.
- Sort with a `sort` parameter: `GET /users?sort=-created_at,name`.
- Prefix with `-` for descending order.

### Versioning

Prefer URI path versioning (`/v1/users`) for public APIs. Use `Accept` header
versioning for internal APIs where URL stability matters.

Never break existing clients — additive changes only within a version.

### Error Responses

Use a consistent error envelope:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Email is required.",
    "details": [
      { "field": "email", "reason": "required" }
    ]
  }
}
```

### Rate Limiting

Return rate limit headers on every response:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 742
X-RateLimit-Reset: 1672531200
```

Return 429 Too Many Requests when the limit is exceeded.

### HATEOAS

For discoverable APIs, include links to related actions:

```json
{
  "id": "order-123",
  "status": "shipped",
  "_links": {
    "self": { "href": "/v1/orders/order-123" },
    "cancel": { "href": "/v1/orders/order-123/cancel", "method": "POST" }
  }
}
```

### OpenAPI Specification

Write OpenAPI 3.1 specs with reusable components. Extract shared schemas into
`components/schemas`, use `$ref` extensively, and define security schemes globally.

See `references/openapi-patterns.md` for component patterns and security schemes.

### Review Checklist

When reviewing an API design, verify:

1. Resources use plural nouns; no verbs in paths.
2. HTTP methods match their semantics (GET is safe, PUT is idempotent).
3. Status codes are correct (201 for creation, 204 for deletion).
4. Error responses follow the standard envelope.
5. Pagination is present for all list endpoints.
6. Versioning strategy is documented and consistent.
7. Rate limiting headers are included.
8. OpenAPI spec compiles without errors.

## Examples

### Example 1: Design an e-commerce orders API

```
User: Design a REST API for managing orders in an e-commerce system.

Agent: Reads project context, then designs endpoints:

  POST   /v1/orders                  — Create order (201)
  GET    /v1/orders                  — List orders with cursor pagination (200)
  GET    /v1/orders/{id}             — Get order details (200)
  PATCH  /v1/orders/{id}             — Update order (200)
  DELETE /v1/orders/{id}             — Cancel order (204)
  POST   /v1/orders/{id}/payments    — Submit payment (201)
  GET    /v1/orders/{id}/items       — List line items (200)

Writes OpenAPI 3.1 spec with shared schemas for Order, LineItem, Payment,
PaginatedResponse. Includes error envelope and rate limit headers.
```

### Example 2: Review an existing API spec

```
User: Review this API for issues. [provides OpenAPI spec]

Agent: Reads the spec, identifies problems:
  - GET /getUsers uses a verb — rename to GET /users
  - POST /users returns 200 — should return 201 with Location header
  - No pagination on list endpoints
  - Error responses use inconsistent formats
  Produces corrected spec.
```

### Example 3: Add filtering and sorting to a list endpoint

```
User: Add filtering by status and date range to GET /v1/orders.

Agent: Adds query parameters to the OpenAPI spec:
  - status: enum (pending, shipped, delivered, cancelled)
  - created_after: date-time
  - created_before: date-time
  - sort: string (default: -created_at)
  Updates the spec with parameter definitions and examples.
```
