---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260723_0850-ContentSync-processkit-content-sync
  created: 2026-07-23 08:50:34+00:00
  updated: '2026-07-23T08:51:24+00:00'
spec:
  source: processkit
  source_url: https://github.com/projectious-work/processkit.git
  from_version: v0.27.5
  to_version: v0.28.1
  state: applied
  generated_by: aibox apply
  generated_at: 2026-07-23 08:50:34+00:00
  summary: 2 changed upstream, 4 conflicts, 5 new, 16 removed, 0 stale-removed (11
    groups affected)
  affected_groups:
  - AGENTS
  - lib
  - schemas/_generated
  - schemas/src
  - skills/data-ai
  - skills/design
  - skills/devops
  - skills/documents
  - skills/engineering
  - skills/processkit
  - skills/product
  affected_files:
  - path: AGENTS.md
    classification: changed-locally-only
  - path: context/schemas/_generated/binding.yaml
    classification: removed-upstream
  - path: context/schemas/_generated/decisionrecord.yaml
    classification: removed-upstream
  - path: context/schemas/_generated/logentry.yaml
    classification: removed-upstream
  - path: context/schemas/_generated/workitem.yaml
    classification: removed-upstream
  - path: context/schemas/src/compositions/decisionrecord.yaml
    classification: removed-upstream
  - path: context/schemas/src/compositions/logentry.yaml
    classification: removed-upstream
  - path: context/schemas/src/fragments/entity.yaml
    classification: removed-upstream
  - path: context/schemas/src/fragments/generation.yaml.j2
    classification: removed-upstream
  - path: context/schemas/src/fragments/record.yaml
    classification: removed-upstream
  - path: context/schemas/src/fragments/relationship.yaml
    classification: removed-upstream
  - path: context/schemas/src/fragments/versioned.yaml
    classification: removed-upstream
  - path: context/schemas/src/primitives/binding.yaml
    classification: removed-upstream
  - path: context/schemas/src/primitives/workitem.yaml
    classification: removed-upstream
  - path: context/schemas/src/registry.yaml
    classification: removed-upstream
  - path: context/schemas/src/templates/schema.yaml.j2
    classification: removed-upstream
  - path: context/skills/_lib/processkit/paths.py
    classification: changed-upstream-only
  - path: context/skills/_lib/processkit/schema.py
    classification: changed-upstream-only
  - path: context/skills/_lib/processkit/schema_generation.py
    classification: removed-upstream
  - path: context/skills/data-ai/ai-fundamentals/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/ai-fundamentals/references/math-foundations.md
    classification: changed-locally-only
  - path: context/skills/data-ai/ai-fundamentals/references/ml-concepts.md
    classification: changed-locally-only
  - path: context/skills/data-ai/data-pipeline/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/data-quality/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/data-science/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/data-science/references/statistical-methods.md
    classification: changed-locally-only
  - path: context/skills/data-ai/data-science/references/tidy-data-principles.md
    classification: changed-locally-only
  - path: context/skills/data-ai/data-science/references/visualization-guidelines.md
    classification: changed-locally-only
  - path: context/skills/data-ai/embedding-vectordb/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/feature-engineering/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/llm-evaluation/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/ml-pipeline/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/ml-pipeline/references/pipeline-stages.md
    classification: changed-locally-only
  - path: context/skills/data-ai/pandas-polars/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/pandas-polars/references/api-comparison.md
    classification: changed-locally-only
  - path: context/skills/data-ai/prompt-engineering/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/prompt-engineering/references/techniques-catalog.md
    classification: changed-locally-only
  - path: context/skills/data-ai/rag-engineering/SKILL.md
    classification: changed-locally-only
  - path: context/skills/data-ai/rag-engineering/references/chunking-strategies.md
    classification: changed-locally-only
  - path: context/skills/data-ai/rag-engineering/references/evaluation.md
    classification: changed-locally-only
  - path: context/skills/data-ai/rag-engineering/references/retrieval-patterns.md
    classification: changed-locally-only
  - path: context/skills/design/excalidraw/SKILL.md
    classification: changed-locally-only
  - path: context/skills/design/excalidraw/references/json-schema.md
    classification: changed-locally-only
  - path: context/skills/design/frontend-design/SKILL.md
    classification: changed-locally-only
  - path: context/skills/design/frontend-design/references/accessibility-checklist.md
    classification: changed-locally-only
  - path: context/skills/design/logo-design/SKILL.md
    classification: changed-locally-only
  - path: context/skills/design/logo-design/references/design-principles.md
    classification: changed-locally-only
  - path: context/skills/design/mobile-app-design/SKILL.md
    classification: changed-locally-only
  - path: context/skills/design/mobile-app-design/references/platform-guidelines.md
    classification: changed-locally-only
  - path: context/skills/design/seo-optimization/SKILL.md
    classification: changed-locally-only
  - path: context/skills/design/seo-optimization/references/technical-seo-checklist.md
    classification: changed-locally-only
  - path: context/skills/devops/alerting-oncall/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/ci-cd-setup/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/container-orchestration/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/container-orchestration/references/compose-patterns.md
    classification: changed-locally-only
  - path: context/skills/devops/distributed-tracing/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/dns-networking/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/dns-networking/references/protocol-reference.md
    classification: changed-locally-only
  - path: context/skills/devops/dns-networking/references/troubleshooting-tools.md
    classification: changed-locally-only
  - path: context/skills/devops/dockerfile-review/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/incident-response/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/kubernetes-basics/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/kubernetes-basics/references/cluster-architecture.md
    classification: changed-locally-only
  - path: context/skills/devops/kubernetes-basics/references/resource-cheatsheet.md
    classification: changed-locally-only
  - path: context/skills/devops/kubernetes-basics/references/troubleshooting.md
    classification: changed-locally-only
  - path: context/skills/devops/linux-administration/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/linux-administration/references/commands-cheatsheet.md
    classification: changed-locally-only
  - path: context/skills/devops/linux-administration/references/systemd-reference.md
    classification: changed-locally-only
  - path: context/skills/devops/logging-strategy/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/logging-strategy/references/structured-logging.md
    classification: changed-locally-only
  - path: context/skills/devops/metrics-management/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/metrics-management/assets/metric-spec.yaml
    classification: changed-locally-only
  - path: context/skills/devops/metrics-monitoring/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/metrics-monitoring/references/metric-types.md
    classification: changed-locally-only
  - path: context/skills/devops/postmortem-writing/SKILL.md
    classification: changed-locally-only
  - path: context/skills/devops/release-semver/SKILL.md
    classification: conflict
  - path: context/skills/devops/release-semver/commands/pk-publish.md
    classification: changed-locally-only
  - path: context/skills/devops/release-semver/commands/pk-release.md
    classification: conflict
  - path: context/skills/devops/terraform-basics/SKILL.md
    classification: changed-locally-only
  - path: context/skills/documents/data-storytelling/SKILL.md
    classification: changed-locally-only
  - path: context/skills/documents/data-visualization/SKILL.md
    classification: changed-locally-only
  - path: context/skills/documents/data-visualization/references/chart-selection.md
    classification: changed-locally-only
  - path: context/skills/documents/docx-authoring/SKILL.md
    classification: changed-locally-only
  - path: context/skills/documents/infographics/SKILL.md
    classification: changed-locally-only
  - path: context/skills/documents/infographics/references/best-practices.md
    classification: changed-locally-only
  - path: context/skills/documents/latex-authoring/SKILL.md
    classification: changed-locally-only
  - path: context/skills/documents/latex-authoring/references/math-reference.md
    classification: changed-locally-only
  - path: context/skills/documents/latex-authoring/references/packages.md
    classification: changed-locally-only
  - path: context/skills/documents/latex-authoring/references/tikz-reference.md
    classification: changed-locally-only
  - path: context/skills/documents/pdf-workflow/SKILL.md
    classification: changed-locally-only
  - path: context/skills/documents/pptx-authoring/SKILL.md
    classification: changed-locally-only
  - path: context/skills/documents/xlsx-modeling/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/api-design/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/api-design/references/openapi-patterns.md
    classification: changed-locally-only
  - path: context/skills/engineering/api-design/references/rest-conventions.md
    classification: changed-locally-only
  - path: context/skills/engineering/auth-patterns/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/auth-patterns/references/jwt-reference.md
    classification: changed-locally-only
  - path: context/skills/engineering/auth-patterns/references/oauth-flows.md
    classification: changed-locally-only
  - path: context/skills/engineering/caching-strategies/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/changelog/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/code-generation/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/code-review/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/concurrency-patterns/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/concurrency-patterns/references/patterns-catalog.md
    classification: changed-locally-only
  - path: context/skills/engineering/database-migration/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/database-modeling/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/database-modeling/references/modeling-patterns.md
    classification: changed-locally-only
  - path: context/skills/engineering/debugging/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/dependency-audit/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/dependency-management/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/domain-driven-design/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/domain-driven-design/references/ddd-building-blocks.md
    classification: changed-locally-only
  - path: context/skills/engineering/error-handling/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/event-driven-architecture/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/event-driven-architecture/references/messaging-patterns.md
    classification: changed-locally-only
  - path: context/skills/engineering/fastapi-patterns/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/fastapi-patterns/references/endpoint-patterns.md
    classification: changed-locally-only
  - path: context/skills/engineering/flutter-development/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/flutter-development/references/widget-catalog.md
    classification: changed-locally-only
  - path: context/skills/engineering/git-branching/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/git-branching/references/strategies.md
    classification: new-upstream
  - path: context/skills/engineering/git-workflow/SKILL.md
    classification: conflict
  - path: context/skills/engineering/go-conventions/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/go-conventions/references/go-patterns.md
    classification: changed-locally-only
  - path: context/skills/engineering/graphql-patterns/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/grpc-protobuf/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/grpc-protobuf/references/proto-conventions.md
    classification: changed-locally-only
  - path: context/skills/engineering/integration-testing/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/integration-testing/references/test-fixtures.md
    classification: changed-locally-only
  - path: context/skills/engineering/java-patterns/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/load-testing/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/microservice-creation/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/nosql-patterns/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/performance-profiling/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/performance-profiling/references/profiling-tools.md
    classification: changed-locally-only
  - path: context/skills/engineering/pixijs-gamedev/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/pixijs-gamedev/references/api-cheatsheet.md
    classification: changed-locally-only
  - path: context/skills/engineering/python-best-practices/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/refactoring/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/refactoring/references/code-smells.md
    classification: changed-locally-only
  - path: context/skills/engineering/refactoring/references/gof-patterns.md
    classification: changed-locally-only
  - path: context/skills/engineering/reflex-python/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/reflex-python/references/component-reference.md
    classification: changed-locally-only
  - path: context/skills/engineering/rust-conventions/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/secret-management/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/secure-coding/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/secure-coding/references/owasp-checklist.md
    classification: changed-locally-only
  - path: context/skills/engineering/shell-scripting/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/shell-scripting/references/bash-patterns.md
    classification: changed-locally-only
  - path: context/skills/engineering/software-architecture/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/software-architecture/references/patterns.md
    classification: changed-locally-only
  - path: context/skills/engineering/software-modularization/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/sql-patterns/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/sql-patterns/references/query-patterns.md
    classification: changed-locally-only
  - path: context/skills/engineering/sql-patterns/references/schema-design.md
    classification: changed-locally-only
  - path: context/skills/engineering/sql-style-guide/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/system-design/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/system-design/references/estimation-cheatsheet.md
    classification: changed-locally-only
  - path: context/skills/engineering/tailwind/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/tailwind/references/cheatsheet.md
    classification: changed-locally-only
  - path: context/skills/engineering/tdd-workflow/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/testing-strategy/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/threat-modeling/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/typescript-patterns/SKILL.md
    classification: changed-locally-only
  - path: context/skills/engineering/webhook-integration/SKILL.md
    classification: changed-locally-only
  - path: context/skills/processkit/pk-doctor/scripts/test_pk_doctor_mcp.py
    classification: new-upstream
  - path: context/skills/processkit/project-reconciliation/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/project-reconciliation/commands/pk-reconcile.md
    classification: new-upstream
  - path: context/skills/processkit/repository-portfolio-review/SKILL.md
    classification: conflict
  - path: context/skills/processkit/supply-chain-audit/SKILL.md
    classification: changed-locally-only
  - path: context/skills/processkit/supply-chain-audit/commands/pk-supply-chain.md
    classification: changed-locally-only
  - path: context/skills/processkit/supply-chain-audit/mcp/SERVER.md
    classification: changed-locally-only
  - path: context/skills/processkit/supply-chain-audit/mcp/mcp-config.json
    classification: changed-locally-only
  - path: context/skills/processkit/supply-chain-audit/mcp/server.py
    classification: changed-locally-only
  - path: context/skills/processkit/supply-chain-audit/scripts/supply_chain_audit.py
    classification: changed-locally-only
  - path: context/skills/processkit/supply-chain-audit/scripts/test_supply_chain_audit.py
    classification: changed-locally-only
  - path: context/skills/product/board-of-advisors/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/devils-advocate/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/documentation/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/email-drafter/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/estimation-planning/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/legal-review/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/onboarding-guide/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/prd-writing/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/research-with-confidence/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/research-with-confidence/commands/pk-research.md
    classification: changed-locally-only
  - path: context/skills/product/sprint-retrospective/SKILL.md
    classification: changed-locally-only
  - path: context/skills/product/user-research/SKILL.md
    classification: changed-locally-only
  started_at: '2026-07-23T08:51:24+00:00'
  applied_at: '2026-07-23T08:51:24+00:00'
  progress_notes:
  - timestamp: '2026-07-23T08:51:24+00:00'
    actor: mcp
    note: Applied the v1.x branch-specific v0.27.5 to v0.28.1 content reconciliation
      after reviewing all four reported conflict paths. Those optional catalog skills
      are not selected into the v1 live skill set, so no live conflict merge was required;
      upstream-only library changes and selected new content were already installed
      by aibox apply.
