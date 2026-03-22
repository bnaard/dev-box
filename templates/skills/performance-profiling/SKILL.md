---
name: performance-profiling
description: Performance analysis methodology and profiling techniques for CPU, memory, and I/O. Flame graphs, benchmarking, and regression detection. Use when optimizing performance, profiling bottlenecks, or reviewing performance-critical code.
allowed-tools: Bash Read Write
---

# Performance Profiling

## When to Use

When the user asks to:
- Identify performance bottlenecks in an application
- Profile CPU usage, memory allocation, or I/O operations
- Interpret flame graphs or profiler output
- Set up benchmarking or performance regression detection
- Optimize a slow function, endpoint, or pipeline
- Review performance-critical code paths

## Instructions

### 1. Follow the Performance Analysis Cycle

Always work through the full cycle: **Identify -> Measure -> Profile -> Optimize -> Verify**.

1. **Identify** the performance concern: slow response time, high memory usage, excessive CPU, I/O stalls
2. **Measure** the current baseline with concrete numbers (latency p50/p95/p99, throughput, memory RSS)
3. **Profile** with the right tool for the resource type (see `references/profiling-tools.md`)
4. **Optimize** the top bottleneck only — do not scatter-shot optimize
5. **Verify** the improvement with the same measurement from step 2

Never skip measurement. Intuition about performance is unreliable — always profile before optimizing.

### 2. CPU Profiling

Identify what the application spends CPU time on:

1. Use a sampling profiler (not an instrumenting profiler) to minimize overhead
2. Collect a profile for a representative workload — not a trivial test case
3. Generate a flame graph for visual analysis
4. Look for: wide stacks (hot functions), deep stacks (excessive abstraction), unexpected functions (regex compilation, serialization, logging)

Key metric: CPU time per operation, not wall-clock time (which includes I/O waits).

### 3. Memory Profiling

Identify allocation pressure and leaks:

1. Track peak RSS and allocation rate, not just current usage
2. For leaks: take heap snapshots at intervals and diff them
3. Look for: growing collections without bounds, caches without eviction, event listener accumulation, reference cycles (in GC languages)
4. Distinguish between a leak (unbounded growth) and high-but-stable usage (may be acceptable)

### 4. I/O Profiling

Identify disk, network, or database bottlenecks:

1. Measure wall-clock time vs CPU time — large gaps indicate I/O waits
2. For database: enable slow query logging, check for N+1 queries, missing indexes
3. For network: check connection pooling, DNS resolution, TLS handshake overhead
4. For disk: check for synchronous writes, excessive fsync, unneeded file operations

### 5. Benchmarking and Regression Detection

Set up reproducible, reliable benchmarks:

1. Isolate the code under test — benchmark the function, not the setup
2. Run enough iterations for statistical significance (report mean, stddev, and percentiles)
3. Control for external factors: disable turbo boost, pin CPU frequency, close other processes
4. For regression detection: store baseline results in version control and compare on each run

```bash
# Example: Rust criterion benchmark structure
cargo bench -- --save-baseline main
# After changes:
cargo bench -- --baseline main
```

### 6. Flame Graph Interpretation

When reading flame graphs:

- **X-axis** is alphabetical (not time), width = proportion of samples
- **Y-axis** is stack depth, read bottom-up (caller -> callee)
- **Wide boxes** at the top are hot leaf functions — optimize these first
- **Wide boxes** at the bottom are hot callers — consider algorithmic changes
- Look for unexpected frames: GC pauses, lock contention, allocator overhead
- Use differential flame graphs (red/blue) to compare before/after

### 7. Common Optimization Patterns

After profiling reveals the bottleneck, apply the right fix:

- **Algorithmic**: O(n^2) -> O(n log n) — always check this first
- **Batching**: combine many small operations into fewer large ones
- **Caching**: reuse computed results (but measure cache hit rate)
- **Pooling**: reuse expensive resources (connections, threads, buffers)
- **Lazy evaluation**: defer work until actually needed
- **Data layout**: struct-of-arrays vs array-of-structs for cache friendliness

### 8. Quick Profiling Checklist

Before diving in, answer these questions to focus your effort:

1. What is the **user-visible symptom**? (slow page, high bill, OOM crash)
2. What **resource** is constrained? (CPU, memory, disk I/O, network, database)
3. Is the issue **constant** or **intermittent**? (steady-state vs spike-triggered)
4. What changed recently? (`git log --since="1 week ago"` — regressions often correlate with recent deploys)
5. What is the **acceptable target**? (p99 < 200ms, RSS < 512MB — you need a goal to know when you are done)

If you cannot answer question 2, start with a broad observability check: CPU usage, memory RSS, disk IOPS, and network bytes over the past hour.

## Examples

**User:** "This API endpoint takes 3 seconds to respond"
**Agent:** Measures end-to-end latency, then profiles the handler. Discovers 80% of time is spent in 47 sequential database queries (N+1 problem). Rewrites as a single JOIN query with eager loading, reducing response time to 120ms. Verifies with the same load profile.

**User:** "Our service memory keeps growing until it OOMs"
**Agent:** Attaches a memory profiler and takes heap snapshots at 1-minute intervals. Diffs reveal a map used for request deduplication that is never pruned. Adds a TTL-based eviction policy, confirms RSS stabilizes at 400MB under sustained load.

**User:** "I need to set up benchmarks for our Rust library"
**Agent:** Sets up criterion benchmarks for the hot-path functions, configures `cargo bench` with statistical thresholds, and adds a CI step that compares against the stored baseline and fails if any benchmark regresses by more than 5%.
