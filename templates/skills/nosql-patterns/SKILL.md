---
name: nosql-patterns
description: NoSQL database patterns for document, key-value, graph, and wide-column stores. Access-pattern-driven design and consistency models. Use when choosing or designing NoSQL data models.
---

# NoSQL Patterns

## When to Use

When the user asks to:
- Choose between NoSQL database types for a use case
- Design a document, key-value, graph, or wide-column data model
- Optimize access patterns in a NoSQL database
- Understand consistency models (eventual, strong, causal)
- Migrate from relational to NoSQL or vice versa

## Instructions

### 1. Choose the Right Store Type

NoSQL is not a single thing. Match the database type to the access pattern:

| Type | Best For | Examples |
|------|----------|---------|
| Document | Variable-schema entities, nested data, content management | MongoDB, Couchbase, Firestore |
| Key-Value | Caching, sessions, feature flags, counters | Redis, DynamoDB (simple), Memcached |
| Wide-Column | Time-series, IoT telemetry, high-write analytical workloads | Cassandra, ScyllaDB, HBase |
| Graph | Relationship-heavy queries: social networks, recommendations, fraud | Neo4j, Neptune, ArangoDB |

**Default to PostgreSQL** unless the use case clearly benefits from a specialized store.
Signs that NoSQL is warranted:
- Schema varies significantly per record (document store)
- Sub-millisecond latency required for simple lookups (key-value)
- Write volume exceeds what a single relational node can handle (wide-column)
- Queries traverse many relationships at variable depth (graph)

### 2. Design for Access Patterns, Not Entities

In relational databases you model entities and normalize. In NoSQL you start with
the queries you need to answer, then design the data to serve those queries directly.

Process:
1. List every read and write operation the application performs
2. For each read, define the exact key or query that retrieves the data
3. Design documents/rows/keys so each read is a single lookup or scan
4. Accept data duplication as a feature, not a bug

### 3. Document Store Patterns (MongoDB)

**Embedding vs. referencing:**
- **Embed** when: child data is always read with the parent, child count is bounded, child is not shared across parents
- **Reference** when: child is shared, child count is unbounded, child is updated independently

```javascript
// Embedded: order with line items (always read together, bounded)
{
  _id: ObjectId("..."),
  customer_id: "cust_123",
  status: "shipped",
  items: [
    { product_id: "prod_456", name: "Widget", qty: 2, price: 9.99 },
    { product_id: "prod_789", name: "Gadget", qty: 1, price: 24.99 }
  ],
  created_at: ISODate("2025-06-15T10:30:00Z")
}

// Referenced: user with posts (unbounded, queried independently)
// users collection
{ _id: "user_123", name: "Alice", email: "alice@example.com" }
// posts collection
{ _id: "post_001", author_id: "user_123", title: "...", body: "..." }
```

**Bucket pattern** for time-series data:
```javascript
// Instead of one document per measurement, bucket by hour
{
  sensor_id: "temp-01",
  bucket: ISODate("2025-06-15T10:00:00Z"),
  measurements: [
    { ts: ISODate("2025-06-15T10:00:12Z"), value: 23.5 },
    { ts: ISODate("2025-06-15T10:01:45Z"), value: 23.7 }
    // ... up to ~200 per bucket
  ],
  count: 2
}
```

### 4. Key-Value Patterns (Redis)

**Naming convention:** use colons as separators for hierarchical keys.
```
user:123:profile        -> JSON blob
user:123:sessions       -> Set of session IDs
session:abc-def-ghi     -> JSON blob with expiry
rate_limit:api:user:123 -> Counter with TTL
```

**Common structures:**
- `SET/GET` with TTL for caching and sessions
- `INCR` with `EXPIRE` for rate limiting
- Sorted sets (`ZADD/ZRANGEBYSCORE`) for leaderboards and time-ordered feeds
- Streams (`XADD/XREAD`) for event processing
- Pub/Sub for real-time notifications (but no persistence; use Streams for durability)

### 5. DynamoDB Single-Table Design

Store multiple entity types in one table using composite keys:

```
PK                  SK                      Attributes
USER#123            PROFILE                 {name, email, plan}
USER#123            ORDER#2025-06-15#001    {status, total, items}
USER#123            ORDER#2025-06-15#002    {status, total, items}
PRODUCT#456         METADATA                {name, price, category}
PRODUCT#456         REVIEW#USER#123         {rating, comment}
```

Query patterns served:
- Get user profile: `PK = USER#123, SK = PROFILE`
- List user orders: `PK = USER#123, SK begins_with ORDER#`
- Orders in date range: `PK = USER#123, SK between ORDER#2025-06-01 and ORDER#2025-06-30`

Use Global Secondary Indexes (GSIs) for access patterns that need a different key structure.

### 6. Consistency Models

| Model | Guarantee | Use When |
|-------|-----------|----------|
| Strong | Read always returns latest write | Financial transactions, inventory counts |
| Eventual | Reads may return stale data temporarily | Social feeds, analytics, caches |
| Causal | Respects happens-before ordering | Chat messages, collaborative editing |
| Read-your-writes | Writer sees own updates immediately | User profile updates, form submissions |

In distributed systems, the CAP theorem constrains choices: during a network
partition, choose availability (AP) or consistency (CP). Most NoSQL databases
default to AP with tunable consistency.

**Practical guidance:**
- Use strong consistency for operations where stale reads cause financial or safety harm
- Use eventual consistency for everything else (it is simpler and faster)
- Implement idempotent writes so retries during failures are safe

## Examples

**User:** "Should I use MongoDB or PostgreSQL for this product catalog?"
**Agent:** Asks about access patterns: if products have highly variable attributes per category (electronics vs. clothing vs. books), MongoDB's flexible schema avoids sparse columns. If the catalog needs complex cross-product queries, joins, and transactional updates across inventory and orders, PostgreSQL with JSONB for variable attributes is simpler. Recommends PostgreSQL with JSONB unless the schema variance is extreme and cross-entity queries are rare.

**User:** "Design a Redis caching layer for our API"
**Agent:** Proposes a cache-aside pattern with TTL-based expiration. Keys follow `resource:id:variant` naming. Uses `SET` with `EX` for individual resources, sorted sets for paginated list caches, and `DEL` on write-through for invalidation. Includes a circuit breaker so the API falls back to the database if Redis is unreachable.

**User:** "We need to model user activity feeds in DynamoDB"
**Agent:** Designs a single-table model with `PK = USER#id` and `SK = ACTIVITY#timestamp#type` for chronological feeds. Adds a GSI with `PK = ACTIVITY_TYPE` and `SK = timestamp` for admin dashboards that query by activity type across all users. Recommends a TTL attribute to auto-expire activities older than 90 days.
