---
name: concurrency-patterns
description: Concurrency and parallelism patterns including async/await, threads, actors, channels, and deadlock prevention. Use when designing concurrent systems, debugging race conditions, or choosing between concurrency models.
---

# Concurrency Patterns

## When to Use

When the user asks to:
- Choose between threads, async/await, or actors for a problem
- Design a concurrent or parallel data pipeline
- Debug deadlocks, race conditions, or data corruption
- Implement producer-consumer, worker pool, or fan-out/fan-in patterns
- Reason about thread safety, synchronization, or shared state
- Optimize throughput by parallelizing work

## Instructions

### 1. Distinguish Concurrency from Parallelism

- **Concurrency**: structuring a program to handle multiple tasks (may interleave on one core)
- **Parallelism**: executing multiple tasks simultaneously on multiple cores

Choose based on the bottleneck:
- **I/O-bound** (network, disk, database): concurrency (async/await or green threads) — one thread can handle thousands of connections
- **CPU-bound** (computation, data processing): parallelism (OS threads or processes) — use all available cores
- **Mixed**: combine both — async for I/O, thread pool for CPU work

### 2. Choose the Right Concurrency Model

| Model | Best For | Watch Out For |
|---|---|---|
| **async/await** | High-concurrency I/O (web servers, API clients) | Don't block the event loop with CPU work |
| **OS threads** | CPU-bound parallelism, simple shared state | Overhead per thread (~2-8MB stack), limited scalability |
| **Green threads / goroutines** | Massive concurrency with simple code (Go, Erlang) | Runtime scheduler quirks, blocking FFI calls |
| **Actors** | Distributed systems, fault isolation | Complexity in ordering guarantees, debugging |
| **Channels / message passing** | Decoupled producer-consumer pipelines | Backpressure management, channel capacity |
| **Thread pool + work queue** | Bounded parallelism for batch jobs | Queue sizing, graceful shutdown |

See `references/patterns-catalog.md` for detailed pattern descriptions.

### 3. Manage Shared State Safely

Prefer message passing over shared state. When shared state is necessary:

**Mutex (mutual exclusion)**: one writer OR one reader at a time.
```rust
let data = Arc::new(Mutex::new(HashMap::new()));
// In each thread:
let mut guard = data.lock().unwrap();
guard.insert(key, value);
// guard drops here, releasing the lock
```

**RwLock (reader-writer lock)**: many concurrent readers OR one writer.
Use when reads vastly outnumber writes.

**Atomic operations**: lock-free updates for simple values (counters, flags).
```rust
let counter = Arc::new(AtomicU64::new(0));
counter.fetch_add(1, Ordering::Relaxed);
```

Rules:
- Hold locks for the **shortest possible duration**
- Never perform I/O or allocations while holding a lock
- Prefer **`Arc<Mutex<T>>`** (Rust) or **`synchronized`** (Java) over manual lock management
- Use **concurrent data structures** (ConcurrentHashMap, DashMap) when available

### 4. Prevent Deadlocks

Deadlock requires four conditions (Coffman): mutual exclusion, hold-and-wait, no preemption, circular wait. Break any one to prevent deadlocks:

- **Lock ordering**: always acquire locks in a consistent global order
- **Try-lock with timeout**: use `try_lock()` with a deadline, back off and retry
- **Reduce lock scope**: restructure to avoid holding multiple locks simultaneously
- **Lock-free algorithms**: use atomics and CAS (compare-and-swap) for simple operations

Detection: enable deadlock detection in development (Rust: `parking_lot` with deadlock detection, Go: `GODEBUG=mutexprofile`, Java: `jstack` thread dumps).

### 5. Detect and Fix Race Conditions

Race conditions occur when correctness depends on timing of uncontrolled events.

Detection tools:
- **Rust**: compile-time prevention via ownership system; use `cargo miri` for unsafe code
- **Go**: `go run -race` / `go test -race` (ThreadSanitizer)
- **C/C++**: ThreadSanitizer (`-fsanitize=thread`), Helgrind
- **Java**: FindBugs/SpotBugs, `-XX:+UseThreadSanitizer`

Common patterns that indicate races:
- Check-then-act without holding a lock (TOCTOU)
- Publishing a partially initialized object
- Updating a shared collection without synchronization
- Using a non-atomic read-modify-write sequence

### 6. Implement Backpressure

When producers are faster than consumers, unbounded queues lead to memory exhaustion:

- **Bounded channels**: block or reject when the channel is full
- **Rate limiting**: control the rate of incoming work
- **Load shedding**: drop low-priority work under pressure
- **Reactive streams**: propagate demand signals upstream

```go
// Go: bounded channel provides natural backpressure
jobs := make(chan Job, 100)  // blocks producer when 100 items queued
```

## Examples

**User:** "Should I use threads or async for my web scraper?"
**Agent:** Since web scraping is I/O-bound (waiting for HTTP responses), recommends async/await with a semaphore to limit concurrency to 50 simultaneous requests. Shows an implementation using `tokio::spawn` with `tokio::sync::Semaphore` in Rust, or `asyncio.gather` with `asyncio.Semaphore` in Python.

**User:** "Our Go service deadlocks under high load"
**Agent:** Enables mutex profiling with `GODEBUG=mutexprofile`, identifies two goroutines acquiring locks on `userCache` and `sessionCache` in opposite orders. Fixes by establishing a consistent lock ordering (always `sessionCache` before `userCache`) and reducing the critical section to avoid holding both locks simultaneously.

**User:** "How do I process 10 million records in parallel?"
**Agent:** Designs a fan-out/fan-in pipeline: a reader goroutine streams records into a bounded channel, a pool of N worker goroutines processes them in parallel, and a collector goroutine aggregates results. Sets N to `runtime.NumCPU()` and channel buffer to `N * 2` for optimal throughput with backpressure.