---

# Migration MIG-20260723_0850-ContentSync-processkit-content-sync

From `v0.27.5` to `v0.28.1` (source: `https://github.com/projectious-work/processkit.git`).

2 changed upstream, 4 conflicts, 5 new, 16 removed, 0 stale-removed (11 groups affected)

## Counts

- unchanged: 541
- changed-locally-only: 161
- changed-upstream-only: 2
- conflict: 4
- new-upstream: 5
- removed-upstream: 16
- removed-upstream-stale: 0

## Changes by group

### AGENTS

**changed-locally-only**

- `AGENTS.md` → `AGENTS.md`

### lib

**changed-upstream-only**

- `context/skills/_lib/processkit/paths.py` → `context/skills/_lib/processkit/paths.py`
- `context/skills/_lib/processkit/schema.py` → `context/skills/_lib/processkit/schema.py`

**removed-upstream**

- `context/skills/_lib/processkit/schema_generation.py` → `context/skills/_lib/processkit/schema_generation.py`

### schemas/_generated

**removed-upstream**

- `context/schemas/_generated/binding.yaml` → `context/schemas/_generated/binding.yaml`
- `context/schemas/_generated/decisionrecord.yaml` → `context/schemas/_generated/decisionrecord.yaml`
- `context/schemas/_generated/workitem.yaml` → `context/schemas/_generated/workitem.yaml`
- `context/schemas/_generated/logentry.yaml` → `context/schemas/_generated/logentry.yaml`

