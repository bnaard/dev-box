---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260717_1534-ContentSync-processkit-content-sync
  created: 2026-07-17 15:34:00+00:00
  updated: '2026-07-17T15:34:13+00:00'
spec:
  source: processkit
  source_url: https://github.com/projectious-work/processkit.git
  from_version: v0.27.2
  to_version: v0.27.4
  state: applied
  generated_by: aibox apply
  generated_at: 2026-07-17 15:34:00+00:00
  summary: 0 changed upstream, 0 conflicts, 289 new, 0 removed, 0 stale-removed (28
    groups affected)
  affected_groups:
  - AGENTS
  - context
  - context/artifacts
  - context/roles
  - lib
  - schemas/INDEX
  - schemas/actor
  - schemas/artifact
  - schemas/binding
  - schemas/category
  - schemas/constraint
  - schemas/context
  - schemas/decisionrecord
  - schemas/discussion
  - schemas/gate
  - schemas/logentry
  - schemas/migration
  - schemas/note
  - schemas/role
  - schemas/roleslot
  - schemas/scope
  - schemas/team-member
  - schemas/workitem
  - skills/INDEX.md
  - skills/data-ai
  - skills/design
  - skills/devops
  - skills/processkit
  affected_files:
  - path: AGENTS.md
    classification: changed-locally-only
  - path: context/.processkit-mcp-manifest.json
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen2-5-72b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen2-5-coder-32b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-235b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-max-preview.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-plus.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-coder.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-anthropic-claude-haiku.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-anthropic-claude-opus.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-anthropic-claude-sonnet.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-aws-amazon-nova-2-lite.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-a.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-r-plus.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-r.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v4-flash.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v4-pro.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-2-5-flash.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-2-5-pro.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-1-pro.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-flash-preview.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-flash.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-pro-preview.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-flash.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-google-gemma-3-27b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-3-70b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-4-maverick.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-4-scout.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-microsoft-phi.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-m.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-text.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-mistral-codestral.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-mistral-devstral.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-deep-think.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-large.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-medium.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-small.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-moonshot-kimi-k2-6.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-nvidia-nvidia-nemotron-3-super-120b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-4o.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-2-codex.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-2-pro.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-5-pro.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-pro.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-o3.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-o4-mini.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-3-5.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-3.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-1-fast.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-1.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-heavy.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-xai-grok.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-zai-glm.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1832-ModelProfile-code-balanced.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1832-ModelProfile-code-deep.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1832-ModelProfile-code-fast.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1832-ModelProfile-general-balanced.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1832-ModelProfile-general-deep.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1832-ModelProfile-general-fast.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1832-ModelProfile-governed-deep.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1832-ModelProfile-research-deep.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1832-ModelProfile-writing-balanced.md
    classification: new-upstream
  - path: context/artifacts/ART-20260504_1425-ModelSpec-openai-gpt-5-3-codex-spark.md
    classification: new-upstream
  - path: context/artifacts/ART-20260505_1545-ModelSpec-xiaomi-mimo-7b.md
    classification: new-upstream
  - path: context/roles/ROLE-account-executive.md
    classification: new-upstream
  - path: context/roles/ROLE-ai-research-scientist.md
    classification: new-upstream
  - path: context/roles/ROLE-assistant.md
    classification: new-upstream
  - path: context/roles/ROLE-brand-manager.md
    classification: new-upstream
  - path: context/roles/ROLE-business-operations-analyst.md
    classification: new-upstream
  - path: context/roles/ROLE-ceo.md
    classification: new-upstream
  - path: context/roles/ROLE-cfo.md
    classification: new-upstream
  - path: context/roles/ROLE-chief-of-staff.md
    classification: new-upstream
  - path: context/roles/ROLE-cloud-architect.md
    classification: new-upstream
  - path: context/roles/ROLE-cloud-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-cmo.md
    classification: new-upstream
  - path: context/roles/ROLE-community-manager.md
    classification: new-upstream
  - path: context/roles/ROLE-compliance-officer.md
    classification: new-upstream
  - path: context/roles/ROLE-content-marketer.md
    classification: new-upstream
  - path: context/roles/ROLE-controller.md
    classification: new-upstream
  - path: context/roles/ROLE-coo.md
    classification: new-upstream
  - path: context/roles/ROLE-cpo.md
    classification: new-upstream
  - path: context/roles/ROLE-cto.md
    classification: new-upstream
  - path: context/roles/ROLE-customer-success-manager.md
    classification: new-upstream
  - path: context/roles/ROLE-data-architect.md
    classification: new-upstream
  - path: context/roles/ROLE-data-protection-officer.md
    classification: new-upstream
  - path: context/roles/ROLE-data-scientist.md
    classification: new-upstream
  - path: context/roles/ROLE-database-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-devops-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-embedded-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-enterprise-architect.md
    classification: new-upstream
  - path: context/roles/ROLE-financial-analyst.md
    classification: new-upstream
  - path: context/roles/ROLE-general-counsel.md
    classification: new-upstream
  - path: context/roles/ROLE-learning-development-manager.md
    classification: new-upstream
  - path: context/roles/ROLE-machine-learning-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-observability-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-pr-manager.md
    classification: new-upstream
  - path: context/roles/ROLE-product-designer.md
    classification: new-upstream
  - path: context/roles/ROLE-product-manager.md
    classification: new-upstream
  - path: context/roles/ROLE-product-marketing-manager.md
    classification: new-upstream
  - path: context/roles/ROLE-product-owner.md
    classification: new-upstream
  - path: context/roles/ROLE-program-manager.md
    classification: new-upstream
  - path: context/roles/ROLE-qa-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-recruiter.md
    classification: new-upstream
  - path: context/roles/ROLE-research-scientist.md
    classification: new-upstream
  - path: context/roles/ROLE-sales-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-scrum-master.md
    classification: new-upstream
  - path: context/roles/ROLE-security-architect.md
    classification: new-upstream
  - path: context/roles/ROLE-security-operations-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-seo-specialist.md
    classification: new-upstream
  - path: context/roles/ROLE-site-reliability-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-software-engineer.md
    classification: new-upstream
  - path: context/roles/ROLE-solutions-architect.md
    classification: new-upstream
  - path: context/roles/ROLE-technical-writer.md
    classification: new-upstream
  - path: context/roles/ROLE-treasury-analyst.md
    classification: new-upstream
  - path: context/roles/ROLE-ux-designer.md
    classification: new-upstream
  - path: context/schemas/INDEX.md
    classification: new-upstream
  - path: context/schemas/actor.yaml
    classification: new-upstream
  - path: context/schemas/artifact.yaml
    classification: new-upstream
  - path: context/schemas/binding.yaml
    classification: new-upstream
  - path: context/schemas/category.yaml
    classification: new-upstream
  - path: context/schemas/constraint.yaml
    classification: new-upstream
  - path: context/schemas/context.yaml
    classification: new-upstream
  - path: context/schemas/decisionrecord.yaml
    classification: new-upstream
  - path: context/schemas/discussion.yaml
    classification: new-upstream
  - path: context/schemas/gate.yaml
    classification: new-upstream
  - path: context/schemas/logentry.yaml
    classification: new-upstream
  - path: context/schemas/migration.yaml
    classification: new-upstream
  - path: context/schemas/note.yaml
    classification: new-upstream
  - path: context/schemas/role.yaml
    classification: new-upstream
  - path: context/schemas/roleslot.yaml
    classification: new-upstream
  - path: context/schemas/scope.yaml
    classification: new-upstream
  - path: context/schemas/team-member.yaml
    classification: new-upstream
  - path: context/schemas/workitem.yaml
    classification: new-upstream
  - path: context/skills/INDEX.md
    classification: new-upstream
  - path: context/skills/_lib/README.md
    classification: new-upstream
  - path: context/skills/_lib/processkit/__init__.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/config.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/entity.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/frontmatter.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/__init__.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/catalog.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/lazy.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/loader.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/naming.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/permissions.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/proxy.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/registry.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/runtime.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/session.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/gateway/transports.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/ids.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/index.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/log.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/paths.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/schema.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/state_machine.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/test_frontmatter.py
    classification: new-upstream
  - path: context/skills/_lib/processkit/test_ids.py
    classification: new-upstream
  - path: context/skills/data-ai/ai-fundamentals/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/ai-fundamentals/references/math-foundations.md
    classification: new-upstream
  - path: context/skills/data-ai/ai-fundamentals/references/ml-concepts.md
    classification: new-upstream
  - path: context/skills/data-ai/data-pipeline/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/data-quality/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/data-science/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/data-science/references/statistical-methods.md
    classification: new-upstream
  - path: context/skills/data-ai/data-science/references/tidy-data-principles.md
    classification: new-upstream
  - path: context/skills/data-ai/data-science/references/visualization-guidelines.md
    classification: new-upstream
  - path: context/skills/data-ai/embedding-vectordb/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/feature-engineering/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/llm-evaluation/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/ml-pipeline/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/ml-pipeline/references/pipeline-stages.md
    classification: new-upstream
  - path: context/skills/data-ai/pandas-polars/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/pandas-polars/references/api-comparison.md
    classification: new-upstream
  - path: context/skills/data-ai/prompt-engineering/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/prompt-engineering/references/techniques-catalog.md
    classification: new-upstream
  - path: context/skills/data-ai/rag-engineering/SKILL.md
    classification: new-upstream
  - path: context/skills/data-ai/rag-engineering/references/chunking-strategies.md
    classification: new-upstream
  - path: context/skills/data-ai/rag-engineering/references/evaluation.md
    classification: new-upstream
  - path: context/skills/data-ai/rag-engineering/references/retrieval-patterns.md
    classification: new-upstream
  - path: context/skills/design/excalidraw/SKILL.md
    classification: new-upstream
  - path: context/skills/design/excalidraw/references/json-schema.md
    classification: new-upstream
  - path: context/skills/design/frontend-design/SKILL.md
    classification: new-upstream
  - path: context/skills/design/frontend-design/references/accessibility-checklist.md
    classification: new-upstream
  - path: context/skills/design/logo-design/SKILL.md
    classification: new-upstream
  - path: context/skills/design/logo-design/references/design-principles.md
    classification: new-upstream
  - path: context/skills/design/mobile-app-design/SKILL.md
    classification: new-upstream
  - path: context/skills/design/mobile-app-design/references/platform-guidelines.md
    classification: new-upstream
  - path: context/skills/design/seo-optimization/SKILL.md
    classification: new-upstream
  - path: context/skills/design/seo-optimization/references/technical-seo-checklist.md
    classification: new-upstream
  - path: context/skills/devops/alerting-oncall/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/ci-cd-setup/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/container-orchestration/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/container-orchestration/references/compose-patterns.md
    classification: new-upstream
  - path: context/skills/devops/distributed-tracing/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/dns-networking/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/dns-networking/references/protocol-reference.md
    classification: new-upstream
  - path: context/skills/devops/dns-networking/references/troubleshooting-tools.md
    classification: new-upstream
  - path: context/skills/devops/dockerfile-review/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/incident-response/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/kubernetes-basics/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/kubernetes-basics/references/cluster-architecture.md
    classification: new-upstream
  - path: context/skills/devops/kubernetes-basics/references/resource-cheatsheet.md
    classification: new-upstream
  - path: context/skills/devops/kubernetes-basics/references/troubleshooting.md
    classification: new-upstream
  - path: context/skills/devops/linux-administration/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/linux-administration/references/commands-cheatsheet.md
    classification: new-upstream
  - path: context/skills/devops/linux-administration/references/systemd-reference.md
    classification: new-upstream
  - path: context/skills/devops/logging-strategy/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/logging-strategy/references/structured-logging.md
    classification: new-upstream
  - path: context/skills/devops/metrics-management/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/metrics-management/assets/metric-spec.yaml
    classification: new-upstream
  - path: context/skills/devops/metrics-monitoring/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/metrics-monitoring/references/metric-types.md
    classification: new-upstream
  - path: context/skills/devops/postmortem-writing/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/release-semver/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/release-semver/commands/pk-publish.md
    classification: new-upstream
  - path: context/skills/devops/release-semver/commands/pk-release.md
    classification: new-upstream
  - path: context/skills/devops/repo-management/SKILL.md
    classification: new-upstream
  - path: context/skills/devops/repo-management/commands/pk-repo-reconcile.md
    classification: new-upstream
  - path: context/skills/devops/repo-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/devops/repo-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/devops/repo-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/devops/repo-management/scripts/test_repo_management.py
    classification: new-upstream
  - path: context/skills/devops/terraform-basics/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/agent-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/agent-management/references/coordination-patterns.md
    classification: new-upstream
  - path: context/skills/processkit/aggregate-mcp/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/aggregate-mcp/mcp/mcp-config.aggregate.json
    classification: new-upstream
  - path: context/skills/processkit/aggregate-mcp/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/aggregate-mcp/scripts/test_aggregate_mcp.py
    classification: new-upstream
  - path: context/skills/processkit/artifact-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/artifact-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/artifact-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/artifact-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/binding-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/binding-management/assets/binding.yaml
    classification: new-upstream
  - path: context/skills/processkit/binding-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/binding-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/binding-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/binding-management/scripts/test_binding_management.py
    classification: new-upstream
  - path: context/skills/processkit/eval-gate-authoring/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/eval-gate-authoring/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/eval-gate-authoring/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/eval-gate-authoring/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/eval-gate-authoring/scripts/test_eval_gate_authoring.py
    classification: new-upstream
  - path: context/skills/processkit/event-log/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/event-log/assets/logentry.yaml
    classification: new-upstream
  - path: context/skills/processkit/event-log/examples/workitem-transitioned.md
    classification: new-upstream
  - path: context/skills/processkit/event-log/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/event-log/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/event-log/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/event-log/scripts/test_log_event.py
    classification: new-upstream
  - path: context/skills/processkit/index-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/index-management/config/settings.toml
    classification: new-upstream
  - path: context/skills/processkit/index-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/index-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/index-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/index-management/scripts/test_db_result_limits.py
    classification: new-upstream
  - path: context/skills/processkit/index-management/scripts/test_get_entity_by_path_and_list_entities.py
    classification: new-upstream
  - path: context/skills/processkit/index-management/scripts/test_index_management_v1_penalty.py
    classification: new-upstream
  - path: context/skills/processkit/note-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/note-management/commands/pk-note-promote.md
    classification: new-upstream
  - path: context/skills/processkit/note-management/commands/pk-note-review.md
    classification: new-upstream
  - path: context/skills/processkit/note-management/commands/pk-note.md
    classification: new-upstream
  - path: context/skills/processkit/note-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/note-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/note-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/assets/OWNER-20260409_2054-GoalsAnd-context.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/assets/OWNER-20260409_2054-TeamAnd-relationships.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/assets/OWNER-20260409_2054-WorkingStyle.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/assets/goals-and-context.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/assets/identity.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/assets/team-and-relationships.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/assets/working-style.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/commands/pk-observe.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/commands/pk-owner-bootstrap.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/references/interview-protocol.md
    classification: new-upstream
  - path: context/skills/processkit/owner-profiling/references/observable-signals.md
    classification: new-upstream
  - path: context/skills/processkit/process-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/retrospective/scripts/pk_retro.py
    classification: new-upstream
  - path: context/skills/processkit/retrospective/scripts/signals/__init__.py
    classification: new-upstream
  - path: context/skills/processkit/retrospective/scripts/signals/drift.py
    classification: new-upstream
  - path: context/skills/processkit/retrospective/scripts/signals/release_summary.py
    classification: new-upstream
  - path: context/skills/processkit/retrospective/scripts/signals/timeline.py
    classification: new-upstream
  - path: context/skills/processkit/retrospective/scripts/signals/workitems.py
    classification: new-upstream
  started_at: '2026-07-17T15:34:13+00:00'
  applied_at: '2026-07-17T15:34:13+00:00'
