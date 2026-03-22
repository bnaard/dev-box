---
name: java-patterns
description: Modern Java 17+ patterns including records, sealed classes, Stream API, and Spring Boot conventions. Use when writing Java code, reviewing Java projects, or modernizing legacy Java.
allowed-tools: Bash(mvn:*) Bash(gradle:*) Read Write
---

# Java Patterns

## When to Use

When the user is working with Java code and asks about modern language features,
Spring Boot conventions, Stream API patterns, testing strategies, or says "how should
I modernize this Java code?". Also applies when reviewing Java for idiomatic usage.

## Instructions

### 1. Modern Java Language Features (17+)

- Use **records** for immutable data carriers: `record Point(int x, int y) {}`
- Use **sealed classes** to restrict type hierarchies: `sealed interface Shape permits Circle, Rectangle {}`
- Use **pattern matching** with `instanceof`: `if (obj instanceof String s)` — no cast needed
- Use **switch expressions** with pattern matching for exhaustive type handling
- Use **text blocks** for multi-line strings: `""" ... """` with proper indentation
- Prefer `var` for local variables when the type is obvious from the right-hand side

### 2. Records and Sealed Classes

- Records replace boilerplate DTOs: automatic `equals`, `hashCode`, `toString`
- Add validation in the compact constructor: `record Age(int value) { Age { if (value < 0) throw new IllegalArgumentException(); } }`
- Sealed classes + records create algebraic data types:
  ```java
  sealed interface Result<T> permits Success, Failure {}
  record Success<T>(T value) implements Result<T> {}
  record Failure<T>(String error) implements Result<T> {}
  ```
- Use sealed hierarchies with switch expressions for exhaustive handling

### 3. Stream API Patterns

- Prefer streams for transformations, not for side effects
- Use `toList()` (Java 16+) instead of `Collectors.toList()`
- Chain: `filter` -> `map` -> `collect` is the most common pipeline
- Use `flatMap` to flatten nested collections
- Use `groupingBy` and `partitioningBy` for aggregation
- Avoid `.get()` on Optional — use `orElseThrow()`, `orElse()`, or `ifPresent()`
- Never use streams for simple iteration where a for-loop is clearer

### 4. Optional Usage

- Return `Optional<T>` from methods that may not produce a result
- Never use `Optional` as a field type or method parameter
- Chain with `map`, `flatMap`, `filter` instead of `isPresent()` + `get()`
- Use `orElseThrow(() -> new NotFoundException(...))` for required values
- Use `Optional.empty()` instead of returning `null`

### 5. Spring Boot Conventions

- Use constructor injection (not field injection with `@Autowired`)
- Keep controllers thin: validate input, delegate to service, return response
- Use `@Transactional` at the service layer, not the controller
- Define configuration in `application.yml` with typed `@ConfigurationProperties` classes
- Use profiles (`application-dev.yml`, `application-prod.yml`) for environment differences
- Return `ResponseEntity<T>` from controllers for explicit status codes
- Use `@RestControllerAdvice` for centralized exception handling

### 6. Dependency Injection

- Prefer constructor injection: makes dependencies explicit and enables testing
- Use interfaces for service contracts; inject the interface, not the implementation
- Avoid circular dependencies — they indicate a design problem
- Use `@Qualifier` or custom annotations to disambiguate multiple implementations
- Keep the number of constructor parameters small (< 5); too many suggests the class does too much

### 7. Testing with JUnit 5 and Mockito

- Use `@Test`, `@DisplayName("descriptive name")` for readable test output
- Use `@ParameterizedTest` with `@CsvSource` or `@MethodSource` for data-driven tests
- Mock dependencies with `@Mock` and inject with `@InjectMocks`
- Use `@ExtendWith(MockitoExtension.class)` instead of the runner
- Use `assertThrows(Exception.class, () -> ...)` for exception testing
- Prefer `AssertJ` fluent assertions: `assertThat(result).isEqualTo(expected)`
- Test slices: `@WebMvcTest` for controllers, `@DataJpaTest` for repositories

### 8. Code Organization

- Package by feature (`com.app.order`, `com.app.user`), not by layer
- Keep classes focused: one primary responsibility per class
- Use `final` on classes and fields where immutability is intended
- Avoid deep inheritance hierarchies — prefer composition and interfaces
- Use `module-info.java` for explicit module boundaries in larger projects

## Examples

**User:** "Convert this class with getters/setters to modern Java"
**Agent:** Replaces the POJO with a `record`, removes boilerplate getters, setters,
equals, hashCode, and toString methods. Adds a compact constructor for validation
if the original had validation logic. Updates all call sites to use the record's
accessor methods (no `get` prefix).

**User:** "This switch statement handling message types keeps growing, how do I fix it?"
**Agent:** Introduces a sealed interface for the message types with record
implementations for each variant. Replaces the switch statement with a switch
expression using pattern matching, which the compiler checks for exhaustiveness.
Adding a new type now forces handling at compile time.

**User:** "Write tests for this Spring Boot service"
**Agent:** Creates a test class with `@ExtendWith(MockitoExtension.class)`, mocks
the repository dependency, and writes tests for the happy path, not-found case,
and validation failure. Uses `@DisplayName` for readable names and AssertJ for
assertions. Adds a `@ParameterizedTest` for the validation rules.
