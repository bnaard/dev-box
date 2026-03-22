---
name: data-science
description: Data analysis workflow from import through modeling and communication. Covers tidy data, EDA, statistical reasoning, and visualization. Use when analyzing datasets, building statistical models, exploring data, or communicating findings.
allowed-tools: Bash(python:*) Bash(jupyter:*) Read Write
---

# Data Science

## When to Use

When the user asks to analyze a dataset, explore data, build a statistical model,
create visualizations, or communicate findings from data. Also applies when cleaning
messy data, selecting features, or choosing between modeling approaches.

## References

- `references/tidy-data-principles.md` — Tidy data rules, messy data fixes, pandas/polars examples
- `references/statistical-methods.md` — Distributions, hypothesis testing, regression guidance
- `references/visualization-guidelines.md` — Chart selection, accessibility, matplotlib/seaborn/plotly

## Instructions

### 1. Data Import and Cleaning

- Inspect shape, dtypes, nulls, and duplicates immediately after loading
- Print `df.info()` and `df.describe()` before any analysis
- Handle missing data explicitly: drop, fill, or flag — never ignore silently
- Parse dates at load time (`parse_dates=`), set correct dtypes early
- Validate assumptions: check for negative values where impossible, string/numeric mismatches

### 2. Tidy Data

- Reshape data to tidy form before analysis: one variable per column, one observation per row
- Use `melt()` for wide-to-long, `pivot_table()` for long-to-wide
- Separate mixed columns (e.g., "city_state" into two columns)
- See `references/tidy-data-principles.md` for patterns and code

### 3. Exploratory Data Analysis (EDA)

- Start with univariate distributions: histograms, value counts, box plots
- Then bivariate: scatter plots, grouped bar charts, correlation matrices
- Look for outliers, skewness, class imbalance, and unexpected patterns
- Summarize findings in plain language before moving to modeling
- Use `.groupby()` aggregations to understand subgroups

### 4. Statistical Reasoning

- State the question before choosing a test
- Check assumptions (normality, equal variance, independence) before applying parametric tests
- Report effect sizes alongside p-values — statistical significance is not practical significance
- Use confidence intervals to communicate uncertainty
- See `references/statistical-methods.md` for method selection

### 5. Feature Selection

- Remove zero-variance and near-zero-variance features
- Check pairwise correlation; drop one of highly correlated pairs (r > 0.9)
- Use domain knowledge first, then data-driven methods (mutual information, permutation importance)
- Watch for data leakage: never use target-derived features in training

### 6. Model Selection Workflow

- Start simple: mean/median baseline, then linear/logistic regression
- Add complexity only if simple models underperform and you understand why
- Use train/validation/test split or cross-validation — never evaluate on training data
- Compare models on the same metric, chosen before fitting (RMSE, AUC, F1, etc.)
- Document why a model was chosen, not just which one won

### 7. Visualization Best Practices

- Every plot needs a title, axis labels, and units
- Choose the chart type that matches the data relationship (see `references/visualization-guidelines.md`)
- Use colorblind-friendly palettes; avoid red/green as sole differentiator
- Annotate key data points; remove chart junk (gridlines, borders, legends when unnecessary)
- Export at publication resolution: `dpi=300`, vector formats (SVG/PDF) when possible

### 8. Communicating Results

- Lead with the finding, not the method
- Use plain language: "customers who used feature X retained 23% more" not "the coefficient was 0.23"
- Show uncertainty: ranges, intervals, caveats
- Include a "so what" — what action does this analysis support?
- Reproducibility: pin library versions, save random seeds, script everything

## Preferred Libraries

| Task | First choice | Alternative |
|------|-------------|-------------|
| DataFrames | polars | pandas |
| Visualization | matplotlib + seaborn | plotly |
| Statistics | scipy.stats | statsmodels |
| ML | scikit-learn | xgboost/lightgbm |
| Notebooks | jupyter lab | marimo |

## Examples

**User:** "I have a CSV of customer transactions. Help me understand churn patterns."
**Agent:** Loads the CSV, prints shape/dtypes/nulls, creates tidy time-series per customer,
runs EDA with churn-rate distributions and cohort analysis, tests whether usage frequency
differs between churned/retained groups (t-test with effect size), and produces annotated
visualizations summarizing the key drivers.

**User:** "Which features matter most for predicting house prices?"
**Agent:** Loads data, cleans nulls and encodes categoricals, checks correlations and
distributions, fits a baseline linear regression, then uses permutation importance to
rank features. Reports top 10 features with confidence intervals and a bar chart.
Notes any multicollinearity concerns.

**User:** "Is the difference in conversion rates between groups A and B significant?"
**Agent:** Computes conversion rates and sample sizes for both groups, runs a chi-square
test (checking expected cell counts >= 5), reports the p-value and 95% confidence interval
for the difference, and visualizes the rates with error bars. States whether the result
is practically meaningful given the effect size.
