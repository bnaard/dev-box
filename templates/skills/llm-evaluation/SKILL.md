---
name: llm-evaluation
description: LLM output evaluation including automated metrics, LLM-as-judge, A/B testing, and regression testing. Use when evaluating model outputs, building eval pipelines, or comparing prompt versions.
---

# LLM Evaluation

## When to Use

- Measuring the quality of LLM outputs systematically
- Comparing prompt versions or model versions
- Building an automated evaluation pipeline
- Setting up regression testing for prompt changes
- Detecting bias or harmful outputs
- Evaluating RAG pipeline quality (retrieval + generation)

## Instructions

### 1. Build an Evaluation Dataset

Everything starts with a good eval set. Without one, you are guessing.

- Collect 50-100 representative input/expected-output pairs from real usage
- Include edge cases and adversarial inputs (10-20% of the set)
- Label with metadata: difficulty, category, expected behavior
- Split into dev set (for iteration) and held-out test set (for final measurement)
- Store as structured data (JSONL or CSV): `{"input": "...", "expected": "...", "category": "..."}`

Sources for eval data: production logs, user feedback, manually curated examples, synthetic generation with manual review.

### 2. Automated Metrics

Use when output has a clear reference answer.

**Text overlap metrics:**
- **BLEU** — N-gram precision between generated and reference text. Good for translation. Poor for open-ended generation.
- **ROUGE** — Recall-oriented (ROUGE-L for longest common subsequence). Good for summarization.
- **Exact match** — Binary pass/fail. Good for factual extraction, classification.

**Semantic similarity metrics:**
- **BERTScore** — Cosine similarity of contextual embeddings. Better than BLEU/ROUGE for paraphrased correct answers.
- **Embedding similarity** — Compare output embedding to reference embedding. Quick and cheap.

**Task-specific metrics:**
- **Code:** Pass@k (does generated code pass test cases?), syntax validity.
- **Classification:** Precision, recall, F1 per category.
- **Structured output:** Schema compliance rate, field accuracy.

Automated metrics are fast and cheap but miss nuance. Always complement with human or LLM-as-judge evaluation.

### 3. LLM-as-Judge

Use a strong LLM to evaluate outputs from a weaker LLM or from different prompts.

Setup:
- Define a rubric with 3-5 criteria (accuracy, helpfulness, safety, format compliance)
- Score each criterion on a 1-5 scale with clear definitions for each level
- Use temperature 0.0 for reproducibility
- Include the rubric, the input, and the output in the judge prompt

Judge prompt template:
```
You are evaluating the quality of an AI response.

Criteria:
- Accuracy (1-5): Is the information factually correct?
- Completeness (1-5): Does it address all parts of the question?
- Clarity (1-5): Is the response well-organized and easy to understand?

Input: {input}
Response: {response}

Score each criterion and provide a brief justification.
Return as JSON: {"accuracy": N, "completeness": N, "clarity": N, "justification": "..."}
```

Mitigations for judge bias:
- Swap the order of candidates in pairwise comparison (position bias)
- Run each evaluation twice and check consistency
- Use multiple judge models and take the average
- Calibrate the judge against human ratings on a small set

### 4. Human Evaluation

The gold standard but expensive. Use strategically.

- Reserve for high-stakes decisions (model choice, major prompt changes)
- Use 3+ raters per example to measure inter-rater agreement (Cohen's kappa)
- Provide clear rubrics — vague instructions lead to noisy ratings
- Blind the raters to which version they are evaluating
- Sample 50-100 examples; more is rarely needed for directional decisions

### 5. A/B Testing Prompts

Compare prompt versions on the same eval set:

1. Run version A and version B on the full eval set
2. Score both using automated metrics + LLM-as-judge
3. Compare distributions (mean, median, p25, p75)
4. Run statistical significance test (paired t-test or Wilcoxon signed-rank)
5. Check for regressions: even if B is better overall, does it fail on cases A handled well?

Track over time: build a leaderboard of prompt versions with scores.

### 6. Regression Testing

Catch prompt or model regressions before they reach production:

- Maintain a suite of critical test cases (golden set) with expected outputs
- Run the suite on every prompt or model change
- Set thresholds: "accuracy must be >95%, format compliance must be 100%"
- Integrate into CI: fail the build if thresholds are breached
- Alert on metric degradation trends even within thresholds

### 7. Bias and Safety Detection

- Test with demographically diverse inputs (names, locations, contexts)
- Check for stereotyping, toxicity, and refusal of valid requests
- Use red-teaming prompts to probe for harmful outputs
- Measure refusal rate — too high means the model is over-cautious
- Tools: Perspective API for toxicity, custom classifiers for domain-specific harms

### 8. RAG-Specific Evaluation (RAGAS)

For RAG pipelines, use the RAGAS framework:

- **Context precision:** What fraction of retrieved chunks are relevant?
- **Context recall:** What fraction of needed information was retrieved?
- **Faithfulness:** Is the answer grounded in the retrieved context (not hallucinated)?
- **Answer relevance:** Does the answer address the original question?

Build a golden dataset: 50+ question/ideal-context/reference-answer triples. Run RAGAS metrics to identify which pipeline stage (retrieval or generation) needs improvement.

## Examples

### Setting Up an Eval Pipeline
User wants to systematically evaluate their customer support chatbot. Help build a JSONL eval set from production conversation logs. Define rubric: accuracy, helpfulness, tone, escalation appropriateness. Implement LLM-as-judge scoring with GPT-4. Set up a CI job that runs evals on every prompt change and reports a scorecard. Flag any regression beyond 5% on any metric.

### Comparing Prompt Versions
User has two versions of a summarization prompt and wants to know which is better. Run both on 50 diverse documents. Score with ROUGE-L (automated) and LLM-as-judge (quality rubric). Build a comparison table showing per-document scores. Run a paired significance test. Identify documents where version B regressed and analyze why. Recommend the winner with caveats.

### Detecting Model Bias
User is deploying a resume screening prompt and wants to check for bias. Generate test resumes with varied names (gender, ethnicity signals) but identical qualifications. Run the prompt on all variants. Compare score distributions across demographic groups. Flag statistically significant differences. Recommend debiasing strategies: remove names before scoring, add explicit fairness instructions, or use a separate bias-check pass.