---

# Migration MIG-20260717_1534-ContentSync-processkit-content-sync

From `v0.27.2` to `v0.27.4` (source: `https://github.com/projectious-work/processkit.git`).

0 changed upstream, 0 conflicts, 289 new, 0 removed, 0 stale-removed (28 groups affected)

## Counts

- unchanged: 418
- changed-locally-only: 1
- changed-upstream-only: 0
- conflict: 0
- new-upstream: 289
- removed-upstream: 0
- removed-upstream-stale: 0

## Changes by group

### AGENTS

**changed-locally-only**

- `AGENTS.md` → `AGENTS.md`

### context

**new-upstream**

- `context/.processkit-mcp-manifest.json` → `context/.processkit-mcp-manifest.json`

### context/artifacts

**new-upstream**

- `context/artifacts/ART-20260503_1424-ModelSpec-anthropic-claude-haiku.md` → `context/artifacts/ART-20260503_1424-ModelSpec-anthropic-claude-haiku.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-mistral-codestral.md` → `context/artifacts/ART-20260503_1424-ModelSpec-mistral-codestral.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v.md` → `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-flash-preview.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-flash-preview.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-3.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-3.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-1-pro.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-1-pro.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-zai-glm.md` → `context/artifacts/ART-20260503_1424-ModelSpec-zai-glm.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-r.md` → `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-r.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-governed-deep.md` → `context/artifacts/ART-20260503_1832-ModelProfile-governed-deep.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-code-fast.md` → `context/artifacts/ART-20260503_1832-ModelProfile-code-fast.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-writing-balanced.md` → `context/artifacts/ART-20260503_1832-ModelProfile-writing-balanced.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-flash.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-flash.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-o3.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-o3.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-m.md` → `context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-m.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-general-deep.md` → `context/artifacts/ART-20260503_1832-ModelProfile-general-deep.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-nvidia-nvidia-nemotron-3-super-120b.md` → `context/artifacts/ART-20260503_1424-ModelSpec-nvidia-nvidia-nemotron-3-super-120b.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-code-balanced.md` → `context/artifacts/ART-20260503_1832-ModelProfile-code-balanced.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemma-3-27b.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemma-3-27b.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-research-deep.md` → `context/artifacts/ART-20260503_1832-ModelProfile-research-deep.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-r-plus.md` → `context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-r-plus.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-o4-mini.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-o4-mini.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-2-codex.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-2-codex.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-2-5-pro.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-2-5-pro.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-3-70b.md` → `context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-3-70b.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-2-pro.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-2-pro.md`
- `context/artifacts/ART-20260505_1545-ModelSpec-xiaomi-mimo-7b.md` → `context/artifacts/ART-20260505_1545-ModelSpec-xiaomi-mimo-7b.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-small.md` → `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-small.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-large.md` → `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-large.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-coder.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-coder.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-plus.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-plus.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-1.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-1.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-anthropic-claude-opus.md` → `context/artifacts/ART-20260503_1424-ModelSpec-anthropic-claude-opus.md`
- `context/artifacts/ART-20260504_1425-ModelSpec-openai-gpt-5-3-codex-spark.md` → `context/artifacts/ART-20260504_1425-ModelSpec-openai-gpt-5-3-codex-spark.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-1-fast.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-1-fast.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-code-deep.md` → `context/artifacts/ART-20260503_1832-ModelProfile-code-deep.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v4-flash.md` → `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v4-flash.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-4o.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-4o.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-235b.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-235b.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-deep-think.md` → `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-deep-think.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-5-pro.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-5-pro.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-4-maverick.md` → `context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-4-maverick.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen2-5-coder-32b.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen2-5-coder-32b.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-anthropic-claude-sonnet.md` → `context/artifacts/ART-20260503_1424-ModelSpec-anthropic-claude-sonnet.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-general-balanced.md` → `context/artifacts/ART-20260503_1832-ModelProfile-general-balanced.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-text.md` → `context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-text.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen2-5-72b.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen2-5-72b.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-general-fast.md` → `context/artifacts/ART-20260503_1832-ModelProfile-general-fast.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-max-preview.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-max-preview.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-3-5.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-3-5.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-aws-amazon-nova-2-lite.md` → `context/artifacts/ART-20260503_1424-ModelSpec-aws-amazon-nova-2-lite.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-heavy.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-heavy.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-a.md` → `context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-a.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-microsoft-phi.md` → `context/artifacts/ART-20260503_1424-ModelSpec-microsoft-phi.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-mistral-devstral.md` → `context/artifacts/ART-20260503_1424-ModelSpec-mistral-devstral.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v4-pro.md` → `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v4-pro.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-moonshot-kimi-k2-6.md` → `context/artifacts/ART-20260503_1424-ModelSpec-moonshot-kimi-k2-6.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-medium.md` → `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-medium.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-4-scout.md` → `context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-4-scout.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-pro-preview.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-pro-preview.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-2-5-flash.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-2-5-flash.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-pro.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-pro.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-flash.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-flash.md`

