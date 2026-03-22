---
id: proc-code-review
version: "1.0"
title: Code Review
roles: [developer, reviewer]
---
# Code Review

## Steps

1. Author opens pull request with description
2. Reviewer checks correctness, readability, test coverage
3. Reviewer leaves comments or approves
4. Author addresses feedback
5. Merge when approved

## Checklist

- [ ] Code compiles and passes CI
- [ ] Tests cover new/changed behavior
- [ ] No unrelated changes bundled
- [ ] Documentation updated if needed

## Definition of Done

- At least one approval
- All comments resolved
- CI green