### schemas/src

**removed-upstream**

- `context/schemas/src/fragments/record.yaml` → `context/schemas/src/fragments/record.yaml`
- `context/schemas/src/fragments/entity.yaml` → `context/schemas/src/fragments/entity.yaml`
- `context/schemas/src/fragments/relationship.yaml` → `context/schemas/src/fragments/relationship.yaml`
- `context/schemas/src/fragments/versioned.yaml` → `context/schemas/src/fragments/versioned.yaml`
- `context/schemas/src/fragments/generation.yaml.j2` → `context/schemas/src/fragments/generation.yaml.j2`
- `context/schemas/src/registry.yaml` → `context/schemas/src/registry.yaml`
- `context/schemas/src/compositions/decisionrecord.yaml` → `context/schemas/src/compositions/decisionrecord.yaml`
- `context/schemas/src/compositions/logentry.yaml` → `context/schemas/src/compositions/logentry.yaml`
- `context/schemas/src/primitives/binding.yaml` → `context/schemas/src/primitives/binding.yaml`
- `context/schemas/src/primitives/workitem.yaml` → `context/schemas/src/primitives/workitem.yaml`
- `context/schemas/src/templates/schema.yaml.j2` → `context/schemas/src/templates/schema.yaml.j2`

### skills/data-ai

