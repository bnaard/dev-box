---
name: ml-pipeline
description: ML pipeline design including data versioning, experiment tracking, model deployment, and drift monitoring. Use when building ML pipelines, setting up MLOps, or reviewing ML infrastructure.
allowed-tools: Bash Read Write
---

# ML Pipeline Design

## When to Use

- Building a new ML pipeline from data to deployment
- Setting up experiment tracking and model versioning
- Choosing deployment patterns (shadow, canary, A/B)
- Implementing monitoring for data drift and model degradation
- Designing CI/CD for ML projects
- Reviewing existing ML infrastructure for gaps

## Instructions

### 1. Pipeline Architecture Overview

A production ML pipeline has these stages (see `references/pipeline-stages.md` for details):

```
Data Collection -> Data Versioning -> Feature Engineering -> Training
    -> Evaluation -> Model Registry -> Deployment -> Monitoring
```

Key principle: every stage must be reproducible. Given the same data version, feature config, and hyperparameters, you must get the same model. Without reproducibility, debugging is impossible.

### 2. Data Versioning

Track datasets like code. Every training run must reference an exact data version.

- **DVC (Data Version Control):** Git-like versioning for large files. Stores metadata in Git, data in remote storage (S3, GCS). Use `dvc push/pull` like `git push/pull`.
- **Delta Lake / Lakehouse:** Versioned data tables with time travel. Good for structured data at scale.
- **Simple approach:** Immutable datasets in object storage with naming convention: `s3://data/training/v42/`. Never overwrite, always create new versions.

Minimum requirements:
- Every training run logs the exact dataset version used
- Data transformations are versioned code, not manual steps
- Raw data is immutable; transformations produce new versioned datasets

### 3. Feature Engineering and Feature Stores

Features should be computed once, reused across training and serving.

- **Feature store (Feast, Tecton, Hopsworks):** Centralized feature repository with consistent online (low-latency serving) and offline (batch training) access.
- **When to use a feature store:** Multiple models share features, training-serving skew is a problem, features require complex transformations.
- **When to skip:** Single model, simple features computed inline, small team.

Feature engineering best practices:
- Version feature definitions alongside model code
- Test feature pipelines with unit tests (input data -> expected features)
- Monitor feature distributions in production (detect drift early)

### 4. Experiment Tracking

Record every training run with its configuration, metrics, and artifacts.

Tools:
- **MLflow:** Open-source, self-hosted or managed. Tracks params, metrics, artifacts. Model registry built-in.
- **Weights & Biases (W&B):** Cloud-hosted, rich visualization, experiment comparison. Best UX for individual practitioners.
- **Neptune:** Cloud-hosted, strong collaboration features. Good for teams.
- **Simple approach:** Structured logs + Git tags for small projects.

What to log per experiment:
- Git commit hash of the code
- Dataset version
- All hyperparameters (learning rate, batch size, architecture)
- Training and validation metrics per epoch
- Final evaluation metrics on held-out test set
- Model artifact (serialized model file)
- Environment (Python version, package versions)

### 5. Model Registry

A central catalog of trained models with metadata, versioning, and lifecycle management.

- Register models after they pass evaluation thresholds
- Tag lifecycle stages: `staging`, `production`, `archived`
- Store model card: description, training data, metrics, limitations, bias assessment
- One model in `production` per model name at a time
- Promotion requires approval (manual or automated gate)

Tools: MLflow Model Registry, Vertex AI Model Registry, SageMaker Model Registry, or a simple database with S3 artifact references.

### 6. Deployment Patterns

| Pattern | Description | Use When |
|---------|-------------|----------|
| **Shadow** | New model runs alongside old, results compared but not served | Validating a new model with real traffic, no user impact |
| **Canary** | New model serves 5-10% of traffic, gradually increasing | Catching issues early with limited blast radius |
| **A/B test** | Two models serve different user segments, measure business metrics | Comparing models on real user outcomes |
| **Blue-green** | Two identical environments, switch traffic at once | Need instant rollback capability |
| **Feature flag** | Model version controlled by feature flag system | Gradual rollout with easy kill switch |

Default recommendation: start with shadow deployment to validate, then canary rollout with automated rollback on metric degradation.

Serving infrastructure:
- **Batch inference:** Scheduled job (Airflow, cron) that processes data in bulk. Simplest. Use when latency tolerance is hours.
- **Online inference (API):** Model behind a REST/gRPC endpoint. Use for real-time predictions. Frameworks: TorchServe, TF Serving, Triton, or a simple FastAPI wrapper.
- **Embedded:** Model shipped with the application (edge, mobile). Use ONNX or TFLite for portability.

### 7. Monitoring and Drift Detection

Production models degrade. Monitor proactively.

**Data drift:** Input feature distributions shift from training distribution.
- Monitor with statistical tests: KS test, PSI (Population Stability Index), Jensen-Shannon divergence
- Alert when drift exceeds threshold on key features
- Dashboard showing feature distributions over time

**Model drift (concept drift):** Relationship between features and target changes.
- Monitor prediction distribution: if output distribution shifts, investigate
- Track business metrics (conversion rate, error rate) alongside model metrics
- Compare recent predictions against delayed ground truth labels

**Operational monitoring:**
- Prediction latency (p50, p99)
- Error rate and failure modes
- Input validation failures (malformed requests, out-of-range features)
- Resource utilization (GPU, memory)

Retraining triggers: scheduled (weekly/monthly), drift-based (alert threshold), performance-based (metric degradation below threshold).

### 8. CI/CD for ML

ML CI/CD extends software CI/CD with data and model validation.

**CI (on every code change):**
- Unit tests for feature engineering and data transformations
- Model training on a small dataset subset (smoke test)
- Linting and code quality checks

**CD (on model promotion):**
- Full evaluation on held-out test set
- Comparison against current production model
- Bias and fairness checks
- Shadow deployment for N hours
- Automated canary rollout with rollback triggers

Tools: GitHub Actions + DVC + MLflow, or managed platforms (Vertex AI Pipelines, SageMaker Pipelines, Kubeflow).

## Examples

### Setting Up a New ML Project
User is starting a classification project from scratch. Set up the project structure: `data/`, `features/`, `models/`, `notebooks/`, `tests/`. Initialize DVC for data versioning. Set up MLflow for experiment tracking (local server for start). Create a training script that logs all params and metrics. Build a simple CI pipeline: lint, unit tests, train on sample data. Write a Makefile with targets: `train`, `evaluate`, `serve`.

### Debugging Model Degradation in Production
User reports their recommendation model's click-through rate dropped 15% this week. Check data drift dashboard: did input feature distributions change? Check for upstream data pipeline failures. Compare recent feature distributions to training data using PSI. If drift detected, identify which features drifted and trace to root cause (new user segment, data bug, seasonal change). If no drift, check for concept drift by comparing predictions on recent labeled data. Recommend retraining on recent data if drift is confirmed.

### Designing a Deployment Pipeline
User has a trained model and wants to deploy to production safely. Set up model registry with staging and production stages. Implement shadow deployment first: route real traffic to both old and new model, compare outputs for 48 hours. If shadow results are acceptable, move to canary: serve 5% of traffic with the new model, monitor business metrics. Automated rollback if error rate increases >2%. Gradual ramp to 100% over one week. Set up drift monitoring from day one.