### context/roles

**new-upstream**

- `context/roles/ROLE-security-operations-engineer.md` → `context/roles/ROLE-security-operations-engineer.md`
- `context/roles/ROLE-customer-success-manager.md` → `context/roles/ROLE-customer-success-manager.md`
- `context/roles/ROLE-observability-engineer.md` → `context/roles/ROLE-observability-engineer.md`
- `context/roles/ROLE-database-engineer.md` → `context/roles/ROLE-database-engineer.md`
- `context/roles/ROLE-product-owner.md` → `context/roles/ROLE-product-owner.md`
- `context/roles/ROLE-seo-specialist.md` → `context/roles/ROLE-seo-specialist.md`
- `context/roles/ROLE-product-designer.md` → `context/roles/ROLE-product-designer.md`
- `context/roles/ROLE-security-architect.md` → `context/roles/ROLE-security-architect.md`
- `context/roles/ROLE-ai-research-scientist.md` → `context/roles/ROLE-ai-research-scientist.md`
- `context/roles/ROLE-cloud-engineer.md` → `context/roles/ROLE-cloud-engineer.md`
- `context/roles/ROLE-product-manager.md` → `context/roles/ROLE-product-manager.md`
- `context/roles/ROLE-cloud-architect.md` → `context/roles/ROLE-cloud-architect.md`
- `context/roles/ROLE-technical-writer.md` → `context/roles/ROLE-technical-writer.md`
- `context/roles/ROLE-learning-development-manager.md` → `context/roles/ROLE-learning-development-manager.md`
- `context/roles/ROLE-scrum-master.md` → `context/roles/ROLE-scrum-master.md`
- `context/roles/ROLE-ceo.md` → `context/roles/ROLE-ceo.md`
- `context/roles/ROLE-research-scientist.md` → `context/roles/ROLE-research-scientist.md`
- `context/roles/ROLE-machine-learning-engineer.md` → `context/roles/ROLE-machine-learning-engineer.md`
- `context/roles/ROLE-cfo.md` → `context/roles/ROLE-cfo.md`
- `context/roles/ROLE-cpo.md` → `context/roles/ROLE-cpo.md`
- `context/roles/ROLE-general-counsel.md` → `context/roles/ROLE-general-counsel.md`
- `context/roles/ROLE-cto.md` → `context/roles/ROLE-cto.md`
- `context/roles/ROLE-assistant.md` → `context/roles/ROLE-assistant.md`
- `context/roles/ROLE-controller.md` → `context/roles/ROLE-controller.md`
- `context/roles/ROLE-product-marketing-manager.md` → `context/roles/ROLE-product-marketing-manager.md`
- `context/roles/ROLE-cmo.md` → `context/roles/ROLE-cmo.md`
- `context/roles/ROLE-pr-manager.md` → `context/roles/ROLE-pr-manager.md`
- `context/roles/ROLE-coo.md` → `context/roles/ROLE-coo.md`
- `context/roles/ROLE-community-manager.md` → `context/roles/ROLE-community-manager.md`
- `context/roles/ROLE-sales-engineer.md` → `context/roles/ROLE-sales-engineer.md`
- `context/roles/ROLE-program-manager.md` → `context/roles/ROLE-program-manager.md`
- `context/roles/ROLE-recruiter.md` → `context/roles/ROLE-recruiter.md`
- `context/roles/ROLE-account-executive.md` → `context/roles/ROLE-account-executive.md`
- `context/roles/ROLE-financial-analyst.md` → `context/roles/ROLE-financial-analyst.md`
- `context/roles/ROLE-enterprise-architect.md` → `context/roles/ROLE-enterprise-architect.md`
- `context/roles/ROLE-content-marketer.md` → `context/roles/ROLE-content-marketer.md`
- `context/roles/ROLE-embedded-engineer.md` → `context/roles/ROLE-embedded-engineer.md`
- `context/roles/ROLE-devops-engineer.md` → `context/roles/ROLE-devops-engineer.md`
- `context/roles/ROLE-data-scientist.md` → `context/roles/ROLE-data-scientist.md`
- `context/roles/ROLE-treasury-analyst.md` → `context/roles/ROLE-treasury-analyst.md`
- `context/roles/ROLE-solutions-architect.md` → `context/roles/ROLE-solutions-architect.md`
- `context/roles/ROLE-qa-engineer.md` → `context/roles/ROLE-qa-engineer.md`
- `context/roles/ROLE-ux-designer.md` → `context/roles/ROLE-ux-designer.md`
- `context/roles/ROLE-site-reliability-engineer.md` → `context/roles/ROLE-site-reliability-engineer.md`
- `context/roles/ROLE-data-protection-officer.md` → `context/roles/ROLE-data-protection-officer.md`
- `context/roles/ROLE-chief-of-staff.md` → `context/roles/ROLE-chief-of-staff.md`
- `context/roles/ROLE-brand-manager.md` → `context/roles/ROLE-brand-manager.md`
- `context/roles/ROLE-compliance-officer.md` → `context/roles/ROLE-compliance-officer.md`
- `context/roles/ROLE-data-architect.md` → `context/roles/ROLE-data-architect.md`
- `context/roles/ROLE-business-operations-analyst.md` → `context/roles/ROLE-business-operations-analyst.md`
- `context/roles/ROLE-software-engineer.md` → `context/roles/ROLE-software-engineer.md`