**changed-locally-only**

- `context/skills/data-ai/pandas-polars/references/api-comparison.md` → `context/skills/data-ai/pandas-polars/references/api-comparison.md`
- `context/skills/data-ai/pandas-polars/SKILL.md` → `context/skills/data-ai/pandas-polars/SKILL.md`
- `context/skills/data-ai/llm-evaluation/SKILL.md` → `context/skills/data-ai/llm-evaluation/SKILL.md`
- `context/skills/data-ai/data-science/references/tidy-data-principles.md` → `context/skills/data-ai/data-science/references/tidy-data-principles.md`
- `context/skills/data-ai/data-science/references/visualization-guidelines.md` → `context/skills/data-ai/data-science/references/visualization-guidelines.md`
- `context/skills/data-ai/data-science/references/statistical-methods.md` → `context/skills/data-ai/data-science/references/statistical-methods.md`
- `context/skills/data-ai/data-science/SKILL.md` → `context/skills/data-ai/data-science/SKILL.md`
- `context/skills/data-ai/prompt-engineering/references/techniques-catalog.md` → `context/skills/data-ai/prompt-engineering/references/techniques-catalog.md`
- `context/skills/data-ai/prompt-engineering/SKILL.md` → `context/skills/data-ai/prompt-engineering/SKILL.md`
- `context/skills/data-ai/feature-engineering/SKILL.md` → `context/skills/data-ai/feature-engineering/SKILL.md`
- `context/skills/data-ai/data-quality/SKILL.md` → `context/skills/data-ai/data-quality/SKILL.md`
- `context/skills/data-ai/rag-engineering/references/chunking-strategies.md` → `context/skills/data-ai/rag-engineering/references/chunking-strategies.md`
- `context/skills/data-ai/rag-engineering/references/retrieval-patterns.md` → `context/skills/data-ai/rag-engineering/references/retrieval-patterns.md`
- `context/skills/data-ai/rag-engineering/references/evaluation.md` → `context/skills/data-ai/rag-engineering/references/evaluation.md`
- `context/skills/data-ai/rag-engineering/SKILL.md` → `context/skills/data-ai/rag-engineering/SKILL.md`
- `context/skills/data-ai/embedding-vectordb/SKILL.md` → `context/skills/data-ai/embedding-vectordb/SKILL.md`
- `context/skills/data-ai/data-pipeline/SKILL.md` → `context/skills/data-ai/data-pipeline/SKILL.md`
- `context/skills/data-ai/ai-fundamentals/references/math-foundations.md` → `context/skills/data-ai/ai-fundamentals/references/math-foundations.md`
- `context/skills/data-ai/ai-fundamentals/references/ml-concepts.md` → `context/skills/data-ai/ai-fundamentals/references/ml-concepts.md`
- `context/skills/data-ai/ai-fundamentals/SKILL.md` → `context/skills/data-ai/ai-fundamentals/SKILL.md`
- `context/skills/data-ai/ml-pipeline/references/pipeline-stages.md` → `context/skills/data-ai/ml-pipeline/references/pipeline-stages.md`
- `context/skills/data-ai/ml-pipeline/SKILL.md` → `context/skills/data-ai/ml-pipeline/SKILL.md`

