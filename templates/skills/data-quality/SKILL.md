---
name: data-quality
description: Data quality framework covering completeness, accuracy, consistency, validation rules, and data contracts. Use when implementing data validation, setting up data quality checks, or defining data contracts.
allowed-tools: Bash Read Write
---

# Data Quality

## When to Use

When the user is implementing data validation, setting up quality checks for a pipeline,
defining data contracts between teams, or asks "how do I ensure data quality?" or
"what checks should I add to this pipeline?". Also applies when investigating data
anomalies or setting up data observability.

## Instructions

### 1. Data Quality Dimensions

Evaluate data against six dimensions:

- **Completeness**: Are all expected records and fields present? Measure: null rate per column, row count vs expected.
- **Accuracy**: Do values reflect reality? Measure: spot checks against source of truth, known reference data.
- **Consistency**: Do related fields agree? Measure: cross-field validation (end_date >= start_date, city matches zip code).
- **Timeliness**: Is data available when needed? Measure: freshness (time since last update), SLA adherence.
- **Uniqueness**: Are there duplicates? Measure: distinct count of natural keys vs total row count.
- **Validity**: Do values conform to expected formats and ranges? Measure: regex matches, range checks, enum membership.

### 2. Validation Rules

- **Schema validation**: Column names, data types, nullable flags match the contract
- **Range checks**: Numeric values within expected bounds (age 0-150, price >= 0)
- **Format checks**: Dates in ISO 8601, emails match regex, phone numbers normalized
- **Referential integrity**: Foreign keys resolve to existing records in parent tables
- **Business rules**: Order total = sum of line items, start_date < end_date
- **Freshness checks**: Last record timestamp within expected window
- Categorize rules by severity: `error` (block pipeline) vs `warning` (log and continue)

### 3. Anomaly Detection

- Track key metrics over time: row counts, null rates, distinct value counts, mean/median
- Alert when metrics deviate beyond a threshold (e.g., row count drops > 20% vs previous run)
- Use statistical methods: z-score for normally distributed metrics, IQR for skewed
- Monitor distribution shifts: column value distributions should be stable over time
- Track schema changes: new columns, type changes, dropped columns
- Check for sudden spikes in null rates or default values

### 4. Data Contracts

- A data contract is a formal agreement between producer and consumer on data shape and quality
- Define: schema (columns, types), SLAs (freshness, availability), quality thresholds, ownership
- Version contracts: breaking changes require a new version with migration period
- Contracts should be machine-readable (JSON Schema, protobuf, YAML) and enforced automatically
- Include: who to contact when the contract is violated, escalation path
- Review contracts quarterly or when business requirements change

### 5. Great Expectations Patterns

- Organize expectations into suites: one per table or pipeline stage
- Core expectations to always include:
  - `expect_table_row_count_to_be_between` — catch empty or exploding tables
  - `expect_column_values_to_not_be_null` — for required columns
  - `expect_column_values_to_be_unique` — for natural keys
  - `expect_column_values_to_be_in_set` — for enum columns
- Run validation as a pipeline step; fail the pipeline on critical expectation failures
- Store validation results for trend analysis and audit trails

### 6. Data Observability

- Instrument pipelines with metadata: row counts, schema snapshots, execution time
- Build a lineage graph: know where each dataset comes from and what depends on it
- Dashboard key health indicators: freshness, volume trends, error rates per table
- Set up alerts for: stale data, volume anomalies, schema drift, quality rule failures
- Automate root cause analysis: when quality drops, trace back through lineage to find the source

### 7. Implementing Quality Checks

- Place checks at pipeline boundaries: after extraction, after transformation, after loading
- Run checks before and after: compare source row count to destination row count
- Store check results in a quality metadata table: timestamp, check name, result, details
- Start with the five critical checks: row count, null rate on key columns, duplicate rate,
  freshness, schema match. Add more as issues arise.
- Make checks configurable: thresholds in config files, not hardcoded

## Examples

**User:** "Set up data quality checks for our customer table"
**Agent:** Implements checks across all six dimensions: completeness (null rate for
email, name, created_at), uniqueness (customer_id has no duplicates), validity
(email matches regex, status in allowed enum), consistency (country matches
postal code format), timeliness (most recent created_at within 24 hours), and
accuracy (spot check sample against CRM export). Categorizes each as error or
warning severity and stores results in a quality_checks table.

**User:** "Our daily report numbers look wrong but the pipeline succeeded"
**Agent:** Adds anomaly detection to the pipeline: tracks row counts and key
aggregates over time, alerts when values deviate more than 2 standard deviations
from the 30-day rolling average. Adds cross-field consistency checks (order totals
match line item sums) and a freshness check to ensure all source tables updated
before the report runs.

**User:** "Define a data contract between the events team and analytics"
**Agent:** Creates a versioned contract in YAML specifying: schema (column names,
types, nullable), quality thresholds (null rate < 1% for user_id, row count >
1000/day), freshness SLA (data available by 06:00 UTC), ownership (events team
as producer, analytics as consumer), and contact/escalation information. Adds
automated validation that runs on each delivery and notifies both teams on violations.