### lib

**new-upstream**

- `context/skills/_lib/README.md` → `context/skills/_lib/README.md`
- `context/skills/_lib/processkit/config.py` → `context/skills/_lib/processkit/config.py`
- `context/skills/_lib/processkit/frontmatter.py` → `context/skills/_lib/processkit/frontmatter.py`
- `context/skills/_lib/processkit/index.py` → `context/skills/_lib/processkit/index.py`
- `context/skills/_lib/processkit/paths.py` → `context/skills/_lib/processkit/paths.py`
- `context/skills/_lib/processkit/log.py` → `context/skills/_lib/processkit/log.py`
- `context/skills/_lib/processkit/__init__.py` → `context/skills/_lib/processkit/__init__.py`
- `context/skills/_lib/processkit/test_frontmatter.py` → `context/skills/_lib/processkit/test_frontmatter.py`
- `context/skills/_lib/processkit/state_machine.py` → `context/skills/_lib/processkit/state_machine.py`
- `context/skills/_lib/processkit/test_ids.py` → `context/skills/_lib/processkit/test_ids.py`
- `context/skills/_lib/processkit/entity.py` → `context/skills/_lib/processkit/entity.py`
- `context/skills/_lib/processkit/ids.py` → `context/skills/_lib/processkit/ids.py`
- `context/skills/_lib/processkit/gateway/catalog.py` → `context/skills/_lib/processkit/gateway/catalog.py`
- `context/skills/_lib/processkit/gateway/naming.py` → `context/skills/_lib/processkit/gateway/naming.py`
- `context/skills/_lib/processkit/gateway/registry.py` → `context/skills/_lib/processkit/gateway/registry.py`
- `context/skills/_lib/processkit/gateway/proxy.py` → `context/skills/_lib/processkit/gateway/proxy.py`
- `context/skills/_lib/processkit/gateway/session.py` → `context/skills/_lib/processkit/gateway/session.py`
- `context/skills/_lib/processkit/gateway/__init__.py` → `context/skills/_lib/processkit/gateway/__init__.py`
- `context/skills/_lib/processkit/gateway/runtime.py` → `context/skills/_lib/processkit/gateway/runtime.py`
- `context/skills/_lib/processkit/gateway/loader.py` → `context/skills/_lib/processkit/gateway/loader.py`
- `context/skills/_lib/processkit/gateway/permissions.py` → `context/skills/_lib/processkit/gateway/permissions.py`
- `context/skills/_lib/processkit/gateway/transports.py` → `context/skills/_lib/processkit/gateway/transports.py`
- `context/skills/_lib/processkit/gateway/lazy.py` → `context/skills/_lib/processkit/gateway/lazy.py`
- `context/skills/_lib/processkit/schema.py` → `context/skills/_lib/processkit/schema.py`

