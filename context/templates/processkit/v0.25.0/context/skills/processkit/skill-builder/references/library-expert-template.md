# Library Expert Skill Template

Use this template when creating StackOverflow-style library expert skills:
offline-first, version-pinned, and recipe-heavy.

## Frontmatter

```yaml
---
name: <library-name>
description: >
  Solve common <library-name> tasks with version-pinned, copy-pasteable
  recipes and current-docs escalation guidance.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v2
    id: SKILL-<library-name>
    version: "1.0.0"
    created: <YYYY-MM-DDTHH:MM:SSZ>
    category: <engineering|data-ai|frontend|...>
    layer: null
  library:
    name: <library-name>
    version: "<major.minor-or-range>"
    homepage: <canonical-docs-url>
    docs: <specific-version-docs-url>
    source_date: <YYYY-MM-DD>
---
```

## Body Shape

1. Intro: one paragraph describing the library and when to use it.
2. Choosing: a decision table if there are close alternatives.
3. Recipes: 5-15 problem/solution blocks.
4. RAG Escalation: when to use current docs or retrieval.
5. Gotchas: agent-specific failure modes.
6. Full reference: official docs links and version notes.

## Recipe Format

Each recipe should be short, searchable, and copy-pasteable:

````markdown
### Group By And Aggregate

**Problem:** Summarize rows by one or more keys.

**Solution:**

```python
# minimal working example pinned to the documented library version
```

**Watch out:** Mention one common pitfall only when it changes the solution.
````

## RAG Escalation

Use baked-in recipes first. Escalate to `rag-engineering` or live docs only
when:

- the user asks about a version newer than `metadata.library.version`
- the task is outside the recipe set
- an API has likely changed
- the answer depends on provider/cloud/runtime-specific integration docs

When escalating, cite the exact docs version used and do not silently mix code
from multiple major versions.

## Builder Checklist

- Verify the current stable library version from official docs.
- Identify the 10 most common tasks from docs, issues, and Q&A patterns.
- Prefer official examples over blog snippets.
- Keep examples minimal but executable.
- Include imports and setup needed to run each recipe.
- Add one "watch out" note only when it prevents a real common failure.
- Run or type-check examples when the dependency is available locally.
- Add trigger tests for "how do I do X in <library>" and negative cases
  that should use a broader language/framework skill instead.
- Document the last source review date in `metadata.library.source_date`.
- Mark any unverified recipe explicitly; do not present guessed code as
  copy-pasteable.

## Reviewer Checklist

Use this checklist from `skill-reviewer` when auditing a library expert skill:

- `metadata.library` includes name, version, homepage, docs, and source date.
- The skill names the version range it covers in the Intro or Overview.
- Recipes use APIs from one major version only.
- Every recipe includes imports/setup needed to run the snippet.
- The RAG Escalation section says when to use current docs.
- Gotchas include stale API risk and project-version mismatch risk.
