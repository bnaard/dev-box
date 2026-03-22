---
name: prompt-engineering
description: Prompt design patterns for LLMs including few-shot, chain-of-thought, structured output, and injection defense. Use when crafting prompts, optimizing LLM outputs, or building prompt-based features.
---

# Prompt Engineering

## When to Use

- Crafting or refining prompts for LLM-based features
- Improving output quality, consistency, or reliability
- Designing system prompts for AI agents or chatbots
- Implementing structured output (JSON, specific formats)
- Defending against prompt injection attacks
- Building prompt templates for reusable workflows
- Evaluating and iterating on prompt performance

## Instructions

### 1. Prompt Structure Fundamentals

A well-structured prompt has these components (in order of importance):

1. **Role/Context** — Who is the model? What domain expertise applies?
2. **Task** — What exactly should it do? Be specific and unambiguous.
3. **Constraints** — Format, length, tone, what to avoid.
4. **Examples** — Input/output pairs demonstrating desired behavior.
5. **Input** — The actual data to process.

Principles:
- Be explicit. LLMs do not read minds — state what you want and what you do not want.
- Put the most important instructions first and last (primacy and recency effects).
- Use delimiters to separate sections: `###`, `---`, XML tags, triple backticks.
- Shorter is not always better — a well-structured 500-word prompt beats an ambiguous 50-word one.

### 2. Core Techniques

See `references/techniques-catalog.md` for detailed templates and examples.

**Zero-shot:** Direct instruction with no examples. Works for simple, well-defined tasks.

**Few-shot:** Provide 2-5 input/output examples before the actual input. The model learns the pattern from examples. Choose diverse, representative examples. Order matters — put the most similar example last.

**Chain-of-Thought (CoT):** Add "Let's think step by step" or provide reasoning examples. Dramatically improves math, logic, and multi-step tasks. Can be combined with few-shot (show reasoning in examples).

**Self-consistency:** Generate multiple responses with temperature > 0, then take the majority answer. Best for factual or reasoning tasks where there is one correct answer.

**Structured output:** Request JSON, XML, or specific formats. Use JSON mode when available. Provide the exact schema in the prompt. Validate output programmatically.

### 3. System Prompt Design

System prompts set persistent behavior for the entire conversation:

- Define the persona, expertise, and communication style
- Set hard constraints (what the model must never do)
- Establish output format expectations
- Include domain-specific knowledge or rules

Best practices:
- Keep system prompts focused — one clear role, not five
- Use positive instructions ("always do X") over negative ("never do Y") where possible
- Test with adversarial inputs to ensure constraints hold
- Version your system prompts and track changes like code

### 4. Temperature and Sampling Parameters

- **Temperature (0.0 - 2.0):** Controls randomness. 0.0 = deterministic, 1.0 = default creative, >1.0 = very random.
  - Use 0.0-0.3 for factual tasks, code generation, structured output
  - Use 0.5-0.8 for creative writing, brainstorming
  - Use 0.0 for reproducible evaluations
- **Top-p (0.0 - 1.0):** Nucleus sampling. 0.9 means consider tokens comprising top 90% probability. Alternative to temperature — do not adjust both simultaneously.
- **Max tokens:** Set to expected output length + buffer. Too low truncates output; too high wastes quota.
- **Stop sequences:** Define strings that halt generation. Useful for structured extraction.

### 5. Prompt Injection Defense

Prompt injection is when user input manipulates the model's behavior by overriding instructions.

Defense layers:
1. **Input sanitization:** Strip or escape known injection patterns. Detect `ignore previous instructions` type phrases.
2. **Delimited input:** Wrap user input in clear delimiters and instruct the model to treat the delimited content as data only, never as instructions.
3. **Output validation:** Verify output conforms to expected format. Reject unexpected formats.
4. **Privilege separation:** Use separate LLM calls for different trust levels. Do not mix system logic and user input in one prompt.
5. **Canary tokens:** Include a secret token in the system prompt. If it appears in output, injection may have occurred.

No defense is perfect. Layer multiple approaches and assume breach.

### 6. Prompt Templates and Iteration

Build reusable templates with variable slots:

```
You are a {role} specializing in {domain}.

Analyze the following {input_type}:
---
{input}
---

Provide your analysis in the following format:
- Summary: (1-2 sentences)
- Key findings: (bullet points)
- Recommendations: (numbered list)
```

Iteration process:
1. Start with a simple prompt that captures the core task
2. Test on 10-20 diverse inputs
3. Identify failure modes (wrong format, missing info, hallucination)
4. Add constraints or examples to address each failure mode
5. Retest — ensure fixes do not break previously working cases
6. Document the prompt version and test results

### 7. Evaluation

Measure prompt quality systematically:
- Build an eval set of 20-50 input/expected-output pairs
- Score each output (binary pass/fail, or rubric-based 1-5)
- Track metrics across prompt versions: accuracy, format compliance, latency
- Use LLM-as-judge for subjective quality (see llm-evaluation skill)
- Automate eval runs in CI when prompt changes are deployed

## Examples

### Designing a Classification Prompt
User needs to classify support tickets into categories. Design a few-shot prompt with 3-5 example tickets per category. Include edge cases. Use temperature 0.0 for consistency. Request JSON output: `{"category": "...", "confidence": "high|medium|low"}`. Validate output schema programmatically. Measure accuracy against labeled test set.

### Building a Code Review Agent
User wants an LLM-powered code review assistant. Design a system prompt defining the reviewer persona (senior engineer, specific language expertise). Include review criteria: correctness, performance, readability, security. Use structured output for findings. Add injection defense for code that might contain adversarial comments. Test with intentionally bad code to verify the agent catches issues.

### Optimizing an Underperforming Prompt
User reports their summarization prompt produces inconsistent output. Diagnose: test on 20 inputs, categorize failures (too long, misses key points, wrong tone). Add length constraints, provide few-shot examples of ideal summaries, add chain-of-thought for complex documents. A/B test the old vs. new prompt on the eval set. Track improvement in format compliance and content accuracy.
