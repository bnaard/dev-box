---
name: go-conventions
description: Go idioms and conventions including error handling, interfaces, goroutine patterns, and testing. Use when writing Go code, reviewing Go projects, or designing Go package layouts.
allowed-tools: Bash(go:*) Read Write
---

# Go Conventions

## When to Use

When the user is working with Go code and asks about idiomatic patterns, error handling,
concurrency, package organization, testing strategies, or says "how should I structure
this in Go?". Also applies when reviewing Go code for correctness and style.

## References

- `references/go-patterns.md` — Common patterns with code: functional options, builder, middleware, graceful shutdown

## Instructions

### 1. Error Handling

- Return errors as the last return value; never panic in library code
- Wrap errors with context: `fmt.Errorf("loading config: %w", err)`
- Use `errors.Is(err, target)` for sentinel errors, `errors.As(err, &target)` for typed errors
- Define sentinel errors with `var ErrNotFound = errors.New("not found")` at package level
- Use custom error types only when callers need to extract structured information
- Never ignore errors silently; assign to `_` only with a comment explaining why

### 2. Interfaces

- Accept interfaces, return concrete types
- Keep interfaces small: 1-3 methods is ideal (the Go proverb: "the bigger the interface, the weaker the abstraction")
- Define interfaces where they are consumed, not where they are implemented
- Use `io.Reader`, `io.Writer`, `fmt.Stringer` and other stdlib interfaces before inventing new ones
- Name single-method interfaces with the `-er` suffix: `Reader`, `Closer`, `Validator`

### 3. Goroutine Patterns

- Always pass `context.Context` as the first parameter to functions that may block or do I/O
- Use `errgroup.Group` for fan-out/fan-in — it handles error propagation and cancellation
- Launch goroutines with clear ownership: the creator is responsible for lifecycle
- Use buffered channels for worker pools; size the buffer based on expected throughput
- Always select on `ctx.Done()` alongside channel operations to respect cancellation
- Never start a goroutine without knowing how it will stop

### 4. Package Layout

- Organize by domain, not by layer: `user/`, `order/`, not `models/`, `handlers/`
- Package names should be short, lowercase, singular: `http`, `user`, `config`
- Avoid `util`, `common`, `helpers` packages — find a more specific home
- Keep `main.go` minimal: parse flags, wire dependencies, call `run(ctx)`
- Internal packages go under `internal/` to prevent external imports
- One package per directory; no multi-package directories

### 5. Testing

- Use table-driven tests for any function with multiple input/output combinations
- Name test cases descriptively: `{name: "empty input returns error", ...}`
- Use `testify/assert` for readable assertions, `testify/require` for fatal checks
- Use `httptest.NewServer` for HTTP client testing; `httptest.NewRecorder` for handler testing
- Use `t.Helper()` in test helper functions so failure line numbers point to the caller
- Run `go test -race ./...` in CI to catch data races
- Use `t.Parallel()` for independent tests to speed up the suite

### 6. Go Proverbs

- Don't communicate by sharing memory; share memory by communicating
- A little copying is better than a little dependency
- Clear is better than clever
- Make the zero value useful
- Errors are values — handle them, don't just check them
- The empty interface says nothing

### 7. Code Style

- Follow `gofmt` unconditionally; use `goimports` for import grouping
- Group imports: stdlib, then a blank line, then third-party, then internal
- Exported names get doc comments: `// UserService manages user lifecycle.`
- Use named return values sparingly — only when they clarify the API
- Prefer early returns to reduce nesting: guard clauses at the top

## Examples

**User:** "This function has a lot of if err != nil blocks, can you clean it up?"
**Agent:** Wraps errors with `fmt.Errorf` and `%w` for context at each call site,
extracts repeated setup into helper functions that return errors, and ensures the
caller receives enough context to diagnose failures without reading the source.

**User:** "I need a worker pool that processes jobs from a channel"
**Agent:** Creates a pool using `errgroup.Group` with a configurable number of workers,
each reading from a shared job channel. Uses `context.Context` for cancellation and
returns the first error encountered. Includes a graceful shutdown that drains
remaining jobs. See `references/go-patterns.md` for the full pattern.

**User:** "How should I organize this Go microservice?"
**Agent:** Proposes a domain-driven layout with `internal/` for business logic, `cmd/`
for the entry point, interfaces defined at consumption sites, and dependency injection
via constructor functions. Adds a `Makefile` with `build`, `test`, `lint` targets.
