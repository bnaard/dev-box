---
name: ai-fundamentals
description: Core ML/AI concepts including model types, training pipelines, evaluation metrics, and neural network architectures. Use when explaining AI concepts, choosing model approaches, designing ML solutions, or reviewing AI-related code.
---

# AI Fundamentals

## When to Use

- Explaining or comparing ML/AI concepts and terminology
- Choosing a model architecture or learning approach for a problem
- Designing or reviewing a training pipeline
- Selecting and interpreting evaluation metrics
- Discussing neural network design decisions
- Reviewing AI-related code for correctness and best practices
- Debugging model performance issues (overfitting, data leakage, class imbalance)

## Instructions

### 1. ML Taxonomy

Know the four major learning paradigms and when each applies:

- **Supervised learning** — labeled input-output pairs; use for classification and regression
- **Unsupervised learning** — no labels; use for clustering, dimensionality reduction, anomaly detection
- **Reinforcement learning** — agent learns via reward signal; use for sequential decision-making, game play, robotics
- **Self-supervised learning** — generates labels from data itself (e.g., masked language modeling, contrastive learning); powers modern foundation models

When advising on approach, start from the data available and the problem type, not from the model.

### 2. Model Types

Match model complexity to the problem:

- **Linear models** (linear/logistic regression, SVM) — interpretable, fast, strong baselines
- **Tree-based models** (decision trees, random forests, gradient boosting) — handle tabular data well, robust to feature scaling; XGBoost/LightGBM are top performers on structured data
- **Neural networks** — excel on unstructured data (images, text, audio); require more data and compute
- **Probabilistic models** (naive Bayes, Gaussian processes, Bayesian networks) — useful when uncertainty quantification matters

Rule of thumb: start simple, add complexity only when justified by data size and performance needs.

### 3. Training Pipeline

A correct pipeline follows this order:

1. **Data preparation** — cleaning, handling missing values, encoding categoricals, normalization/standardization
2. **Train/validation/test split** — split before any preprocessing that uses statistics from the data; typical ratio 70/15/15 or use k-fold cross-validation
3. **Feature engineering** — domain-driven transformations, feature selection
4. **Model training** — fit on training set only
5. **Hyperparameter tuning** — grid search, random search, or Bayesian optimization against validation set
6. **Regularization** — L1 (sparsity), L2 (weight decay), dropout, early stopping to prevent overfitting
7. **Final evaluation** — report metrics on held-out test set; never tune against test set

See `references/ml-concepts.md` for details on cross-validation and gradient descent.

### 4. Evaluation Metrics

Choose metrics that match the business problem:

| Task | Metrics | When to prefer |
|------|---------|----------------|
| Binary classification | Accuracy, Precision, Recall, F1, AUC-ROC | F1 for imbalanced classes; AUC-ROC for ranking |
| Multi-class classification | Macro/micro F1, confusion matrix | Macro F1 when all classes matter equally |
| Regression | RMSE, MAE, R-squared | MAE when outliers should not dominate; RMSE to penalize large errors |
| Ranking/retrieval | Precision@k, NDCG, MAP | NDCG when position in ranking matters |
| Generation (NLP) | BLEU, ROUGE, perplexity, human eval | Human eval is gold standard for open-ended generation |

Always report confidence intervals or variance across folds, not single-point estimates.

### 5. Neural Network Architectures

Core architectures and their domains:

- **MLP (Multi-Layer Perceptron)** — fully connected layers; general-purpose but no structural bias
- **CNN (Convolutional Neural Network)** — spatial hierarchy via convolutions; images, time series, any grid-structured data
- **RNN/LSTM/GRU** — sequential processing with memory; time series, legacy NLP (largely replaced by Transformers)
- **Transformer** — self-attention over all positions; dominant in NLP, vision (ViT), and multimodal tasks
- **GAN (Generative Adversarial Network)** — generator vs. discriminator; image synthesis, data augmentation
- **VAE (Variational Autoencoder)** — latent variable model with probabilistic encoder/decoder; generation with structured latent space
- **Diffusion models** — iterative denoising process; state-of-the-art image and video generation

### 6. Modern LLM Concepts

Key ideas behind large language models:

- **Attention mechanism** — allows the model to weigh relevance of all input tokens; scaled dot-product attention is the core operation
- **Tokenization** — BPE, WordPiece, or SentencePiece break text into subword units; vocabulary size affects model capacity and efficiency
- **Pre-training + fine-tuning** — pre-train on large corpus (self-supervised), then fine-tune on task-specific data
- **RLHF (Reinforcement Learning from Human Feedback)** — aligns model outputs with human preferences via reward model + PPO/DPO
- **Prompting strategies** — zero-shot, few-shot, chain-of-thought, retrieval-augmented generation (RAG)
- **Scaling laws** — performance improves predictably with more data, parameters, and compute (Chinchilla-optimal ratios)

### 7. Common Pitfalls

Watch for these failure modes in any ML project:

- **Data leakage** — test information bleeds into training (e.g., normalizing before splitting, using future data)
- **Overfitting** — model memorizes training data; symptoms: large train-test performance gap; fix with regularization, more data, simpler model
- **Underfitting** — model too simple to capture patterns; increase capacity or improve features
- **Class imbalance** — minority class is underrepresented; use stratified splits, class weights, SMOTE, or focal loss
- **Distribution shift** — training and production data differ; monitor with drift detection, retrain periodically
- **Metric mismatch** — optimizing a metric that does not reflect business value
- **Confirmation bias in evaluation** — cherry-picking results or evaluating on non-representative data

See `references/math-foundations.md` for the mathematical building blocks.

## Examples

### Scenario 1: Choosing an approach for tabular customer churn prediction

With 50K labeled rows of structured customer data, start with a gradient-boosted tree model (XGBoost or LightGBM). Use stratified k-fold cross-validation since churn is typically imbalanced (5-10% positive). Report F1 and AUC-ROC. Baseline with logistic regression first. Only consider neural approaches if tree models plateau and you have sufficient data.

### Scenario 2: Reviewing a training pipeline for data leakage

A teammate normalizes the entire dataset before splitting into train/test. This leaks test set statistics into training. Fix: split first, fit the scaler on training data only, then transform both train and test with the training scaler. Same principle applies to any feature engineering step that computes statistics (e.g., target encoding, PCA).

### Scenario 3: Selecting architecture for a document Q&A system

Use a Transformer-based approach: retrieve relevant passages with a vector store (RAG pattern), then feed them as context to an LLM. Fine-tune the retriever on domain-specific queries if generic embeddings underperform. Evaluate with exact match, F1 over answer tokens, and human judgment on a held-out set. Monitor for hallucination by checking answer grounding against retrieved passages.
