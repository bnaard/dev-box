---
name: integration-testing
description: Integration and E2E testing patterns including testcontainers, database fixtures, API mocking, and CI isolation. Use when writing integration tests, setting up test infrastructure, or debugging flaky tests.
allowed-tools: Bash Read Write
---

# Integration Testing

## When to Use

When the user asks to:
- Write integration tests against databases, APIs, or message queues
- Set up test containers or fixture infrastructure
- Mock external APIs in tests (wiremock, MSW, nock)
- Debug or prevent flaky tests
- Configure CI test isolation and parallelism
- Write snapshot or E2E tests

## Instructions

### 1. Testcontainers

Spin up real databases and services as disposable containers during tests:

- Each test suite gets its own container -- no shared state between suites
- Use fixed image tags (not `latest`) for reproducibility
- Set health checks or wait strategies so tests start only after the service is ready
- Tear down containers after the suite completes (most libraries handle this via RAII or `afterAll`)

Supported in: Python (`testcontainers`), Rust (`testcontainers`), JS/TS (`testcontainers`), Java (`testcontainers`), Go (`testcontainers-go`).

### 2. Database Fixtures

See `references/test-fixtures.md` for language-specific patterns.

Core strategies:
- **Factories** -- Generate objects with sensible defaults, override only what matters per test
- **Fixtures** -- Predefined datasets loaded before tests; good for read-heavy tests
- **Seeds** -- Baseline data loaded once for the full suite; reset per test via transactions

Cleanup approaches (pick one per project):
- **Transaction rollback** -- Wrap each test in a transaction, roll back after. Fast but limited to single-connection tests.
- **Truncate tables** -- Clear all tables between tests. Works with multi-connection scenarios.
- **Recreate schema** -- Drop and recreate per suite. Slowest but guarantees clean state.

### 3. API Mocking

Mock external HTTP APIs so integration tests run without network dependencies:

| Tool | Language | Approach |
|------|----------|----------|
| **WireMock** | Java, standalone | Runs as a local HTTP server, matches requests by URL/body |
| **MSW** | JS/TS | Intercepts at the network level via Service Workers or node interceptors |
| **nock** | Node.js | Patches `http`/`https` modules to intercept outbound requests |
| **httpmock** | Rust | Starts a mock server, register expectations before the test |
| **responses** | Python | Decorates tests to intercept `requests` library calls |

Best practices:
- Assert on what the code sent (method, headers, body), not just what it received
- Use strict mode -- fail if an unexpected request is made
- Record real API responses once, replay in CI (contract tests)

### 4. Snapshot Testing

Capture the output of a function or component and compare against a stored baseline:

- Good for: serialized JSON, rendered HTML, CLI output, error messages
- Update snapshots intentionally (`--update-snapshots` / `-u`) and review diffs in PRs
- Store snapshots in version control next to tests
- Avoid snapshotting volatile data (timestamps, random IDs) -- normalize first

### 5. E2E Testing Patterns

- **Page Object Model** -- Encapsulate page interactions behind an API; tests read like user stories
- **Arrange-Act-Assert** -- Set up state, perform the action, verify the outcome; keep each phase obvious
- **Test user journeys**, not individual pages -- one E2E test should cover a meaningful flow
- Use dedicated test accounts and isolated environments; never test against production data

### 6. CI Test Isolation and Parallelism

- Run integration tests in a separate CI job with longer timeouts
- Use testcontainers or Docker Compose to provide backing services
- Assign each parallel worker a unique database/schema to avoid collisions
- Set a per-test timeout (not just per-job) to catch hangs early
- Cache container images in CI to speed up startup

### 7. Flaky Test Prevention

Common causes and fixes:
- **Shared state** -- Isolate tests with per-test databases or transaction rollback
- **Timing/race conditions** -- Replace `sleep` with explicit wait-for conditions
- **Non-deterministic ordering** -- Randomize test order in CI to surface hidden dependencies
- **Network flakiness** -- Mock all external APIs; use testcontainers for internal services
- **Resource exhaustion** -- Limit parallelism, close connections/files after use

Tag known flaky tests and track them. A flaky test that is ignored is worse than no test.

## Examples

### Example 1: Testcontainers with Database (Python)

```python
import pytest
from testcontainers.postgres import PostgresContainer
from sqlalchemy import create_engine, text

@pytest.fixture(scope="module")
def db_engine():
    with PostgresContainer("postgres:16-alpine") as pg:
        engine = create_engine(pg.get_connection_url())
        # Run migrations
        with engine.begin() as conn:
            conn.execute(text("CREATE TABLE users (id SERIAL, name TEXT)"))
        yield engine

def test_insert_and_query_user(db_engine):
    with db_engine.begin() as conn:
        conn.execute(text("INSERT INTO users (name) VALUES (:n)"), {"n": "Alice"})
        result = conn.execute(text("SELECT name FROM users")).fetchone()
    assert result[0] == "Alice"
```

### Example 2: API Mocking with MSW (TypeScript)

```typescript
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { fetchWeather } from "../src/weather";

const server = setupServer(
  http.get("https://api.weather.example/current", ({ request }) => {
    const url = new URL(request.url);
    if (url.searchParams.get("city") === "Berlin") {
      return HttpResponse.json({ temp: 18, condition: "cloudy" });
    }
    return new HttpResponse(null, { status: 404 });
  })
);

beforeAll(() => server.listen({ onUnhandledRequest: "error" }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test("returns weather for known city", async () => {
  const weather = await fetchWeather("Berlin");
  expect(weather.temp).toBe(18);
});

test("throws for unknown city", async () => {
  await expect(fetchWeather("Atlantis")).rejects.toThrow("City not found");
});
```

### Example 3: Testcontainers with httpmock (Rust)

```rust
use httpmock::prelude::*;
use reqwest;

#[tokio::test]
async fn test_client_sends_auth_header() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/data")
            .header("Authorization", "Bearer test-token");
        then.status(200)
            .json_body_obj(&serde_json::json!({"items": [1, 2, 3]}));
    });

    let client = ApiClient::new(&server.base_url(), "test-token");
    let result = client.fetch_data().await.unwrap();

    assert_eq!(result.items, vec![1, 2, 3]);
    mock.assert(); // Verify the expected request was made
}
```
