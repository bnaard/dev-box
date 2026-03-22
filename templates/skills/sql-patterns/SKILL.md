---
name: sql-patterns
description: SQL query patterns, schema design, and optimization. Joins, CTEs, window functions, indexing, and anti-patterns. Use when writing SQL queries, designing schemas, optimizing database performance, or reviewing database code.
allowed-tools: Bash Read Write
---

# SQL Patterns

## When to Use

When the user asks to:
- Write or optimize SQL queries (joins, subqueries, CTEs, window functions)
- Design or review a database schema
- Analyze query performance using EXPLAIN plans
- Choose indexing strategies for a workload
- Fix slow queries or identify anti-patterns
- Implement pagination, deduplication, or analytical queries

## Instructions

### 1. Query Construction

Build queries incrementally, starting with the core logic before adding complexity.

**Joins** - choose the right type for the data relationship:
- `INNER JOIN` — only matching rows in both tables (default, most common)
- `LEFT JOIN` — all rows from left table, NULLs where right has no match
- `RIGHT JOIN` — all rows from right table (rare; rewrite as LEFT JOIN for clarity)
- `FULL OUTER JOIN` — all rows from both, NULLs on the unmatched side
- `CROSS JOIN` — cartesian product (use intentionally for generating combinations)

**CTEs (WITH clauses)** — prefer CTEs over nested subqueries for readability:
```sql
WITH active_users AS (
    SELECT user_id, MAX(login_at) AS last_login
    FROM logins
    WHERE login_at > CURRENT_DATE - INTERVAL '30 days'
    GROUP BY user_id
),
user_orders AS (
    SELECT user_id, COUNT(*) AS order_count, SUM(total) AS revenue
    FROM orders
    GROUP BY user_id
)
SELECT u.name, au.last_login, uo.order_count, uo.revenue
FROM users u
JOIN active_users au USING (user_id)
LEFT JOIN user_orders uo USING (user_id);
```

**Window functions** — use for rankings, running totals, and row comparisons:
```sql
SELECT
    department,
    employee,
    salary,
    RANK() OVER (PARTITION BY department ORDER BY salary DESC) AS dept_rank,
    salary - LAG(salary) OVER (PARTITION BY department ORDER BY salary) AS diff_from_prev,
    SUM(salary) OVER (PARTITION BY department) AS dept_total
FROM employees;
```

**Aggregations** — remember the evaluation order: `WHERE` filters rows, `GROUP BY` groups them, `HAVING` filters groups, window functions run after `HAVING`.

### 2. Query Optimization

When optimizing, follow this process:

1. **Read the EXPLAIN plan** — look for sequential scans on large tables, nested loops on large result sets, sort operations without indexes
2. **Check indexes** — ensure WHERE, JOIN, and ORDER BY columns are indexed
3. **Reduce data early** — filter in WHERE before joining; join on indexed columns
4. **Avoid anti-patterns:**
   - `SELECT *` in production queries (fetches unnecessary columns)
   - Functions on indexed columns in WHERE (`WHERE YEAR(created_at) = 2025` defeats the index; use range instead)
   - `NOT IN` with NULLable columns (use `NOT EXISTS` instead)
   - Correlated subqueries that execute per-row (rewrite as JOIN or CTE)
   - Missing LIMIT on exploratory queries
   - Implicit type conversions that prevent index use

### 3. Indexing Strategy

Choose index types based on query patterns:
- **B-tree** (default) — equality, range, ORDER BY, LIKE 'prefix%'
- **Hash** — equality only, smaller than B-tree (Postgres-specific)
- **GIN** — full-text search, JSONB containment, array operations
- **GiST** — geometric data, range types, nearest-neighbor
- **Composite indexes** — column order matters; leftmost prefix rule applies
- **Partial indexes** — `WHERE is_active = true` for queries that always filter on that condition
- **Covering indexes** — `INCLUDE` columns to enable index-only scans

### 4. Transactions and Concurrency

- Use explicit transactions for multi-statement operations
- Choose the right isolation level: READ COMMITTED (default, good for most), REPEATABLE READ (consistent snapshots), SERIALIZABLE (full isolation, retry on conflict)
- Keep transactions short — avoid user interaction inside a transaction
- Use `SELECT ... FOR UPDATE` when reading rows you intend to modify
- Handle deadlocks with retry logic, not just longer timeouts

### 5. Schema Design Basics

Refer to `references/schema-design.md` for detailed patterns. Key principles:
- Normalize to 3NF by default, denormalize intentionally with a documented reason
- Use surrogate keys (auto-increment or UUID) as primary keys
- Add `created_at` and `updated_at` timestamps to all tables
- Use foreign keys to enforce referential integrity
- Name constraints explicitly for readable error messages

See `references/query-patterns.md` for common query recipes.

## Examples

**User:** "Write a query to find the top 3 products per category by revenue"
**Agent:** Uses a CTE with `ROW_NUMBER()` window function partitioned by category, ordered by revenue descending, then filters to `row_num <= 3`. Includes the JOIN to get category and product names, and suggests an index on `(category_id, revenue DESC)`.

**User:** "This query is slow, can you optimize it?"
**Agent:** Runs `EXPLAIN ANALYZE` on the query, identifies a sequential scan on a 2M-row table due to a function call on an indexed column in the WHERE clause. Rewrites the condition to use a range comparison, confirms the index is now used, and shows the before/after execution times.

**User:** "How should I index this table for our read patterns?"
**Agent:** Analyzes the common query patterns (filter by status + date range, search by email, sort by created_at), recommends a composite index `(status, created_at)`, a unique index on `email`, and a partial index for `WHERE status = 'pending'` since 80% of queries filter on that. Explains the trade-off with write performance.
