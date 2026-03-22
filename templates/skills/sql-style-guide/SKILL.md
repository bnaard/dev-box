---
name: sql-style-guide
description: SQL formatting and naming conventions for tables, columns, queries, migrations, and constraints. Use when writing SQL, reviewing database code, or establishing SQL style guidelines.
---

# SQL Style Guide

## When to Use

When the user is writing SQL queries, designing database schemas, creating migration
files, or asks "how should I format this SQL?" or "what naming convention should I use
for tables?". Also applies when reviewing SQL for consistency.

## Instructions

### 1. Table and Column Naming

- Use `snake_case` for all identifiers: `user_account`, `created_at`
- Table names are singular: `user`, `order`, `product` (not `users`, `orders`)
- Join tables combine both names: `user_role`, `order_product`
- Boolean columns use `is_` or `has_` prefix: `is_active`, `has_subscription`
- Timestamp columns use `_at` suffix: `created_at`, `updated_at`, `deleted_at`
- Avoid reserved words as identifiers; if unavoidable, do not quote — rename instead
- Primary keys are `id`; foreign keys are `<referenced_table>_id`: `user_id`, `order_id`

### 2. Keyword Capitalization

- SQL keywords in UPPERCASE: `SELECT`, `FROM`, `WHERE`, `JOIN`, `INSERT`
- Function names in UPPERCASE: `COUNT()`, `COALESCE()`, `NOW()`
- Identifiers (tables, columns) in lowercase
- Be consistent within a project — pick one style and enforce it

### 3. Query Formatting

- One clause per line: `SELECT`, `FROM`, `WHERE`, `JOIN`, `ORDER BY` each on their own line
- Use leading commas in select lists for easier diffing:
  ```sql
  SELECT
      u.id
    , u.name
    , u.email
    , u.created_at
  FROM user u
  ```
- Indent join conditions and subqueries by 4 spaces
- Align `ON` with its `JOIN`:
  ```sql
  FROM order o
  JOIN user u ON u.id = o.user_id
  JOIN product p ON p.id = o.product_id
  ```
- Use explicit `JOIN` syntax — never comma joins in `FROM`
- Use table aliases that are meaningful abbreviations: `u` for user, `o` for order

### 4. Comment Conventions

- Use `--` for single-line comments explaining why, not what
- Add a header comment to complex queries explaining the business purpose
- Comment non-obvious filter conditions: `WHERE status = 3 -- 3 = 'completed'`
- In migrations, comment the purpose of each structural change

### 5. Migration File Naming

- Use sequential timestamps: `20260322_001_create_user_table.sql`
- One structural change per migration (do not mix schema and data changes)
- Always include both `up` and `down` migrations
- Name describes the action: `add_email_to_user`, `create_order_table`, `drop_legacy_column`

### 6. Constraint Naming

- Primary key: `pk_<table>` — `pk_user`
- Foreign key: `fk_<table>_<referenced_table>` — `fk_order_user`
- Unique: `uq_<table>_<columns>` — `uq_user_email`
- Check: `ck_<table>_<description>` — `ck_order_positive_total`
- Index: `ix_<table>_<columns>` — `ix_user_created_at`
- Always name constraints explicitly — never rely on auto-generated names

### 7. Query Best Practices

- Use `WHERE EXISTS` instead of `WHERE IN (SELECT ...)` for correlated subqueries
- Use CTEs (`WITH`) to break complex queries into readable steps
- Avoid `SELECT *` — list columns explicitly
- Use `COALESCE` for default values, not application-level null handling
- Always specify `ORDER BY` when query results must be deterministic

## Examples

**User:** "Create a schema for a task management app"
**Agent:** Designs tables with singular names (`task`, `project`, `user`), snake_case
columns, explicit constraint names (`pk_task`, `fk_task_project`, `uq_user_email`),
timestamp columns with `_at` suffix, and boolean columns with `is_` prefix. Includes
a migration file with proper naming.

**User:** "Format this messy SQL query"
**Agent:** Reformats with one clause per line, leading commas in the select list,
UPPERCASE keywords, meaningful table aliases, explicit JOIN syntax, and adds a header
comment explaining the query's business purpose.

**User:** "Review this migration file"
**Agent:** Checks for named constraints, singular table names, snake_case columns,
both up and down sections, and that the migration contains a single structural change.
Flags any auto-generated constraint names and suggests explicit alternatives.
