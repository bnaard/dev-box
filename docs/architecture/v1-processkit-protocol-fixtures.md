# V1 processkit protocol fixtures

This is a provisional fixture contract for WS-G / M5. It is deliberately
limited to a versioned opaque install request and result. Aibox retains only
enabled state, source/channel/version, profile, harnesses, workspace root,
and environment/path facts. The producer owns all policy and interprets the
request without aibox inspecting processkit layouts, skills, templates,
migrations, MCP topology, or harness projections.

The request is passed to `processkit install --request-json <json> --output json`
as typed argv, never a persisted shell command. Results preserve only outcome,
retryable error code, and producer provenance. The fixtures cover success,
no-op, failure/retry, interruption, malformed output, incompatible versions,
availability discovery, and secret-canary non-leakage.

## Producer-release gate

This code is **not wired to production installation**. It may not replace or
remove the bounded v0 bridge until processkit issue #118 ships a compatible,
released CLI and the parties ratify these fixtures with representative golden
project parity. A compatible producer release remains the explicit gate for
M5 production integration.
