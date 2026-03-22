---
name: feature-engineering
description: Feature engineering for ML including encoding, imputation, scaling, selection, and time-series features. Use when preparing data for ML models, selecting features, or engineering new features from raw data.
allowed-tools: Bash(python:*) Read Write
---

# Feature Engineering

## When to Use

When the user is preparing data for machine learning models, encoding categorical
variables, handling missing data, creating new features from raw data, or asks
"how should I engineer features for this model?". Also applies when selecting
features or building time-series features.

## Instructions

### 1. Encoding Categorical Variables

- **One-hot encoding**: Use for nominal categories with < 15 unique values. Creates binary columns.
  Use `pd.get_dummies(drop_first=True)` or `OneHotEncoder(drop="first")` to avoid multicollinearity.
- **Ordinal encoding**: Use when categories have natural order (low/medium/high, grade levels).
  Map explicitly: `{"low": 0, "medium": 1, "high": 2}` — never rely on alphabetical order.
- **Target encoding**: Use for high-cardinality categoricals (city, zip code). Replace category
  with mean target value. Always use cross-validated encoding to prevent leakage.
- **Frequency encoding**: Replace category with its count or proportion. Useful when frequency
  itself is predictive (popular products, common zip codes).
- Never use label encoding for nominal categories — it implies false ordinal relationships.

### 2. Handling Missing Data

- Understand the missingness mechanism first: MCAR, MAR, or MNAR
- **Numerical imputation**: Median (robust to outliers) or mean (preserves distribution mean).
  Add a binary `_is_missing` indicator column when missingness may be informative.
- **Categorical imputation**: Mode for low-cardinality, or treat missing as its own category `"unknown"`.
- **Advanced**: KNN imputation or iterative imputation (MICE) when features are correlated.
- **Drop**: Only when < 5% of rows are affected and missingness is MCAR.
- Always impute using training set statistics only — fit on train, transform on test.

### 3. Feature Scaling

- **StandardScaler** (z-score): Use for algorithms assuming normal distribution (linear regression,
  SVM, PCA). Centers to mean=0, std=1.
- **MinMaxScaler** (0-1): Use for neural networks or when bounded range is needed.
  Sensitive to outliers.
- **RobustScaler** (IQR-based): Use when data has outliers. Centers on median, scales by IQR.
- Tree-based models (Random Forest, XGBoost) do not need scaling.
- Always fit the scaler on training data only; apply the same transformation to test data.

### 4. Feature Selection

- **Filter methods**: Correlation with target (Pearson, Spearman), mutual information, chi-square.
  Fast but ignores feature interactions.
- **Wrapper methods**: Recursive feature elimination (RFE), forward/backward selection.
  Expensive but considers feature combinations.
- **Embedded methods**: L1 regularization (Lasso), tree-based feature importance, permutation
  importance. Best balance of speed and accuracy.
- Remove zero-variance features first (constant columns).
- Drop one of highly correlated pairs (r > 0.9) — they add noise without information.
- Use domain knowledge as the first filter before data-driven methods.

### 5. Time-Series Features

- **Lag features**: Previous values as predictors: `value_lag_1`, `value_lag_7`, `value_lag_30`.
  Choose lags based on domain (daily seasonality = lag 7 for weekly data).
- **Rolling statistics**: Rolling mean, std, min, max over windows: `value_rolling_mean_7d`.
  Window size should match the pattern you want to capture.
- **Seasonal features**: Extract from timestamps: `hour_of_day`, `day_of_week`, `month`,
  `is_weekend`, `is_holiday`. Encode cyclical features with sin/cos: `sin(2*pi*hour/24)`.
- **Difference features**: Period-over-period change: `value - value_lag_1`, percent change.
- **Expanding statistics**: Cumulative mean, sum, count from the start of the series.
- Always respect temporal ordering — never use future values as features (data leakage).

### 6. Text Features

- **TF-IDF**: Baseline text representation. Use `max_features` to limit dimensionality.
  Works well for classification with traditional ML models.
- **Count vectorizer**: Simpler than TF-IDF; use when term frequency alone is predictive.
- **Embeddings**: Pre-trained embeddings (sentence-transformers) for semantic similarity.
  Superior for nuanced meaning but more expensive.
- **Extracted features**: Text length, word count, punctuation count, sentiment score,
  readability score. Often more interpretable than bag-of-words.
- Combine TF-IDF with extracted features for a strong baseline before using embeddings.

### 7. Interaction and Derived Features

- Create interaction terms for known relationships: `price * quantity = revenue`
- Ratio features: `clicks / impressions = CTR`, `revenue / users = ARPU`
- Polynomial features: use sparingly (degree 2 max), only for known non-linear relationships
- Binning: Convert continuous to categorical when the relationship is step-wise
  (age groups, income brackets). Use domain-driven bins, not arbitrary quantiles.
- Log transform: Apply to right-skewed features (income, prices, counts) to reduce skew

### 8. Avoiding Data Leakage

- Never compute features using information from the test set
- Fit all transformers (scaler, imputer, encoder) on training data only
- For time-series: features must only use data available at prediction time
- Target encoding must use cross-validation folds, not the full training set
- Pipeline your preprocessing: use `sklearn.pipeline.Pipeline` to ensure proper ordering

## Examples

**User:** "I have a dataset with user_id, city, purchase_amount, and timestamp. How should I engineer features?"
**Agent:** Target-encodes `city` (high cardinality) with cross-validated means,
extracts `day_of_week`, `hour`, `is_weekend` from timestamp, creates lag features
for previous purchase amounts per user, adds rolling 7-day and 30-day purchase
means, and creates a `days_since_last_purchase` feature. Wraps everything in a
sklearn Pipeline to prevent leakage.

**User:** "My model performance is poor. I have 200 features."
**Agent:** Starts by removing zero-variance and near-constant columns, then drops
one of each highly correlated pair (r > 0.9). Runs permutation importance on the
baseline model to rank features, selects the top 30-50, and retrains. Compares
performance with the reduced set. Checks for data leakage in remaining features.

**User:** "How do I handle the 40% missing values in the income column?"
**Agent:** Investigates whether missingness correlates with other features (MAR check).
Creates a binary `income_is_missing` indicator feature. Imputes with median from
training set (robust to the skewed income distribution). Tests KNN imputation as
an alternative and compares model performance with both approaches.
