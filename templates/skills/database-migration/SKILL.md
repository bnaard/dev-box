---
name: database-migration
description: Schema migration workflows including zero-downtime migrations, data backfills, and rollback strategies. Use when planning database migrations, reviewing migration scripts, or troubleshooting migration failures.
allowed-tools: Bash Read Write
---

# Database Migration

## When to Use

When the user asks to:
- Write or review a schema migration (up/down)
- Plan a zero-downtime migration for a production database
- Backfill data after a schema change
- Set up migration tooling or version tracking
- Roll back a failed migration
- Avoid common migration pitfalls (locking, data loss)

## Instructions

### 1. Migration File Structure

Every migration has an `up` (apply) and `down` (revert) script. Use sequential
versioning or timestamps depending on your tool:

```
migrations/
  001_create_users.up.sql
  001_create_users.down.sql
  002_add_user_email_index.up.sql
  002_add_user_email_index.down.sql
```

Rules:
- One logical change per migration (do not mix unrelated alterations)
- The `down` migration must fully reverse the `up` migration
- Migrations are immutable once applied — never edit a deployed migration
- Test both directions: `up` then `down` then `up` again must leave the schema intact

### 2. Zero-Downtime Migrations (Expand/Contract)

Never make a breaking change in a single step. Use the expand/contract pattern:

**Phase 1 — Expand:** add the new structure alongside the old.
```sql
-- Migration: add new column (nullable, no default needed for existing rows)
ALTER TABLE users ADD COLUMN display_name TEXT;
```

**Phase 2 — Migrate:** backfill data and update application code.
```sql
-- Backfill in batches to avoid locking
UPDATE users SET display_name = name WHERE display_name IS NULL AND id BETWEEN 1 AND 10000;
UPDATE users SET display_name = name WHERE display_name IS NULL AND id BETWEEN 10001 AND 20000;
-- ... continue in batches
```
Deploy application code that writes to both `name` and `display_name`, reads from `display_name`.

**Phase 3 — Contract:** remove the old structure once fully migrated.
```sql
ALTER TABLE users ALTER COLUMN display_name SET NOT NULL;
-- Later, in a separate migration:
ALTER TABLE users DROP COLUMN name;
```

### 3. Safe Operations by Database

Not all ALTER TABLE operations are safe in all databases:

| Operation | PostgreSQL | MySQL (InnoDB) |
|-----------|-----------|----------------|
| Add nullable column | Fast, no lock | Fast (8.0+), locks in 5.7 |
| Add column with default | Fast (11+) | Rewrites table in 5.7, fast in 8.0+ |
| Drop column | Fast (marks invisible) | Rewrites table |
| Add index | `CONCURRENTLY` avoids lock | `ALGORITHM=INPLACE` for online |
| Rename column | Fast | Fast (8.0+) |
| Change column type | Rewrites table, locks | Rewrites table, locks |

**PostgreSQL-specific tips:**
- Always use `CREATE INDEX CONCURRENTLY` to avoid blocking writes
- `CONCURRENTLY` cannot run inside a transaction; set migration tool accordingly
- Adding a `NOT NULL` constraint with `NOT VALID` skips checking existing rows; validate later with `VALIDATE CONSTRAINT`

### 4. Data Backfills

Large backfills require care to avoid locking and overwhelming the database:

```sql
-- Batch update pattern (do this in application code or a script)
DO $$
DECLARE
    batch_size INT := 5000;
    max_id BIGINT;
BEGIN
    SELECT MAX(id) INTO max_id FROM users;
    FOR i IN 0..max_id/batch_size LOOP
        UPDATE users
        SET display_name = name
        WHERE display_name IS NULL
          AND id BETWEEN i * batch_size + 1 AND (i + 1) * batch_size;
        COMMIT;
        PERFORM pg_sleep(0.1);  -- throttle to reduce load
    END LOOP;
END $$;
```

Guidelines:
- Process in batches of 1,000-10,000 rows
- Add a short sleep between batches to let replication catch up
- Run during low-traffic periods for large tables
- Monitor replication lag and pause if it exceeds your threshold

### 5. Rollback Strategies

Plan for failure before running any migration:

- **Down migration**: the standard rollback path. Must be tested before deploying.
- **Forward-fix**: sometimes rolling back is riskier than fixing forward (e.g., after a destructive migration). Write a new migration that corrects the issue.
- **Point-in-time recovery (PITR)**: last resort. Restores the entire database to a timestamp. Data written after that point is lost.

Before running migrations in production:
1. Take a backup or confirm PITR is enabled
2. Run the migration on a staging copy with production-like data
3. Measure execution time on staging (a 2-second migration on staging may take 20 minutes on production)
4. Have the rollback command ready to execute

### 6. Version Tracking

Migration tools track which migrations have been applied in a metadata table:

```sql
CREATE TABLE schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Common tools: Flyway, Liquibase, golang-migrate, Alembic (Python), Knex (Node),
Diesel (Rust), Ecto (Elixir). All follow the same pattern: ordered migrations
with a tracking table.

### 7. Common Pitfalls

- **Locking a large table**: `ALTER TABLE ... ADD COLUMN ... DEFAULT x` rewrites the table in older PostgreSQL/MySQL. Use nullable + backfill instead.
- **Missing index creation for new FKs**: foreign keys without indexes cause slow deletes on the parent table.
- **Running DDL inside a long transaction**: in PostgreSQL, DDL takes `ACCESS EXCLUSIVE` locks; if combined with long-running queries, this causes cascading lock waits.
- **Forgetting to handle NULL during backfill**: application code deployed before backfill completes must handle NULL in the new column.
- **Irreversible migrations without backup**: dropping columns or tables with no backup and no down migration is data loss waiting to happen.

## Examples

**User:** "I need to rename the `username` column to `handle` without downtime"
**Agent:** Plans a 3-step expand/contract migration: (1) add `handle` column, (2) deploy code that writes to both and reads from `handle`, backfill `handle` from `username`, (3) drop `username` once all reads are migrated. Writes the three migration files with up/down scripts and a backfill script that processes 5,000 rows per batch.

**User:** "This migration is taking forever on production"
**Agent:** Checks the migration SQL for table-rewriting operations (column type changes, adding a column with a volatile default). Identifies that `ALTER TABLE orders ADD COLUMN total DECIMAL NOT NULL DEFAULT 0` is rewriting a 50M-row table. Recommends splitting into: add nullable column (instant), backfill in batches, then add NOT NULL constraint with `NOT VALID` + `VALIDATE CONSTRAINT`.

**User:** "How do I add an index to a 100M row table without locking?"
**Agent:** Writes `CREATE INDEX CONCURRENTLY idx_orders_customer ON orders (customer_id);` and explains that this must run outside a transaction block. Configures the migration tool to disable transaction wrapping for this migration. Estimates build time based on table size and recommends running during low-traffic hours. Includes the down migration: `DROP INDEX CONCURRENTLY idx_orders_customer;`.