### skills/design

**changed-locally-only**

- `context/skills/design/mobile-app-design/references/platform-guidelines.md` → `context/skills/design/mobile-app-design/references/platform-guidelines.md`
- `context/skills/design/mobile-app-design/SKILL.md` → `context/skills/design/mobile-app-design/SKILL.md`
- `context/skills/design/seo-optimization/references/technical-seo-checklist.md` → `context/skills/design/seo-optimization/references/technical-seo-checklist.md`
- `context/skills/design/seo-optimization/SKILL.md` → `context/skills/design/seo-optimization/SKILL.md`
- `context/skills/design/logo-design/references/design-principles.md` → `context/skills/design/logo-design/references/design-principles.md`
- `context/skills/design/logo-design/SKILL.md` → `context/skills/design/logo-design/SKILL.md`
- `context/skills/design/excalidraw/references/json-schema.md` → `context/skills/design/excalidraw/references/json-schema.md`
- `context/skills/design/excalidraw/SKILL.md` → `context/skills/design/excalidraw/SKILL.md`
- `context/skills/design/frontend-design/references/accessibility-checklist.md` → `context/skills/design/frontend-design/references/accessibility-checklist.md`
- `context/skills/design/frontend-design/SKILL.md` → `context/skills/design/frontend-design/SKILL.md`

### skills/devops

**changed-locally-only**

- `context/skills/devops/linux-administration/references/commands-cheatsheet.md` → `context/skills/devops/linux-administration/references/commands-cheatsheet.md`
- `context/skills/devops/linux-administration/references/systemd-reference.md` → `context/skills/devops/linux-administration/references/systemd-reference.md`
- `context/skills/devops/linux-administration/SKILL.md` → `context/skills/devops/linux-administration/SKILL.md`
- `context/skills/devops/logging-strategy/references/structured-logging.md` → `context/skills/devops/logging-strategy/references/structured-logging.md`
- `context/skills/devops/logging-strategy/SKILL.md` → `context/skills/devops/logging-strategy/SKILL.md`
- `context/skills/devops/incident-response/SKILL.md` → `context/skills/devops/incident-response/SKILL.md`
- `context/skills/devops/metrics-monitoring/references/metric-types.md` → `context/skills/devops/metrics-monitoring/references/metric-types.md`
- `context/skills/devops/metrics-monitoring/SKILL.md` → `context/skills/devops/metrics-monitoring/SKILL.md`
- `context/skills/devops/terraform-basics/SKILL.md` → `context/skills/devops/terraform-basics/SKILL.md`
- `context/skills/devops/container-orchestration/references/compose-patterns.md` → `context/skills/devops/container-orchestration/references/compose-patterns.md`
- `context/skills/devops/container-orchestration/SKILL.md` → `context/skills/devops/container-orchestration/SKILL.md`
- `context/skills/devops/kubernetes-basics/references/cluster-architecture.md` → `context/skills/devops/kubernetes-basics/references/cluster-architecture.md`
- `context/skills/devops/kubernetes-basics/references/troubleshooting.md` → `context/skills/devops/kubernetes-basics/references/troubleshooting.md`
- `context/skills/devops/kubernetes-basics/references/resource-cheatsheet.md` → `context/skills/devops/kubernetes-basics/references/resource-cheatsheet.md`
- `context/skills/devops/kubernetes-basics/SKILL.md` → `context/skills/devops/kubernetes-basics/SKILL.md`
- `context/skills/devops/metrics-management/SKILL.md` → `context/skills/devops/metrics-management/SKILL.md`
- `context/skills/devops/metrics-management/assets/metric-spec.yaml` → `context/skills/devops/metrics-management/assets/metric-spec.yaml`
- `context/skills/devops/dns-networking/references/troubleshooting-tools.md` → `context/skills/devops/dns-networking/references/troubleshooting-tools.md`
- `context/skills/devops/dns-networking/references/protocol-reference.md` → `context/skills/devops/dns-networking/references/protocol-reference.md`
- `context/skills/devops/dns-networking/SKILL.md` → `context/skills/devops/dns-networking/SKILL.md`
- `context/skills/devops/postmortem-writing/SKILL.md` → `context/skills/devops/postmortem-writing/SKILL.md`
- `context/skills/devops/dockerfile-review/SKILL.md` → `context/skills/devops/dockerfile-review/SKILL.md`
- `context/skills/devops/distributed-tracing/SKILL.md` → `context/skills/devops/distributed-tracing/SKILL.md`
- `context/skills/devops/ci-cd-setup/SKILL.md` → `context/skills/devops/ci-cd-setup/SKILL.md`
- `context/skills/devops/release-semver/commands/pk-publish.md` → `context/skills/devops/release-semver/commands/pk-publish.md`
- `context/skills/devops/alerting-oncall/SKILL.md` → `context/skills/devops/alerting-oncall/SKILL.md`

