---
name: refactoring
description: Systematic code refactoring using Fowler's catalog, Gang of Four patterns, and code smell detection. Use when restructuring code without changing behavior.
---

# Refactoring

## When to Use

When the user asks to:
- Refactor, clean up, or restructure code
- Fix code smells (long method, feature envy, shotgun surgery, etc.)
- Apply design patterns to simplify complex code
- Reduce duplication or improve readability
- Prepare code for a new feature by improving its structure first

## Instructions

### 1. Verify Safety Net Before Refactoring

Before any refactoring:

1. **Check for tests**: Run existing tests to confirm they pass. If no tests exist for the code being refactored, write characterization tests first (tests that capture current behavior, not desired behavior).
2. **Plan small steps**: Break the refactoring into a sequence of atomic transformations. Each step must keep tests green.
3. **Commit after each step**: Each refactoring move gets its own commit, separate from behavior changes.

Golden rule: **Never mix refactoring with feature changes in the same commit.**

### 2. Identify Code Smells

Systematically scan for smells (see `references/code-smells.md` for the full catalog):

| Smell | What to Look For |
|---|---|
| **Long Method** | Function > 20 lines or does more than one thing |
| **Large Class** | Class with > 5 responsibilities or > 300 lines |
| **Long Parameter List** | Function with > 3 parameters |
| **Feature Envy** | Method uses another object's data more than its own |
| **Data Clumps** | Same group of variables appears together repeatedly |
| **Primitive Obsession** | Using strings/ints where a domain type belongs |
| **Shotgun Surgery** | One change requires edits in many unrelated files |
| **Divergent Change** | One class changed for multiple unrelated reasons |
| **Duplicate Code** | Same logic in two or more places |
| **Dead Code** | Unreachable or unused code |

### 3. Apply Refactoring Patterns

Match smells to specific refactorings from Fowler's catalog:

**Composing Methods:**
- **Extract Function**: Pull a block into a named function (fixes Long Method)
- **Inline Function**: Remove trivial single-use wrappers
- **Extract Variable**: Name a complex expression for clarity
- **Replace Temp with Query**: Replace a local variable with a method call

**Moving Features:**
- **Move Function/Field**: Relocate to the class that uses it most (fixes Feature Envy)
- **Extract Class**: Split a class with multiple responsibilities (fixes Large Class)
- **Inline Class**: Merge a class that does too little

**Simplifying Conditionals:**
- **Replace Nested Conditional with Guard Clauses**: Early returns reduce nesting
- **Decompose Conditional**: Extract condition and branches into named functions
- **Replace Conditional with Polymorphism**: Use strategy/state pattern for type-based switches
- **Introduce Null Object**: Eliminate null checks with a no-op implementation

**Organizing Data:**
- **Introduce Parameter Object**: Group related parameters into a struct/class
- **Replace Magic Number with Constant**: Name unexplained literals
- **Encapsulate Collection**: Return copies or iterators, not raw mutable collections

**Generalization:**
- **Pull Up / Push Down Method**: Move methods to the right level in the hierarchy
- **Extract Interface/Trait**: Define a contract for polymorphism
- **Replace Inheritance with Composition**: Prefer delegation over subclassing

### 4. Apply GoF Patterns When They Simplify Code

Only introduce a design pattern when it solves a concrete problem, never speculatively. See `references/gof-patterns.md` for the full catalog. Most commonly useful patterns:

| Pattern | When It Helps |
|---|---|
| **Strategy** | Replace conditional logic selecting algorithms at runtime |
| **Observer** | Decouple event producers from consumers |
| **Factory Method** | Centralize complex object creation logic |
| **Builder** | Construct complex objects step-by-step with optional fields |
| **Adapter** | Integrate a third-party library behind your own interface |
| **Decorator** | Add behavior without modifying existing code |
| **Command** | Encapsulate operations for undo, queuing, or logging |
| **State** | Replace complex state-dependent conditionals |
| **Template Method** | Share algorithm structure, vary specific steps |
| **Iterator** | Provide uniform traversal over custom collections |

Warning signs of pattern overuse:
- Adding a pattern "just in case" (speculative generality)
- Pattern adds more code than it removes
- Only one implementation of an interface exists with no plans for more
- The team cannot explain why the pattern is there

### 5. Modern Refactoring Considerations

For async/concurrent code:
- Extract async boundaries clearly (sync core, async shell)
- Replace callback chains with async/await where possible
- Isolate side effects at the edges

For functional-style code:
- Replace mutable loops with map/filter/reduce pipelines
- Extract pure functions from impure ones
- Use algebraic types (enums/unions) over class hierarchies for closed sets

## Examples

**User:** "This function is too long, clean it up"
**Agent:** Identifies three responsibilities in the function. Extracts each into a named helper with descriptive names, keeping the original as a high-level orchestrator. Runs tests after each extraction. Commits each extraction separately.

**User:** "There's too much duplication between these two modules"
**Agent:** Identifies the shared logic, extracts it into a common module with a clear interface. Uses Extract Function and Move Function. Verifies both call sites work with the extracted version. Checks for subtle differences that might need parameterization.

**User:** "This switch statement keeps growing"
**Agent:** Recognizes the switch dispatches on a type code. Applies Replace Conditional with Polymorphism by introducing a trait/interface with implementations for each case. Uses the Strategy pattern if the behavior varies at runtime, or simple polymorphism if types are known at compile time.

**User:** "Make this code more testable"
**Agent:** Identifies concrete dependencies (file I/O, HTTP calls, database). Extracts interfaces/traits at the boundaries (Adapter pattern). Moves domain logic into pure functions. Introduces dependency injection so tests can provide fakes.