### schemas/INDEX

**new-upstream**

- `context/schemas/INDEX.md` → `context/schemas/INDEX.md`

### schemas/actor

**new-upstream**

- `context/schemas/actor.yaml` → `context/schemas/actor.yaml`

### schemas/artifact

**new-upstream**

- `context/schemas/artifact.yaml` → `context/schemas/artifact.yaml`

### schemas/binding

**new-upstream**

- `context/schemas/binding.yaml` → `context/schemas/binding.yaml`

### schemas/category

**new-upstream**

- `context/schemas/category.yaml` → `context/schemas/category.yaml`

### schemas/constraint

**new-upstream**

- `context/schemas/constraint.yaml` → `context/schemas/constraint.yaml`

### schemas/context

**new-upstream**

- `context/schemas/context.yaml` → `context/schemas/context.yaml`

### schemas/decisionrecord

**new-upstream**

- `context/schemas/decisionrecord.yaml` → `context/schemas/decisionrecord.yaml`

### schemas/discussion

**new-upstream**

- `context/schemas/discussion.yaml` → `context/schemas/discussion.yaml`

### schemas/gate

**new-upstream**

- `context/schemas/gate.yaml` → `context/schemas/gate.yaml`

### schemas/logentry

**new-upstream**

- `context/schemas/logentry.yaml` → `context/schemas/logentry.yaml`