**conflict**

- `context/skills/devops/release-semver/SKILL.md` → `context/skills/devops/release-semver/SKILL.md`
- `context/skills/devops/release-semver/commands/pk-release.md` → `context/skills/devops/release-semver/commands/pk-release.md`

### skills/documents

**changed-locally-only**

- `context/skills/documents/data-storytelling/SKILL.md` → `context/skills/documents/data-storytelling/SKILL.md`
- `context/skills/documents/data-visualization/references/chart-selection.md` → `context/skills/documents/data-visualization/references/chart-selection.md`
- `context/skills/documents/data-visualization/SKILL.md` → `context/skills/documents/data-visualization/SKILL.md`
- `context/skills/documents/infographics/references/best-practices.md` → `context/skills/documents/infographics/references/best-practices.md`
- `context/skills/documents/infographics/SKILL.md` → `context/skills/documents/infographics/SKILL.md`
- `context/skills/documents/docx-authoring/SKILL.md` → `context/skills/documents/docx-authoring/SKILL.md`
- `context/skills/documents/pdf-workflow/SKILL.md` → `context/skills/documents/pdf-workflow/SKILL.md`
- `context/skills/documents/xlsx-modeling/SKILL.md` → `context/skills/documents/xlsx-modeling/SKILL.md`
- `context/skills/documents/pptx-authoring/SKILL.md` → `context/skills/documents/pptx-authoring/SKILL.md`
- `context/skills/documents/latex-authoring/references/math-reference.md` → `context/skills/documents/latex-authoring/references/math-reference.md`
- `context/skills/documents/latex-authoring/references/packages.md` → `context/skills/documents/latex-authoring/references/packages.md`
- `context/skills/documents/latex-authoring/references/tikz-reference.md` → `context/skills/documents/latex-authoring/references/tikz-reference.md`
- `context/skills/documents/latex-authoring/SKILL.md` → `context/skills/documents/latex-authoring/SKILL.md`

### skills/engineering

**changed-locally-only**

