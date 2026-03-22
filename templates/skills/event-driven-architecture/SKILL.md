---
name: event-driven-architecture
description: Event-driven system design including event sourcing, CQRS, pub/sub, saga patterns, and message broker selection. Use when designing event-driven systems, implementing messaging, or reviewing async architectures.
---

# Event-Driven Architecture

## When to Use

- Designing a new event-driven system or migrating from synchronous to asynchronous
- Choosing between message brokers (Kafka, RabbitMQ, NATS)
- Implementing event sourcing or CQRS
- Designing saga patterns for distributed transactions
- Reviewing an existing async architecture for reliability or ordering issues
- Handling dead letters, schema evolution, or idempotency concerns

## Instructions

### 1. Identify Event-Driven Suitability

Not every system benefits from event-driven design. Evaluate fit:

- **Good fit:** Loose coupling needed between services, complex workflows spanning multiple services, real-time processing requirements, independent scaling of producers and consumers, audit trail requirements (event sourcing).
- **Poor fit:** Simple CRUD with strong consistency needs, low throughput where synchronous calls are fine, small monolith with no distribution requirement.

Ask: "Would synchronous request-response be simpler and sufficient?" If yes, do not introduce events just for the sake of it.

### 2. Choose the Right Messaging Pattern

See `references/messaging-patterns.md` for detailed patterns with diagrams.

| Pattern | Use Case | Guarantee |
|---|---|---|
| Pub/Sub | Fan-out notifications, event broadcasting | At-least-once per subscriber |
| Point-to-Point | Task distribution, work queues | Exactly-once with ack |
| Request-Reply | Async RPC, query with callback | Correlation-based |
| Competing Consumers | Load balancing across workers | At-least-once, single delivery |

### 3. Select a Message Broker

Match the broker to your requirements:

- **Apache Kafka** — Log-based, high throughput, replay capability. Best for event sourcing, stream processing, and high-volume pipelines. Operational complexity is high; use managed (Confluent, MSK) when possible.
- **RabbitMQ** — Mature, flexible routing (exchanges, bindings). Best for task queues, complex routing topologies, and RPC-style patterns. Lower throughput than Kafka but richer routing.
- **NATS** — Lightweight, low latency, simple ops. Best for microservice communication, IoT, and edge. JetStream adds persistence and exactly-once. Minimal operational overhead.
- **Amazon SQS/SNS** — Fully managed, zero ops. Best for AWS-native workloads. SQS for queues, SNS for pub/sub. Limited features but reliable.
- **Redis Streams** — Good for lightweight streaming when Redis is already in the stack. Not a replacement for Kafka at scale.

Decision factors: throughput needs, ordering guarantees, replay requirements, operational budget, existing infrastructure.

### 4. Implement Event Sourcing

Store state as an append-only sequence of events rather than mutable records.

Key principles:
- Events are immutable facts: `OrderPlaced`, `ItemAdded`, `OrderShipped`
- Current state is derived by replaying events (or from snapshots + subsequent events)
- Never delete events; use compensating events (`OrderCancelled`) instead
- Snapshot periodically to avoid replaying entire history (every 100-500 events)

Pitfalls:
- Schema evolution is hard — use a schema registry (Avro, Protobuf) and additive-only changes
- Event replay can be slow without snapshots
- Querying event stores directly is painful — use CQRS read models

### 5. Design CQRS (Command Query Responsibility Segregation)

Separate the write model (commands) from read models (queries):

- **Write side:** Validates commands, emits events, enforces business rules. Optimized for consistency.
- **Read side:** Projects events into denormalized views. Optimized for query patterns. Can have multiple read models for different query needs.
- **Eventual consistency:** Read models lag behind writes. Design UIs to handle this (optimistic updates, "processing" states).

CQRS without event sourcing is valid and simpler. Use event sourcing only when you need full audit trail or temporal queries.

### 6. Implement Saga Patterns

For distributed transactions spanning multiple services, use sagas:

**Orchestration (centralized coordinator):**
- A saga orchestrator tells each service what to do and handles compensation on failure
- Easier to understand and debug; the flow is visible in one place
- Risk: orchestrator becomes a single point of failure or a god service

**Choreography (event-based):**
- Each service listens for events and reacts independently, emitting its own events
- Better decoupling; no central coordinator
- Risk: hard to track the overall flow, debugging requires distributed tracing

Choose orchestration when: workflow logic is complex, visibility matters, fewer than 5-6 steps.
Choose choreography when: services are truly independent, workflow is simple, teams own services independently.

### 7. Ensure Reliability

- **Idempotent consumers:** Every consumer must handle duplicate messages safely. Use idempotency keys (event ID + consumer ID) with a deduplication store.
- **Dead letter queues (DLQ):** Route messages that fail processing after N retries to a DLQ. Monitor DLQ depth. Build tooling to inspect and replay DLQ messages.
- **Ordering guarantees:** Kafka guarantees order within a partition. RabbitMQ does not guarantee cross-queue order. Design consumers to tolerate out-of-order delivery where possible.
- **Outbox pattern:** Write events to an outbox table in the same DB transaction as the state change, then publish asynchronously. Prevents lost events when the broker is down.
- **Schema evolution:** Use a schema registry. Make all changes backward-compatible (add fields, never remove or rename). Version your event types.

## Examples

### Designing an Order Processing System
User wants to build an async order pipeline. Analyze the workflow (order placed, payment processed, inventory reserved, shipment created). Recommend Kafka for event backbone with event sourcing on the order aggregate. Design an orchestration saga for the order fulfillment flow with compensating actions (refund on payment failure, release inventory on shipment failure). Define event schemas with Avro and a schema registry.

### Migrating from Sync to Async
User has a monolith where service A calls service B and C synchronously. Map the current call graph and identify which interactions are fire-and-forget (notifications, logging) vs. requiring a response. Introduce pub/sub for fire-and-forget, request-reply for queries. Start with the outbox pattern to avoid dual-write issues. Suggest RabbitMQ for the mixed routing needs.

### Debugging Event Pipeline Issues
User reports messages being processed multiple times or out of order. Check for missing idempotency keys in consumers. Verify partition key strategy in Kafka (are related events on the same partition?). Check consumer group configuration and rebalancing behavior. Inspect DLQ for poison messages. Review consumer commit/ack strategy (auto-commit vs. manual).