### schemas/migration

**new-upstream**

- `context/schemas/migration.yaml` → `context/schemas/migration.yaml`

### schemas/note

**new-upstream**

- `context/schemas/note.yaml` → `context/schemas/note.yaml`

### schemas/role

**new-upstream**

- `context/schemas/role.yaml` → `context/schemas/role.yaml`

### schemas/roleslot

**new-upstream**

- `context/schemas/roleslot.yaml` → `context/schemas/roleslot.yaml`

### schemas/scope

**new-upstream**

- `context/schemas/scope.yaml` → `context/schemas/scope.yaml`

### schemas/team-member

**new-upstream**

- `context/schemas/team-member.yaml` → `context/schemas/team-member.yaml`

### schemas/workitem

**new-upstream**

- `context/schemas/workitem.yaml` → `context/schemas/workitem.yaml`

### skills/INDEX.md

**new-upstream**

- `context/skills/INDEX.md` → `context/skills/INDEX.md`

### skills/data-ai

**new-upstream**

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

**new-upstream**

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

**new-upstream**

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
- `context/skills/devops/repo-management/mcp/server.py` → `context/skills/devops/repo-management/mcp/server.py`
- `context/skills/devops/repo-management/mcp/SERVER.md` → `context/skills/devops/repo-management/mcp/SERVER.md`
- `context/skills/devops/repo-management/mcp/mcp-config.json` → `context/skills/devops/repo-management/mcp/mcp-config.json`
- `context/skills/devops/repo-management/scripts/test_repo_management.py` → `context/skills/devops/repo-management/scripts/test_repo_management.py`
- `context/skills/devops/repo-management/SKILL.md` → `context/skills/devops/repo-management/SKILL.md`
- `context/skills/devops/repo-management/commands/pk-repo-reconcile.md` → `context/skills/devops/repo-management/commands/pk-repo-reconcile.md`
- `context/skills/devops/distributed-tracing/SKILL.md` → `context/skills/devops/distributed-tracing/SKILL.md`
- `context/skills/devops/ci-cd-setup/SKILL.md` → `context/skills/devops/ci-cd-setup/SKILL.md`
- `context/skills/devops/release-semver/SKILL.md` → `context/skills/devops/release-semver/SKILL.md`
- `context/skills/devops/release-semver/commands/pk-release.md` → `context/skills/devops/release-semver/commands/pk-release.md`
- `context/skills/devops/release-semver/commands/pk-publish.md` → `context/skills/devops/release-semver/commands/pk-publish.md`
- `context/skills/devops/alerting-oncall/SKILL.md` → `context/skills/devops/alerting-oncall/SKILL.md`

