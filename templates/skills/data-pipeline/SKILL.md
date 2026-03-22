---
name: data-pipeline
description: Data pipeline patterns including ETL/ELT, batch vs streaming, idempotency, and orchestration. Use when designing data pipelines, reviewing data workflows, or troubleshooting data processing.
allowed-tools: Bash Read Write
---

# Data Pipeline

## When to Use

When the user is designing a data pipeline, choosing between batch and streaming,
implementing data ingestion or transformation, setting up orchestration, or asks
"how should I structure this data workflow?". Also applies when debugging pipeline
failures or improving reliability.

## Instructions

### 1. ETL vs ELT

- **ETL** (Extract, Transform, Load): Transform before loading into the target. Use when
  the target system has limited compute or data must be cleaned before storage.
- **ELT** (Extract, Load, Transform): Load raw data first, transform in the target. Use
  when the target is a modern data warehouse (BigQuery, Snowflake, Redshift) with cheap compute.
- Prefer ELT for analytics workloads — it preserves raw data and allows re-transformation
- Use ETL when data governance requires filtering PII before it reaches the warehouse
- Never mix transformation logic between the pipeline and the warehouse without documenting which is source of truth

### 2. Batch vs Streaming

- **Batch**: Process data in scheduled intervals (hourly, daily). Simpler to build, debug, and retry.
- **Streaming**: Process data as it arrives. Required for real-time dashboards, alerting, event-driven systems.
- Start with batch unless there is an explicit real-time requirement
- Micro-batch (Spark Structured Streaming, 1-5 min intervals) bridges the gap
- Streaming adds complexity: ordering, late arrivals, exactly-once semantics — only pay that cost when needed

### 3. Idempotency

- Every pipeline step must be safely re-runnable without duplicating data
- Use upserts (INSERT ON CONFLICT UPDATE) or merge statements for loading
- Partition output by date/run_id; overwrite entire partitions on rerun
- Include a deduplication step: hash the natural key columns, deduplicate before loading
- Design so that running the same job twice produces the same result as running it once

### 4. Data Quality Checks

- Validate at each stage: after extraction, after transformation, after loading
- Check row counts: source count should match destination (within tolerance)
- Check for nulls in required columns, duplicates in unique keys
- Validate value ranges: dates in expected range, amounts non-negative
- Fail fast: halt the pipeline on quality violations rather than propagating bad data
- Log quality metrics for every run: row counts, null rates, duplicate rates

### 5. Schema Management

- Use a schema registry for event-driven pipelines (Avro, Protobuf, JSON Schema)
- Enforce backward compatibility: new fields are optional, never remove or rename
- Version your schemas: `v1`, `v2` with explicit migration paths
- Document every field: name, type, description, example value, nullable
- Test schema changes against consumers before deploying to production

### 6. Backfill Strategies

- Design pipelines to accept a date range parameter from the start
- Use partition overwrite for backfills: reprocess and replace entire partitions
- Run backfills at lower priority to avoid starving production workloads
- Test backfills on a small date range first, then scale up
- Track backfill progress: log which partitions have been reprocessed

### 7. Orchestration

- Use a workflow orchestrator (Airflow, Prefect, Dagster) for scheduling and dependency management
- Define DAGs with explicit dependencies: task B waits for task A
- Set appropriate retries with exponential backoff: 3 retries, 1/5/15 minute delays
- Use sensors or triggers rather than polling where possible
- Tag tasks with owners and SLA expectations
- Keep task definitions small and testable; business logic lives in separate modules

### 8. Monitoring and Dead Letter Queues

- Alert on: pipeline failure, SLA breach, unexpected row count changes (> 20% deviation)
- Log pipeline metadata: start time, end time, rows processed, rows rejected
- Use dead letter queues (DLQ) for records that fail processing
- DLQ records must include: original payload, error message, timestamp, source topic/table
- Review and reprocess DLQ records regularly — they represent data loss until resolved

## Examples

**User:** "Design a pipeline to load daily sales data from an API into our warehouse"
**Agent:** Designs an ELT pipeline with three stages: extract (API call with pagination,
save raw JSON to cloud storage), load (bulk insert raw data into a staging table),
transform (SQL in the warehouse to clean, deduplicate, and join with dimension tables).
Adds date-range parameters for backfills, idempotent loads via partition overwrite,
row-count checks between source and staging, and Airflow DAG with retries.

**User:** "Our pipeline keeps producing duplicate records"
**Agent:** Diagnoses the idempotency gap: checks whether the load step uses insert-only
(the problem) vs upsert. Adds a natural key hash column, implements INSERT ON CONFLICT
for the target table, and adds a pre-load deduplication step. Adds a data quality check
that compares distinct key counts between source and target.

**User:** "Should we use streaming or batch for our event tracking?"
**Agent:** Evaluates the requirements: if events only feed daily reports, recommends
batch with hourly micro-batches as a compromise. If real-time alerting is needed,
recommends streaming with Kafka, but warns about exactly-once complexity. In either
case, designs for idempotency and includes a dead letter queue for malformed events.
