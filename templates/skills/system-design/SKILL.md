---
name: system-design
description: System design methodology from requirements through capacity estimation to component design and trade-offs. Use when designing distributed systems, evaluating architectures, or preparing system design discussions.
---

# System Design

## When to Use

- Designing a new distributed system from scratch
- Evaluating an existing architecture for scalability, reliability, or cost
- Performing capacity estimation (QPS, storage, bandwidth)
- Discussing trade-offs between different architectural approaches
- Preparing for or conducting system design reviews

## Instructions

### 1. Gather Requirements

Always start with requirements before drawing boxes. Split into functional and non-functional.

**Functional requirements:** What does the system do?
- Core use cases (2-4 most important)
- User-facing features vs. internal features
- Data input/output formats
- Key workflows end-to-end

**Non-functional requirements:** How well must it do it?
- Scale: DAU, peak QPS, data volume
- Latency: p50, p99 targets
- Availability: 99.9%? 99.99%? (minutes of downtime per year)
- Consistency: strong or eventual? Which parts?
- Durability: can we lose data? RPO/RTO targets
- Security: auth, encryption, compliance constraints
- Cost: budget constraints, managed vs. self-hosted preference

Do not over-specify. Focus on what differentiates this system from a generic CRUD app.

### 2. Capacity Estimation

See `references/estimation-cheatsheet.md` for numbers and formulas.

Back-of-envelope estimation framework:
1. **Users:** DAU -> peak concurrent -> requests per second
2. **Storage:** data per record * records per day * retention period
3. **Bandwidth:** request size * QPS (inbound) + response size * QPS (outbound)
4. **Compute:** QPS / requests-per-instance = instance count

Key assumptions to state explicitly:
- Read:write ratio (most systems are 10:1 to 100:1)
- Peak to average ratio (typically 3x-10x)
- Data growth rate (linear, exponential?)
- Cache hit rate assumption (80-95% for read-heavy)

### 3. High-Level Design

Draw the system as 5-10 major components:

- Start with the client and work inward
- Identify the data flow for each core use case
- Place standard infrastructure: load balancer, API gateway, CDN, cache, database, queue
- Show the primary data flow with numbered arrows
- Identify the data store for each type of data (relational, document, blob, cache, search index)

Keep it simple first. Add complexity only when requirements demand it.

### 4. Component Deep Dives

For each critical component, detail:

- **API design:** endpoints, request/response formats, pagination strategy
- **Data model:** schema, indexing strategy, partitioning/sharding key
- **Scaling strategy:** horizontal vs. vertical, stateless vs. stateful
- **Failure handling:** what happens when this component goes down?
- **Caching:** what is cached, where (client, CDN, application, database), TTL, invalidation

Focus deep dives on the components that are most unique to this system or most likely to be bottlenecks.

### 5. Trade-Off Analysis

Every design decision involves trade-offs. Make them explicit:

- **Consistency vs. availability:** CAP theorem — in a partition, choose C or A
- **Latency vs. throughput:** batching increases throughput but adds latency
- **Simplicity vs. scalability:** simpler designs are easier to operate but may not scale
- **Cost vs. performance:** caching reduces latency but increases infrastructure cost
- **Flexibility vs. optimization:** denormalization speeds reads but complicates writes

For each trade-off: state which side you chose and why, given the requirements.

### 6. Scalability Patterns

Common patterns to apply when scaling:

- **Horizontal scaling:** stateless services behind a load balancer
- **Database sharding:** partition by user ID, tenant ID, or geographic region
- **Read replicas:** offload reads to replicas; accept replication lag
- **Caching layers:** local cache -> distributed cache (Redis) -> CDN
- **Async processing:** move non-critical work to background queues
- **Event-driven decoupling:** pub/sub to decouple producers from consumers
- **Rate limiting:** protect services from overload (token bucket, sliding window)
- **Circuit breaker:** prevent cascade failures when downstream is unhealthy

## Examples

### URL Shortener Design
User asks to design a URL shortener. Functional: create short URL, redirect, analytics. Non-functional: 100M URLs/month, <100ms redirect, 99.99% availability. Estimate: ~40 writes/s, ~4000 reads/s (100:1 ratio), ~10TB storage over 5 years. Design: API service (stateless) -> Redis cache (hot URLs) -> database (URL mapping, sharded by hash). Base62 encode a unique ID for short codes. CDN for redirect caching. Analytics via async event stream.

### Chat System Design
User asks to design a real-time chat system. Functional: 1:1 and group messaging, online status, message history. Non-functional: 50M DAU, <200ms delivery, eventual consistency for history. Estimate: 500K concurrent connections at peak, ~100K messages/s. Design: WebSocket gateway (stateful, sharded by user), message routing service, Cassandra for message storage (partitioned by chat_id), Redis for presence, push notification service for offline users.

### News Feed Design
User asks to design a social media news feed. Functional: post creation, feed generation, follow relationships. Non-functional: 500M DAU, feed loads in <500ms, eventual consistency acceptable. Estimate: 10K posts/s writes, 500K feed reads/s. Design: fan-out on write for users with <10K followers (precompute feeds in Redis), fan-out on read for celebrities (merge at read time). Post storage in sharded database, feed cache in Redis sorted sets, media in object storage behind CDN.