### skills/processkit

**new-upstream**

- `context/skills/processkit/eval-gate-authoring/mcp/server.py` → `context/skills/processkit/eval-gate-authoring/mcp/server.py`
- `context/skills/processkit/eval-gate-authoring/mcp/SERVER.md` → `context/skills/processkit/eval-gate-authoring/mcp/SERVER.md`
- `context/skills/processkit/eval-gate-authoring/mcp/mcp-config.json` → `context/skills/processkit/eval-gate-authoring/mcp/mcp-config.json`
- `context/skills/processkit/eval-gate-authoring/scripts/test_eval_gate_authoring.py` → `context/skills/processkit/eval-gate-authoring/scripts/test_eval_gate_authoring.py`
- `context/skills/processkit/eval-gate-authoring/SKILL.md` → `context/skills/processkit/eval-gate-authoring/SKILL.md`
- `context/skills/processkit/owner-profiling/references/observable-signals.md` → `context/skills/processkit/owner-profiling/references/observable-signals.md`
- `context/skills/processkit/owner-profiling/references/interview-protocol.md` → `context/skills/processkit/owner-profiling/references/interview-protocol.md`
- `context/skills/processkit/owner-profiling/SKILL.md` → `context/skills/processkit/owner-profiling/SKILL.md`
- `context/skills/processkit/owner-profiling/commands/pk-owner-bootstrap.md` → `context/skills/processkit/owner-profiling/commands/pk-owner-bootstrap.md`
- `context/skills/processkit/owner-profiling/commands/pk-observe.md` → `context/skills/processkit/owner-profiling/commands/pk-observe.md`
- `context/skills/processkit/owner-profiling/assets/OWNER-20260409_2054-TeamAnd-relationships.md` → `context/skills/processkit/owner-profiling/assets/OWNER-20260409_2054-TeamAnd-relationships.md`
- `context/skills/processkit/owner-profiling/assets/identity.md` → `context/skills/processkit/owner-profiling/assets/identity.md`
- `context/skills/processkit/owner-profiling/assets/OWNER-20260409_2054-GoalsAnd-context.md` → `context/skills/processkit/owner-profiling/assets/OWNER-20260409_2054-GoalsAnd-context.md`
- `context/skills/processkit/owner-profiling/assets/goals-and-context.md` → `context/skills/processkit/owner-profiling/assets/goals-and-context.md`
- `context/skills/processkit/owner-profiling/assets/OWNER-20260409_2054-WorkingStyle.md` → `context/skills/processkit/owner-profiling/assets/OWNER-20260409_2054-WorkingStyle.md`
- `context/skills/processkit/owner-profiling/assets/working-style.md` → `context/skills/processkit/owner-profiling/assets/working-style.md`
- `context/skills/processkit/owner-profiling/assets/team-and-relationships.md` → `context/skills/processkit/owner-profiling/assets/team-and-relationships.md`
- `context/skills/processkit/note-management/mcp/server.py` → `context/skills/processkit/note-management/mcp/server.py`
- `context/skills/processkit/note-management/mcp/SERVER.md` → `context/skills/processkit/note-management/mcp/SERVER.md`
- `context/skills/processkit/note-management/mcp/mcp-config.json` → `context/skills/processkit/note-management/mcp/mcp-config.json`
- `context/skills/processkit/note-management/SKILL.md` → `context/skills/processkit/note-management/SKILL.md`
- `context/skills/processkit/note-management/commands/pk-note.md` → `context/skills/processkit/note-management/commands/pk-note.md`
- `context/skills/processkit/note-management/commands/pk-note-review.md` → `context/skills/processkit/note-management/commands/pk-note-review.md`
- `context/skills/processkit/note-management/commands/pk-note-promote.md` → `context/skills/processkit/note-management/commands/pk-note-promote.md`
- `context/skills/processkit/aggregate-mcp/mcp/server.py` → `context/skills/processkit/aggregate-mcp/mcp/server.py`
- `context/skills/processkit/aggregate-mcp/mcp/mcp-config.aggregate.json` → `context/skills/processkit/aggregate-mcp/mcp/mcp-config.aggregate.json`
- `context/skills/processkit/aggregate-mcp/scripts/test_aggregate_mcp.py` → `context/skills/processkit/aggregate-mcp/scripts/test_aggregate_mcp.py`
- `context/skills/processkit/aggregate-mcp/SKILL.md` → `context/skills/processkit/aggregate-mcp/SKILL.md`
- `context/skills/processkit/binding-management/mcp/server.py` → `context/skills/processkit/binding-management/mcp/server.py`
- `context/skills/processkit/binding-management/mcp/SERVER.md` → `context/skills/processkit/binding-management/mcp/SERVER.md`
- `context/skills/processkit/binding-management/mcp/mcp-config.json` → `context/skills/processkit/binding-management/mcp/mcp-config.json`
- `context/skills/processkit/binding-management/scripts/test_binding_management.py` → `context/skills/processkit/binding-management/scripts/test_binding_management.py`
- `context/skills/processkit/binding-management/SKILL.md` → `context/skills/processkit/binding-management/SKILL.md`
- `context/skills/processkit/binding-management/assets/binding.yaml` → `context/skills/processkit/binding-management/assets/binding.yaml`
- `context/skills/processkit/index-management/config/settings.toml` → `context/skills/processkit/index-management/config/settings.toml`
- `context/skills/processkit/index-management/mcp/server.py` → `context/skills/processkit/index-management/mcp/server.py`
- `context/skills/processkit/index-management/mcp/SERVER.md` → `context/skills/processkit/index-management/mcp/SERVER.md`
- `context/skills/processkit/index-management/mcp/mcp-config.json` → `context/skills/processkit/index-management/mcp/mcp-config.json`
- `context/skills/processkit/index-management/scripts/test_get_entity_by_path_and_list_entities.py` → `context/skills/processkit/index-management/scripts/test_get_entity_by_path_and_list_entities.py`
- `context/skills/processkit/index-management/scripts/test_db_result_limits.py` → `context/skills/processkit/index-management/scripts/test_db_result_limits.py`
- `context/skills/processkit/index-management/scripts/test_index_management_v1_penalty.py` → `context/skills/processkit/index-management/scripts/test_index_management_v1_penalty.py`
- `context/skills/processkit/index-management/SKILL.md` → `context/skills/processkit/index-management/SKILL.md`
- `context/skills/processkit/agent-management/references/coordination-patterns.md` → `context/skills/processkit/agent-management/references/coordination-patterns.md`
- `context/skills/processkit/agent-management/SKILL.md` → `context/skills/processkit/agent-management/SKILL.md`
- `context/skills/processkit/event-log/mcp/server.py` → `context/skills/processkit/event-log/mcp/server.py`
- `context/skills/processkit/event-log/mcp/SERVER.md` → `context/skills/processkit/event-log/mcp/SERVER.md`
- `context/skills/processkit/event-log/mcp/mcp-config.json` → `context/skills/processkit/event-log/mcp/mcp-config.json`
- `context/skills/processkit/event-log/examples/workitem-transitioned.md` → `context/skills/processkit/event-log/examples/workitem-transitioned.md`
- `context/skills/processkit/event-log/scripts/test_log_event.py` → `context/skills/processkit/event-log/scripts/test_log_event.py`
- `context/skills/processkit/event-log/SKILL.md` → `context/skills/processkit/event-log/SKILL.md`
- `context/skills/processkit/event-log/assets/logentry.yaml` → `context/skills/processkit/event-log/assets/logentry.yaml`
- `context/skills/processkit/artifact-management/mcp/server.py` → `context/skills/processkit/artifact-management/mcp/server.py`
- `context/skills/processkit/artifact-management/mcp/SERVER.md` → `context/skills/processkit/artifact-management/mcp/SERVER.md`
- `context/skills/processkit/artifact-management/mcp/mcp-config.json` → `context/skills/processkit/artifact-management/mcp/mcp-config.json`
- `context/skills/processkit/artifact-management/SKILL.md` → `context/skills/processkit/artifact-management/SKILL.md`
- `context/skills/processkit/process-management/SKILL.md` → `context/skills/processkit/process-management/SKILL.md`
- `context/skills/processkit/retrospective/scripts/signals/timeline.py` → `context/skills/processkit/retrospective/scripts/signals/timeline.py`
- `context/skills/processkit/retrospective/scripts/signals/drift.py` → `context/skills/processkit/retrospective/scripts/signals/drift.py`
- `context/skills/processkit/retrospective/scripts/signals/__init__.py` → `context/skills/processkit/retrospective/scripts/signals/__init__.py`
- `context/skills/processkit/retrospective/scripts/signals/workitems.py` → `context/skills/processkit/retrospective/scripts/signals/workitems.py`
- `context/skills/processkit/retrospective/scripts/signals/release_summary.py` → `context/skills/processkit/retrospective/scripts/signals/release_summary.py`
- `context/skills/processkit/retrospective/scripts/pk_retro.py` → `context/skills/processkit/retrospective/scripts/pk_retro.py`
