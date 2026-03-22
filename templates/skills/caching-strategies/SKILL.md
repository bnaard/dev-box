---
name: caching-strategies
description: Caching patterns including cache-aside, write-through, TTL strategies, cache invalidation, and HTTP caching. Use when designing caching layers, optimizing response times, or debugging cache-related issues.
---

# Caching Strategies

## When to Use

When the user asks to:
- Add caching to reduce latency or database load
- Choose between caching patterns for a specific use case
- Configure HTTP caching headers (Cache-Control, ETag, CDN)
- Debug stale data, cache inconsistency, or cache stampede issues
- Design a cache invalidation strategy
- Warm caches for predictable performance

## Instructions

### 1. Choose the Right Caching Pattern

Select based on read/write ratio and consistency requirements:

**Cache-Aside (Lazy Loading)** — most common, application manages the cache:
```
Read: check cache -> miss -> read DB -> populate cache -> return
Write: update DB -> invalidate cache (do NOT update cache)
```
Best for: read-heavy workloads, data that tolerates brief staleness. Simple to implement.

**Read-Through** — cache itself loads from the data source on miss:
```
Read: check cache -> miss -> cache reads DB internally -> return
```
Best for: when you want the cache library to own the loading logic. Reduces application complexity.

**Write-Through** — writes go through cache to the data source:
```
Write: update cache -> cache writes to DB synchronously -> return
```
Best for: data that must always be fresh in cache. Higher write latency but strong consistency.

**Write-Behind (Write-Back)** — cache writes to the data source asynchronously:
```
Write: update cache -> return immediately -> cache flushes to DB later
```
Best for: write-heavy workloads. Risk of data loss if cache fails before flush.

### 2. Design TTL Strategies

TTL (Time-to-Live) is the primary tool for bounding staleness:

- **Short TTL (1-60s)**: rapidly changing data (stock prices, active sessions)
- **Medium TTL (5-60min)**: user profiles, product listings, API responses
- **Long TTL (hours-days)**: static assets, configuration, reference data
- **No TTL (manual invalidation)**: data that changes only through known events

Apply **jitter** to TTLs to prevent thundering herd on expiration:
```python
ttl = base_ttl + random.uniform(0, base_ttl * 0.1)  # +/- 10% jitter
```

### 3. Cache Invalidation

The two hard problems in computer science: cache invalidation and naming things.

**Event-driven invalidation** (preferred when possible):
- On write/update/delete, explicitly invalidate or update the cache entry
- Use pub/sub or change data capture (CDC) to propagate invalidations across services

**Tag-based invalidation**:
- Tag cache entries by entity (e.g., `user:123`, `product:456`)
- Invalidate all entries with a given tag when that entity changes

**Versioned keys**:
```
cache_key = f"user:{user_id}:v{version}"
# Increment version on change — old entries expire naturally via TTL
```

Rules:
- Prefer **invalidation over update** (avoids race conditions)
- Always set a **TTL as a safety net**, even with explicit invalidation
- For distributed caches, accept that invalidation is **eventually consistent**

### 4. HTTP Caching

Configure HTTP caching headers correctly:

```
# Immutable static assets (hashed filenames)
Cache-Control: public, max-age=31536000, immutable

# API responses that vary by user
Cache-Control: private, max-age=0, must-revalidate
ETag: "a1b2c3"

# Shared content (CDN-cacheable)
Cache-Control: public, s-maxage=3600, max-age=60, stale-while-revalidate=300
```

**ETag / Last-Modified** — conditional requests for revalidation:
- Server returns `ETag` or `Last-Modified` header
- Client sends `If-None-Match` / `If-Modified-Since` on next request
- Server returns `304 Not Modified` if unchanged (no body transfer)

**CDN caching** — add `s-maxage` to control CDN TTL independently of browser TTL. Use `Surrogate-Key` headers for tag-based CDN purging (Fastly, Varnish).

### 5. Prevent Cache Stampede

When a popular key expires, hundreds of requests can hit the database simultaneously.

**Locking (mutex)**:
```python
value = cache.get(key)
if value is None:
    if cache.acquire_lock(f"lock:{key}", timeout=5):
        value = db.query(key)
        cache.set(key, value, ttl=300)
        cache.release_lock(f"lock:{key}")
    else:
        value = cache.get(key)  # retry — another process is loading
```

**Probabilistic early expiration (XFetch)**:
```python
# Recompute before TTL expires, with increasing probability as expiry approaches
if time_remaining < ttl * 0.1 * random.random():
    recompute_and_cache(key)
```

**Stale-while-revalidate**: serve stale data immediately while refreshing in the background.

### 6. Cache Warming

Pre-populate caches before traffic hits:

- On deploy: warm caches for the top-N most accessed keys
- On cache flush: use a warming script that replays recent access patterns
- For predictable access: pre-compute and cache during off-peak hours

Do not warm everything — warm only keys with high access frequency and expensive computation.

## Examples

**User:** "Our product page loads in 800ms, we need it under 200ms"
**Agent:** Profiles the endpoint and finds 3 database queries per request. Implements cache-aside with Redis: product data (TTL 10min), category tree (TTL 1hr), user-specific pricing (TTL 60s, private). Adds `stale-while-revalidate` to HTTP headers. Response time drops to 45ms on cache hit.

**User:** "Users see stale data after updating their profile"
**Agent:** Finds the profile cache uses a 15-minute TTL with no explicit invalidation. Adds event-driven invalidation: the profile update endpoint now deletes the cache key after writing to the database. Retains the TTL as a safety net for edge cases.

**User:** "Our cache hit rate dropped from 95% to 60% after scaling to 4 servers"
**Agent:** Discovers each server runs its own in-memory cache, so the same key is cached independently on each instance. Migrates to a shared Redis cache with consistent hashing, restoring the hit rate to 94%. Adds cache stampede protection with a mutex pattern for the most expensive queries.
