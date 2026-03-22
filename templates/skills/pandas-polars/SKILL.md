---
name: pandas-polars
description: DataFrame operations with pandas and polars including groupby, joins, reshaping, and performance optimization. Use when manipulating tabular data, choosing between pandas and polars, or optimizing DataFrame code.
allowed-tools: Bash(python:*) Read Write
---

# Pandas and Polars

## When to Use

When the user is working with tabular data, asks about DataFrame operations,
data transformations, or says "clean this data" or "aggregate by group" or
"should I use pandas or polars?"

## Instructions

### 1. Choosing Between Pandas and Polars

- **Pandas**: mature ecosystem, broader library compatibility (sklearn, plotly, etc.),
  familiar API, best for exploratory work and smaller datasets (<1GB)
- **Polars**: faster execution, lower memory usage, lazy evaluation, better for
  production pipelines and larger datasets (1GB+)
- Both read/write the same formats (CSV, Parquet, JSON, Excel)
- Polars has no index — use explicit columns instead
- If the project already uses one, prefer consistency over switching

### 2. DataFrame Creation and I/O

- Prefer Parquet over CSV for performance and type preservation
- Use `dtype` / `schema` parameters when reading to enforce types upfront
- For large CSVs, read in chunks (pandas) or use lazy scanning (polars)
- Always specify `parse_dates` or date columns explicitly — auto-detection is unreliable
- Use `usecols` / `columns` to read only needed columns for memory efficiency

### 3. Selection and Filtering

- Select columns by name, not position — more readable and maintainable
- In polars, use expressions: `df.select(pl.col("name"), pl.col("age"))`
- In pandas, use `.loc[]` for label-based, `.iloc[]` for position-based
- Chain filters with `&` (and), `|` (or) in both libraries; wrap conditions in parentheses
- Polars `filter()` is preferred over boolean indexing for clarity

### 4. GroupBy and Aggregation

- Always name aggregated columns explicitly for clarity
- Polars: `df.group_by("category").agg(pl.col("price").mean().alias("avg_price"))`
- Pandas: `df.groupby("category")["price"].mean().reset_index(name="avg_price")`
- Use multiple aggregations in one pass for efficiency
- Polars supports `over()` for window functions without groupby: `pl.col("x").mean().over("group")`
- Avoid iterating over groups with loops — use vectorized aggregations

### 5. Joins and Merges

- Specify join type explicitly: `how="inner"`, `"left"`, `"outer"`, `"cross"`, `"anti"`, `"semi"`
- Use `on=` for same-named keys, `left_on=`/`right_on=` for different names
- Check for duplicates before joining to avoid row explosion
- Polars anti-join is efficient for "rows not in other table" queries
- Use `validate="m:1"` (pandas) to catch unexpected duplicates

### 6. Reshaping (Pivot and Melt)

- Pivot: wide format (columns from values) — for summary tables and reporting
- Melt/unpivot: long format (values into rows) — for analysis and plotting
- Polars: `df.pivot(on="category", index="date", values="sales")`
- Pandas: `df.pivot_table(index="date", columns="category", values="sales", aggfunc="sum")`
- Use `stack()`/`unstack()` in pandas for multi-level index reshaping

### 7. Missing Data

- Check missing: `df.null_count()` (polars), `df.isna().sum()` (pandas)
- Fill strategies: `fill_null()` / `fillna()` with value, forward-fill, backward-fill
- Drop rows: `df.drop_nulls(subset=["critical_col"])` — only drop when appropriate
- Interpolation for time series: `df["col"].interpolate()` (pandas)
- Document why missing data exists — it affects analysis validity

### 8. String Operations

- Polars: `pl.col("name").str.to_lowercase()`, `.str.contains("pattern")`
- Pandas: `df["name"].str.lower()`, `.str.contains("pattern")`
- Use regex sparingly — literal string operations are faster
- Strip whitespace early: `.str.strip_chars()` (polars), `.str.strip()` (pandas)
- Extract patterns with `.str.extract()` — useful for parsing structured strings

### 9. Datetime Operations

- Parse dates on read: specify format for speed and correctness
- Polars: `pl.col("date").dt.year()`, `.dt.month()`, `.dt.weekday()`
- Pandas: `df["date"].dt.year`, `.dt.month`, `.dt.dayofweek`
- Resample time series: pandas `resample("1h").mean()`, polars `group_by_dynamic()`
- Always be explicit about timezones — naive datetimes cause subtle bugs

### 10. Performance Optimization

- Use lazy evaluation in polars: `df.lazy().filter(...).select(...).collect()`
- Polars optimizes query plans — push filters and projections down automatically
- In pandas, avoid `apply()` with Python functions — use vectorized operations
- Use categorical dtype for low-cardinality string columns
- Pre-sort before groupby for better cache locality (polars does this automatically)
- Profile with `%timeit` and check memory with `df.estimated_size()` (polars)

## References

- `references/api-comparison.md` — Side-by-side pandas vs polars for common operations
- [Pandas Documentation](https://pandas.pydata.org/docs/)
- [Polars User Guide](https://docs.pola.rs/)

## Examples

**User:** "Read a large CSV, filter rows, and compute group statistics"
**Agent:** Uses polars lazy mode to scan the CSV, applies filters before collection,
groups by the requested column with multiple aggregations in one `.agg()` call,
sorts results, and writes output to Parquet for downstream use.

**User:** "Convert this pandas code to polars"
**Agent:** Maps pandas operations to polars equivalents: replaces `.loc[]` with
`filter()`/`select()`, converts `groupby().agg()` to polars `group_by().agg()` with
`pl.col()` expressions, removes index-based operations, and adds `.lazy()`/`.collect()`
for optimized execution.

**User:** "Clean this messy dataset: duplicates, missing values, inconsistent formats"
**Agent:** Drops exact duplicates, identifies columns with high null rates, fills or
drops nulls based on context, normalizes string columns (lowercase, strip whitespace),
parses dates with explicit format, and validates value ranges with filter expressions.
