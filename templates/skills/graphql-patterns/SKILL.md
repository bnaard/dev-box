---
name: graphql-patterns
description: GraphQL schema design, resolver patterns, N+1 prevention with DataLoader, and federation. Use when designing GraphQL APIs, implementing resolvers, or optimizing GraphQL performance.
allowed-tools: Bash Read Write
---

# GraphQL Patterns

Design, implement, and optimize GraphQL APIs — schema design, resolver architecture,
performance tuning, federation, and schema evolution.

## When to Use

- Designing a GraphQL schema (types, queries, mutations, subscriptions).
- Implementing resolvers and data fetching logic.
- Diagnosing or preventing N+1 query problems.
- Adding pagination to list fields.
- Setting up federation across microservices.
- Evolving a schema without breaking clients.

## Instructions

### Schema Design

Design from the client perspective, not the database schema:

- **Types**: domain entities as nouns (`User`, `Order`).
- **Queries**: read operations — `user(id: ID!)`, `orders(filter: OrderFilter)`.
- **Mutations**: write operations — verb-object naming: `createOrder`, `cancelOrder`.
- **Subscriptions**: real-time streams — `orderStatusChanged(orderId: ID!)`.

Prefer non-nullable fields (`String!`) by default. Use custom scalars for domain values
(`DateTime`, `Email`). Define `input` types for mutation arguments.

### Resolver Patterns

- **Root resolvers** handle top-level Query/Mutation fields.
- **Field resolvers** handle nested fields from a different data source than the parent.
- **Default resolvers** return `parent[fieldName]` — do not write trivial resolvers.

Keep resolvers thin: extract business logic into service/domain layers.

### N+1 Problem and DataLoader

A list query triggering one query per item is the N+1 problem. **DataLoader** batches
and caches within a single request:

```javascript
const orderLoader = new DataLoader(async (userIds) => {
  const orders = await db.orders.findByUserIds(userIds);
  return userIds.map(id => orders.filter(o => o.userId === id));
});
// In User.orders resolver: (user) => orderLoader.load(user.id)
```

- Create a new DataLoader instance **per request** (prevents cross-request cache leaks).
- Batch function must return results in the **same order** as input keys.

### Pagination (Relay Connection Spec)

Use cursor-based pagination for list fields:

```graphql
type OrderConnection {
  edges: [OrderEdge!]!
  pageInfo: PageInfo!
}
type OrderEdge { cursor: String!; node: Order! }
type PageInfo { hasNextPage: Boolean!; hasPreviousPage: Boolean!; endCursor: String }
```

Cursors should be opaque (base64-encoded). Default `first` to 20, enforce max 100.

### Error Handling

**Domain errors as union types** (preferred for expected errors):
```graphql
union CreateOrderResult = Order | ValidationError | InsufficientStock
```
Clients handle errors with type-safe pattern matching instead of parsing error strings.

Use top-level `errors` array for unexpected failures (auth, server errors). Include
`extensions.code` (e.g., `UNAUTHENTICATED`) for machine-readable classification.

### Federation

For microservice architectures (Apollo Federation):
- Each service owns a subgraph with `@key` directives on shared entities.
- Gateway composes subgraphs into a supergraph at build time.
- A subgraph only extends fields it can resolve.

### Schema Evolution

- **Adding** fields/types/enum values is always safe.
- **Deprecating**: `@deprecated(reason: "Use newField")`. Monitor usage before removal.
- **Never** change a field's type or make a nullable field non-nullable.

### Persisted Queries

Use automatic persisted queries (APQ) in production: client sends a query hash, server
looks it up. Reduces bandwidth and enables query allowlisting for security.

## Examples

### Example 1: Design a schema for a task management app

```
User: Design a GraphQL schema for projects, tasks, and team members.

Agent: Designs types (Project, Task, User, Comment), queries (projects,
  project(id), myTasks), mutations (createTask, assignTask, addComment),
  subscription (taskUpdated). Relay-style pagination on task lists, union
  result types for mutation errors, input types for all arguments.
```

### Example 2: Fix N+1 performance problem

```
User: Fetching 50 projects takes 3 seconds.

Agent: Identifies N+1: 1 query for projects + 50 for owner + 50 for
  taskCount. Implements DataLoader for both fields — ownerLoader batches
  user IDs, taskCountLoader batches project IDs. Response drops to 120ms.
```

### Example 3: Evolve schema to replace a deprecated field

```
User: Split User.name into firstName and lastName without breaking clients.

Agent: Adds firstName/lastName, marks name as @deprecated, resolver returns
  concatenation for compatibility. Monitors usage, removes after adoption >98%.
```