- `context/skills/engineering/grpc-protobuf/references/proto-conventions.md` → `context/skills/engineering/grpc-protobuf/references/proto-conventions.md`
- `context/skills/engineering/grpc-protobuf/SKILL.md` → `context/skills/engineering/grpc-protobuf/SKILL.md`
- `context/skills/engineering/database-modeling/references/modeling-patterns.md` → `context/skills/engineering/database-modeling/references/modeling-patterns.md`
- `context/skills/engineering/database-modeling/SKILL.md` → `context/skills/engineering/database-modeling/SKILL.md`
- `context/skills/engineering/go-conventions/references/go-patterns.md` → `context/skills/engineering/go-conventions/references/go-patterns.md`
- `context/skills/engineering/go-conventions/SKILL.md` → `context/skills/engineering/go-conventions/SKILL.md`
- `context/skills/engineering/secret-management/SKILL.md` → `context/skills/engineering/secret-management/SKILL.md`
- `context/skills/engineering/error-handling/SKILL.md` → `context/skills/engineering/error-handling/SKILL.md`
- `context/skills/engineering/nosql-patterns/SKILL.md` → `context/skills/engineering/nosql-patterns/SKILL.md`
- `context/skills/engineering/changelog/SKILL.md` → `context/skills/engineering/changelog/SKILL.md`
- `context/skills/engineering/python-best-practices/SKILL.md` → `context/skills/engineering/python-best-practices/SKILL.md`
- `context/skills/engineering/pixijs-gamedev/references/api-cheatsheet.md` → `context/skills/engineering/pixijs-gamedev/references/api-cheatsheet.md`
- `context/skills/engineering/pixijs-gamedev/SKILL.md` → `context/skills/engineering/pixijs-gamedev/SKILL.md`
- `context/skills/engineering/code-review/SKILL.md` → `context/skills/engineering/code-review/SKILL.md`
- `context/skills/engineering/debugging/SKILL.md` → `context/skills/engineering/debugging/SKILL.md`
- `context/skills/engineering/auth-patterns/references/oauth-flows.md` → `context/skills/engineering/auth-patterns/references/oauth-flows.md`
- `context/skills/engineering/auth-patterns/references/jwt-reference.md` → `context/skills/engineering/auth-patterns/references/jwt-reference.md`
- `context/skills/engineering/auth-patterns/SKILL.md` → `context/skills/engineering/auth-patterns/SKILL.md`
- `context/skills/engineering/api-design/references/rest-conventions.md` → `context/skills/engineering/api-design/references/rest-conventions.md`
- `context/skills/engineering/api-design/references/openapi-patterns.md` → `context/skills/engineering/api-design/references/openapi-patterns.md`
- `context/skills/engineering/api-design/SKILL.md` → `context/skills/engineering/api-design/SKILL.md`
- `context/skills/engineering/integration-testing/references/test-fixtures.md` → `context/skills/engineering/integration-testing/references/test-fixtures.md`
- `context/skills/engineering/integration-testing/SKILL.md` → `context/skills/engineering/integration-testing/SKILL.md`
- `context/skills/engineering/fastapi-patterns/references/endpoint-patterns.md` → `context/skills/engineering/fastapi-patterns/references/endpoint-patterns.md`
- `context/skills/engineering/fastapi-patterns/SKILL.md` → `context/skills/engineering/fastapi-patterns/SKILL.md`
- `context/skills/engineering/dependency-audit/SKILL.md` → `context/skills/engineering/dependency-audit/SKILL.md`
- `context/skills/engineering/graphql-patterns/SKILL.md` → `context/skills/engineering/graphql-patterns/SKILL.md`
- `context/skills/engineering/threat-modeling/SKILL.md` → `context/skills/engineering/threat-modeling/SKILL.md`
- `context/skills/engineering/event-driven-architecture/references/messaging-patterns.md` → `context/skills/engineering/event-driven-architecture/references/messaging-patterns.md`
- `context/skills/engineering/event-driven-architecture/SKILL.md` → `context/skills/engineering/event-driven-architecture/SKILL.md`
- `context/skills/engineering/tailwind/references/cheatsheet.md` → `context/skills/engineering/tailwind/references/cheatsheet.md`
- `context/skills/engineering/tailwind/SKILL.md` → `context/skills/engineering/tailwind/SKILL.md`
- `context/skills/engineering/performance-profiling/references/profiling-tools.md` → `context/skills/engineering/performance-profiling/references/profiling-tools.md`
- `context/skills/engineering/performance-profiling/SKILL.md` → `context/skills/engineering/performance-profiling/SKILL.md`
- `context/skills/engineering/code-generation/SKILL.md` → `context/skills/engineering/code-generation/SKILL.md`
- `context/skills/engineering/caching-strategies/SKILL.md` → `context/skills/engineering/caching-strategies/SKILL.md`
- `context/skills/engineering/shell-scripting/references/bash-patterns.md` → `context/skills/engineering/shell-scripting/references/bash-patterns.md`
- `context/skills/engineering/shell-scripting/SKILL.md` → `context/skills/engineering/shell-scripting/SKILL.md`
- `context/skills/engineering/concurrency-patterns/references/patterns-catalog.md` → `context/skills/engineering/concurrency-patterns/references/patterns-catalog.md`
- `context/skills/engineering/concurrency-patterns/SKILL.md` → `context/skills/engineering/concurrency-patterns/SKILL.md`
- `context/skills/engineering/secure-coding/references/owasp-checklist.md` → `context/skills/engineering/secure-coding/references/owasp-checklist.md`
- `context/skills/engineering/secure-coding/SKILL.md` → `context/skills/engineering/secure-coding/SKILL.md`
- `context/skills/engineering/dependency-management/SKILL.md` → `context/skills/engineering/dependency-management/SKILL.md`
- `context/skills/engineering/testing-strategy/SKILL.md` → `context/skills/engineering/testing-strategy/SKILL.md`
- `context/skills/engineering/webhook-integration/SKILL.md` → `context/skills/engineering/webhook-integration/SKILL.md`
- `context/skills/engineering/tdd-workflow/SKILL.md` → `context/skills/engineering/tdd-workflow/SKILL.md`
- `context/skills/engineering/microservice-creation/SKILL.md` → `context/skills/engineering/microservice-creation/SKILL.md`
- `context/skills/engineering/software-architecture/references/patterns.md` → `context/skills/engineering/software-architecture/references/patterns.md`
- `context/skills/engineering/software-architecture/SKILL.md` → `context/skills/engineering/software-architecture/SKILL.md`
- `context/skills/engineering/system-design/references/estimation-cheatsheet.md` → `context/skills/engineering/system-design/references/estimation-cheatsheet.md`
- `context/skills/engineering/system-design/SKILL.md` → `context/skills/engineering/system-design/SKILL.md`
- `context/skills/engineering/java-patterns/SKILL.md` → `context/skills/engineering/java-patterns/SKILL.md`
- `context/skills/engineering/load-testing/SKILL.md` → `context/skills/engineering/load-testing/SKILL.md`
- `context/skills/engineering/reflex-python/references/component-reference.md` → `context/skills/engineering/reflex-python/references/component-reference.md`
- `context/skills/engineering/reflex-python/SKILL.md` → `context/skills/engineering/reflex-python/SKILL.md`
- `context/skills/engineering/refactoring/references/gof-patterns.md` → `context/skills/engineering/refactoring/references/gof-patterns.md`
- `context/skills/engineering/refactoring/references/code-smells.md` → `context/skills/engineering/refactoring/references/code-smells.md`
- `context/skills/engineering/refactoring/SKILL.md` → `context/skills/engineering/refactoring/SKILL.md`
- `context/skills/engineering/software-modularization/SKILL.md` → `context/skills/engineering/software-modularization/SKILL.md`
- `context/skills/engineering/sql-style-guide/SKILL.md` → `context/skills/engineering/sql-style-guide/SKILL.md`
- `context/skills/engineering/database-migration/SKILL.md` → `context/skills/engineering/database-migration/SKILL.md`
- `context/skills/engineering/flutter-development/references/widget-catalog.md` → `context/skills/engineering/flutter-development/references/widget-catalog.md`
- `context/skills/engineering/flutter-development/SKILL.md` → `context/skills/engineering/flutter-development/SKILL.md`
- `context/skills/engineering/rust-conventions/SKILL.md` → `context/skills/engineering/rust-conventions/SKILL.md`
- `context/skills/engineering/domain-driven-design/references/ddd-building-blocks.md` → `context/skills/engineering/domain-driven-design/references/ddd-building-blocks.md`
- `context/skills/engineering/domain-driven-design/SKILL.md` → `context/skills/engineering/domain-driven-design/SKILL.md`
- `context/skills/engineering/typescript-patterns/SKILL.md` → `context/skills/engineering/typescript-patterns/SKILL.md`
- `context/skills/engineering/sql-patterns/references/query-patterns.md` → `context/skills/engineering/sql-patterns/references/query-patterns.md`
- `context/skills/engineering/sql-patterns/references/schema-design.md` → `context/skills/engineering/sql-patterns/references/schema-design.md`
- `context/skills/engineering/sql-patterns/SKILL.md` → `context/skills/engineering/sql-patterns/SKILL.md`

