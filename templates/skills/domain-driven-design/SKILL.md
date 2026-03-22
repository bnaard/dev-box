---
name: domain-driven-design
description: Domain-Driven Design strategic and tactical patterns. Bounded contexts, aggregates, value objects, and context mapping. Use when modeling complex domains, designing microservice boundaries, or reviewing domain models.
---

# Domain-Driven Design

## When to Use

- Modeling a complex business domain where the logic is the core differentiator
- Defining microservice boundaries based on business capabilities
- Reviewing an existing domain model for aggregate design or invariant violations
- Creating a ubiquitous language to align developers and domain experts
- Deciding how bounded contexts should communicate (context mapping)
- Refactoring a big ball of mud into well-defined domain boundaries

## Instructions

### 1. Establish Ubiquitous Language

Before writing any code, build a shared vocabulary with domain experts.

- Identify key domain terms and write them down with precise definitions
- Reject technical jargon in the domain model (no `EntityManager`, `ServiceHandler` — use domain terms like `Order`, `Shipment`, `PolicyRenewal`)
- If two teams use the same word differently, you likely have two bounded contexts
- Maintain a glossary in the project docs; update it as understanding evolves
- Code must use exactly the same terms — class names, method names, variable names all reflect the ubiquitous language

### 2. Strategic Design — Bounded Contexts

A bounded context is a boundary within which a domain model is consistent and a ubiquitous language applies.

How to identify bounded contexts:
- Map the business capabilities (ordering, billing, shipping, inventory)
- Look for linguistic boundaries — where the same word means different things (`Product` in catalog vs. warehouse)
- Align with team ownership — one team per bounded context is ideal
- Start coarse, split later — merging contexts is harder than splitting

Each bounded context:
- Has its own domain model (its own `Customer` class, not shared across contexts)
- Has its own data store (or at minimum its own schema)
- Communicates with other contexts through well-defined interfaces

### 3. Context Mapping

Define relationships between bounded contexts. See `references/ddd-building-blocks.md` for details.

| Pattern | Relationship | Use When |
|---|---|---|
| Shared Kernel | Two contexts share a small common model | Tightly coupled teams that co-evolve a small shared piece |
| Customer-Supplier | Upstream supplies, downstream consumes | Clear dependency direction, upstream accommodates downstream |
| Conformist | Downstream conforms to upstream's model | Cannot influence upstream (third-party API) |
| Anti-Corruption Layer (ACL) | Downstream translates upstream's model | Protecting your model from a legacy or external system |
| Open Host Service | Upstream provides a well-defined protocol | Many consumers; upstream publishes a stable API |
| Published Language | Shared interchange format (e.g., JSON schema) | Cross-context communication needs a neutral format |
| Separate Ways | No integration | Contexts have no meaningful interaction |

Default recommendation: use an Anti-Corruption Layer when integrating with any external system or legacy context. It prevents foreign concepts from leaking into your domain model.

### 4. Tactical Design — Aggregates

An aggregate is a cluster of domain objects treated as a single unit for data changes.

Rules for aggregate design:
- **One aggregate root** per aggregate — external references point only to the root
- **Protect invariants** — the aggregate enforces all business rules within its boundary
- **Small aggregates** — prefer smaller aggregates that reference each other by ID, not by object reference
- **Transactional boundary** — one transaction modifies one aggregate. Cross-aggregate consistency is eventual.
- **Identity** — aggregate roots have a globally unique ID; internal entities have locally unique IDs

Common mistake: making aggregates too large. If `Order` contains `Customer` contains `Address`, any change to an address locks the entire order. Instead: `Order` references `customerId`.

### 5. Tactical Design — Entities, Value Objects, Domain Events

**Entities** have identity and lifecycle. Two entities with the same attributes but different IDs are different.
- Example: `Order(id: OrderId)`, `User(id: UserId)`
- Implement equality by ID, not by attribute comparison

**Value Objects** have no identity. They are defined by their attributes and are immutable.
- Example: `Money(amount: Decimal, currency: Currency)`, `Address(street, city, zip)`
- Two value objects with the same attributes are equal
- Prefer value objects over primitives (use `EmailAddress` not `String`)

**Domain Events** represent something meaningful that happened in the domain.
- Named in past tense: `OrderPlaced`, `PaymentReceived`, `InventoryReserved`
- Immutable once created
- Carry enough data for consumers to react without querying back
- Published after the aggregate state change is persisted

### 6. Tactical Design — Repositories, Services, Factories

**Repositories** provide collection-like access to aggregates. One repository per aggregate root.
- Interface defined in the domain layer, implementation in infrastructure
- Methods: `find_by_id`, `save`, `find_by_criteria` — domain-oriented, not SQL-oriented
- Never expose query builders or ORM details through the repository interface

**Domain Services** contain business logic that does not naturally belong to a single entity or value object.
- Example: `TransferService.transfer(from: Account, to: Account, amount: Money)`
- Stateless — all state lives in entities and value objects
- Use sparingly — most logic should live on aggregates

**Factories** encapsulate complex object creation.
- Use when aggregate creation involves validation, default values, or multi-step initialization
- Can be a static method on the aggregate root or a separate factory class

### 7. Common Anti-Patterns

- **Anemic domain model:** Entities are data bags with getters/setters; all logic in services. Fix: move behavior onto entities and aggregates.
- **God aggregate:** One massive aggregate that locks everything. Fix: split into smaller aggregates connected by ID references.
- **Leaking context:** Using another context's types directly. Fix: introduce an ACL or map to local types.
- **Premature abstraction:** Defining bounded contexts before understanding the domain. Fix: start with a monolith, discover boundaries through usage.
- **Shared database:** Multiple bounded contexts reading/writing the same tables. Fix: give each context its own data store or schema.

## Examples

### Modeling an E-Commerce Domain
User asks to design a domain model for an online store. Identify bounded contexts: Catalog (product browsing), Ordering (cart + checkout), Payment, Shipping, Inventory. Define aggregates: `Product` (in Catalog), `Order` with `OrderLine` (in Ordering), `Shipment` (in Shipping). Use value objects for `Money`, `Address`, `SKU`. Map contexts: Ordering is customer of Inventory (reserve stock), Payment is separate with ACL to payment gateway.

### Reviewing Aggregate Boundaries
User has a `Customer` aggregate that contains `Orders`, `Addresses`, `PaymentMethods`, and `Preferences`. Identify that this is a god aggregate — modifying an address locks the entire customer. Recommend splitting: `Customer` (profile + preferences), `Order` (references customerId), `PaymentMethod` (references customerId). Each is its own aggregate with its own repository.

### Designing Microservice Boundaries
User wants to split a monolith into microservices. Map the existing code to business capabilities using event storming or domain narrative. Identify bounded contexts by looking for linguistic boundaries and team ownership. Propose context map showing relationships (ACLs at legacy boundaries, pub/sub events between contexts). Start by extracting the most independent context first.
