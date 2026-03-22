---
name: tdd-workflow
description: Test-Driven Development workflow with red-green-refactor cycle, test naming, and when to apply TDD. Use when writing tests first, practicing TDD, or choosing between TDD and test-after approaches.
allowed-tools: Bash Read Write
---

# TDD Workflow

## When to Use

When the user asks to:
- Write code using test-driven development
- Practice the red-green-refactor cycle
- Decide between TDD and test-after approaches
- Name or organize tests effectively
- Understand test doubles (mocks, stubs, spies, fakes)
- Explore property-based or mutation testing

## Instructions

### 1. Red-Green-Refactor Cycle

Follow the cycle strictly in this order:

1. **Red** -- Write a failing test that describes the desired behavior. Run it. Confirm it fails for the right reason (not a syntax error or import issue).
2. **Green** -- Write the minimum code to make the test pass. Do not optimize, do not handle edge cases yet. The goal is a green bar as fast as possible.
3. **Refactor** -- Clean up both production code and test code. Remove duplication, improve naming, extract helpers. All tests must stay green.

Rules of thumb:
- Never write production code without a failing test
- Each cycle should take 2-10 minutes; if longer, the step is too big
- Commit after each green-refactor pair for easy rollback

### 2. When to Use TDD vs Test-After

**Use TDD when:**
- Implementing well-understood business rules or algorithms
- Building APIs where the contract is known upfront
- Fixing bugs (write a test that reproduces the bug first)
- Working with pure functions or isolated modules

**Use test-after when:**
- Prototyping or exploring an unfamiliar problem space
- Writing integration tests against third-party systems
- The design is highly uncertain and likely to change completely
- Working with UI layout code where visual feedback is primary

### 3. Outside-In vs Inside-Out

**Outside-in (London school):** Start from the outermost layer (HTTP handler, CLI entry point). Use test doubles for dependencies. Build inward, defining interfaces as you go. Best for systems with clear layered architecture.

**Inside-out (Chicago school):** Start from the core domain logic with no dependencies. Build outward, composing tested units. Uses real objects where possible. Best for algorithmic or data-heavy code.

### 4. Test Naming Conventions

Use names that describe behavior, not implementation:

- `test_empty_cart_has_zero_total` (good -- describes behavior)
- `test_calculate` (bad -- vague, says nothing about expectation)
- `should_reject_negative_quantities` (good -- states the rule)
- Pattern: `test_<scenario>_<expected_outcome>` or `should_<expected_behavior>_when_<condition>`

### 5. Test Doubles

| Double | Purpose | When to Use |
|--------|---------|-------------|
| **Stub** | Returns canned data | Replace a dependency that provides input |
| **Mock** | Verifies interactions | Assert that a method was called with specific args |
| **Spy** | Records calls for later assertion | When you need both real behavior and verification |
| **Fake** | Working implementation (e.g., in-memory DB) | When stubs become too complex |

Prefer fakes and stubs over mocks. Excessive mocking couples tests to implementation.

### 6. Property-Based Testing

Instead of testing specific examples, define properties that must always hold:

- **Invariants**: `sort(xs).length == xs.length` -- sorting preserves length
- **Round-trips**: `decode(encode(x)) == x` -- serialization is reversible
- **Idempotence**: `f(f(x)) == f(x)` -- applying twice gives same result

Use property-based testing alongside example-based tests, not as a replacement.

### 7. Mutation Testing

Mutation testing measures test quality by introducing small changes (mutations) to production code and checking whether tests catch them:

- **Killed mutant**: A test failed, meaning the test suite detected the change
- **Survived mutant**: No test failed, revealing a gap in test coverage
- Tools: `mutmut` (Python), `cargo-mutants` (Rust), `Stryker` (JS/TS)

A high mutation score (>80%) indicates tests verify behavior, not just execute code.

## Examples

### Example 1: Full Red-Green-Refactor Cycle (Python)

```python
# RED: Write failing test
def test_fizzbuzz_returns_fizz_for_multiples_of_3():
    assert fizzbuzz(3) == "Fizz"
    assert fizzbuzz(9) == "Fizz"

# Run: pytest => NameError: name 'fizzbuzz' is not defined (FAILS -- good)

# GREEN: Minimal implementation
def fizzbuzz(n):
    if n % 3 == 0:
        return "Fizz"
    return str(n)

# Run: pytest => PASS

# REFACTOR: Add next rule via new failing test
def test_fizzbuzz_returns_buzz_for_multiples_of_5():
    assert fizzbuzz(5) == "Buzz"

# Run: pytest => AssertionError (FAILS -- good, next RED phase)
```

### Example 2: Outside-In with Stubs (TypeScript)

```typescript
// Start from the handler, stub the repository
describe("CreateOrder", () => {
  it("should persist order and return confirmation", async () => {
    const repo = { save: vi.fn().mockResolvedValue({ id: "ord-1" }) };
    const handler = new CreateOrderHandler(repo);

    const result = await handler.execute({ item: "widget", qty: 2 });

    expect(result.id).toBe("ord-1");
    expect(repo.save).toHaveBeenCalledWith(
      expect.objectContaining({ item: "widget", qty: 2 })
    );
  });
});
// Now implement CreateOrderHandler to make this pass (GREEN).
// Then write the real repository and its own unit tests (inside-out from here).
```

### Example 3: Property-Based Test (Rust)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn reverse_twice_is_identity(ref v in prop::collection::vec(any::<i32>(), 0..100)) {
        let mut reversed = v.clone();
        reversed.reverse();
        reversed.reverse();
        assert_eq!(v, &reversed);
    }

    #[test]
    fn sort_preserves_length(ref v in prop::collection::vec(any::<i32>(), 0..100)) {
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(v.len(), sorted.len());
    }
}
```