**conflict**

- `context/skills/engineering/git-workflow/SKILL.md` → `context/skills/engineering/git-workflow/SKILL.md`

**new-upstream**

- `context/skills/engineering/git-branching/references/strategies.md` → `context/skills/engineering/git-branching/references/strategies.md`
- `context/skills/engineering/git-branching/SKILL.md` → `context/skills/engineering/git-branching/SKILL.md`

### skills/processkit

**changed-locally-only**

- `context/skills/processkit/supply-chain-audit/mcp/server.py` → `context/skills/processkit/supply-chain-audit/mcp/server.py`
- `context/skills/processkit/supply-chain-audit/mcp/SERVER.md` → `context/skills/processkit/supply-chain-audit/mcp/SERVER.md`
- `context/skills/processkit/supply-chain-audit/mcp/mcp-config.json` → `context/skills/processkit/supply-chain-audit/mcp/mcp-config.json`
- `context/skills/processkit/supply-chain-audit/scripts/supply_chain_audit.py` → `context/skills/processkit/supply-chain-audit/scripts/supply_chain_audit.py`
- `context/skills/processkit/supply-chain-audit/scripts/test_supply_chain_audit.py` → `context/skills/processkit/supply-chain-audit/scripts/test_supply_chain_audit.py`
- `context/skills/processkit/supply-chain-audit/SKILL.md` → `context/skills/processkit/supply-chain-audit/SKILL.md`
- `context/skills/processkit/supply-chain-audit/commands/pk-supply-chain.md` → `context/skills/processkit/supply-chain-audit/commands/pk-supply-chain.md`

**conflict**

- `context/skills/processkit/repository-portfolio-review/SKILL.md` → `context/skills/processkit/repository-portfolio-review/SKILL.md`

**new-upstream**

- `context/skills/processkit/project-reconciliation/SKILL.md` → `context/skills/processkit/project-reconciliation/SKILL.md`
- `context/skills/processkit/project-reconciliation/commands/pk-reconcile.md` → `context/skills/processkit/project-reconciliation/commands/pk-reconcile.md`
- `context/skills/processkit/pk-doctor/scripts/test_pk_doctor_mcp.py` → `context/skills/processkit/pk-doctor/scripts/test_pk_doctor_mcp.py`

### skills/product

**changed-locally-only**

- `context/skills/product/estimation-planning/SKILL.md` → `context/skills/product/estimation-planning/SKILL.md`
- `context/skills/product/documentation/SKILL.md` → `context/skills/product/documentation/SKILL.md`
- `context/skills/product/sprint-retrospective/SKILL.md` → `context/skills/product/sprint-retrospective/SKILL.md`
- `context/skills/product/research-with-confidence/SKILL.md` → `context/skills/product/research-with-confidence/SKILL.md`
- `context/skills/product/research-with-confidence/commands/pk-research.md` → `context/skills/product/research-with-confidence/commands/pk-research.md`
- `context/skills/product/devils-advocate/SKILL.md` → `context/skills/product/devils-advocate/SKILL.md`
- `context/skills/product/user-research/SKILL.md` → `context/skills/product/user-research/SKILL.md`
- `context/skills/product/legal-review/SKILL.md` → `context/skills/product/legal-review/SKILL.md`
- `context/skills/product/onboarding-guide/SKILL.md` → `context/skills/product/onboarding-guide/SKILL.md`
- `context/skills/product/email-drafter/SKILL.md` → `context/skills/product/email-drafter/SKILL.md`
- `context/skills/product/board-of-advisors/SKILL.md` → `context/skills/product/board-of-advisors/SKILL.md`
- `context/skills/product/prd-writing/SKILL.md` → `context/skills/product/prd-writing/SKILL.md`
