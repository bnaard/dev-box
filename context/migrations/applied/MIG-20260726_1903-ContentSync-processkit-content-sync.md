---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260726_1903-ContentSync-processkit-content-sync
  created: 2026-07-26 19:03:38+00:00
  updated: '2026-07-26T19:16:52+00:00'
spec:
  source: processkit
  source_url: https://github.com/projectious-work/processkit.git
  from_version: v0.28.3
  to_version: v0.28.4
  state: applied
  generated_by: aibox apply
  generated_at: 2026-07-26 19:03:38+00:00
  summary: 0 changed upstream, 0 conflicts, 722 new, 0 removed, 0 stale-removed (44
    groups affected)
  affected_groups:
  - ''
  - AGENTS
  - context
  - context/artifacts
  - context/bindings
  - context/roles
  - context/team
  - context/team-members/cora
  - context/team-members/cora/relations
  - context/team-members/thrifty-otter
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
  - skills/documents
  - skills/engineering
  - skills/processkit
  - skills/product
  - state-machines/INDEX
  - state-machines/decisionrecord
  - state-machines/discussion
  - state-machines/migration
  - state-machines/note
  - state-machines/scope
  - state-machines/workitem
  affected_files:
  - path: AGENTS.md
    classification: new-upstream
  - path: INDEX.md
    classification: new-upstream
  - path: context/.processkit-mcp-manifest.json
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen2-5-72b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen2-5-coder-32b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-235b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-flash.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-max-preview.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-plus.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-7-max.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-8-max-preview.md
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
  - path: context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-6-flash.md
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
  - path: context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-m3.md
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
  - path: context/artifacts/ART-20260503_1424-ModelSpec-moonshot-kimi-k3.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-nvidia-nvidia-nemotron-3-super-120b.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-4o.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-2-codex.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-2-pro.md
    classification: new-upstream
  - path: context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-3-codex-spark.md
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
  - path: context/artifacts/ART-20260503_1424-ModelSpec-subquadratic-subq-1-1-small.md
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
  - path: context/artifacts/ART-20260503_1424-ModelSpec-xiaomi-mimo-7b.md
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
  - path: context/bindings/BIND-ai-research-scientist-expert-r1-h3ea4c0.md
    classification: new-upstream
  - path: context/bindings/BIND-ai-research-scientist-junior-ha25e5b.md
    classification: new-upstream
  - path: context/bindings/BIND-ai-research-scientist-principal-h5e96d8.md
    classification: new-upstream
  - path: context/bindings/BIND-ai-research-scientist-senior-h18c312.md
    classification: new-upstream
  - path: context/bindings/BIND-ai-research-scientist-specialist-r1-h52964c.md
    classification: new-upstream
  - path: context/bindings/BIND-assistant-expert-r1-h67ab7c.md
    classification: new-upstream
  - path: context/bindings/BIND-assistant-junior-hae1bfb.md
    classification: new-upstream
  - path: context/bindings/BIND-assistant-principal-hb5ac7b.md
    classification: new-upstream
  - path: context/bindings/BIND-assistant-senior-h771629.md
    classification: new-upstream
  - path: context/bindings/BIND-assistant-specialist-r1-he4f117.md
    classification: new-upstream
  - path: context/bindings/BIND-cora-default-r1-h299cb7.md
    classification: new-upstream
  - path: context/bindings/BIND-data-scientist-expert-r1-hb8654f.md
    classification: new-upstream
  - path: context/bindings/BIND-data-scientist-junior-he28d96.md
    classification: new-upstream
  - path: context/bindings/BIND-data-scientist-principal-h192434.md
    classification: new-upstream
  - path: context/bindings/BIND-data-scientist-senior-h8816d2.md
    classification: new-upstream
  - path: context/bindings/BIND-data-scientist-specialist-r1-h5f7da2.md
    classification: new-upstream
  - path: context/bindings/BIND-database-engineer-senior-r1-h39bdeb.md
    classification: new-upstream
  - path: context/bindings/BIND-machine-learning-engineer-senior-r1-hd54ded.md
    classification: new-upstream
  - path: context/bindings/BIND-product-manager-expert-r1-hf330ef.md
    classification: new-upstream
  - path: context/bindings/BIND-product-manager-junior-h9275a0.md
    classification: new-upstream
  - path: context/bindings/BIND-product-manager-principal-h723e55.md
    classification: new-upstream
  - path: context/bindings/BIND-product-manager-senior-hf3f8e9.md
    classification: new-upstream
  - path: context/bindings/BIND-product-manager-senior-r1-hf96017.md
    classification: new-upstream
  - path: context/bindings/BIND-product-manager-specialist-r1-h42cae9.md
    classification: new-upstream
  - path: context/bindings/BIND-qa-engineer-expert-r1-h68a730.md
    classification: new-upstream
  - path: context/bindings/BIND-qa-engineer-junior-h796cc2.md
    classification: new-upstream
  - path: context/bindings/BIND-qa-engineer-principal-h07f4cd.md
    classification: new-upstream
  - path: context/bindings/BIND-qa-engineer-senior-h0436ab.md
    classification: new-upstream
  - path: context/bindings/BIND-qa-engineer-senior-r1-h4db0e9.md
    classification: new-upstream
  - path: context/bindings/BIND-qa-engineer-senior-r1-h522915.md
    classification: new-upstream
  - path: context/bindings/BIND-qa-engineer-specialist-r1-he98ba3.md
    classification: new-upstream
  - path: context/bindings/BIND-research-scientist-expert-r1-h3e65e9.md
    classification: new-upstream
  - path: context/bindings/BIND-research-scientist-junior-h87372f.md
    classification: new-upstream
  - path: context/bindings/BIND-research-scientist-principal-hb4e3bb.md
    classification: new-upstream
  - path: context/bindings/BIND-research-scientist-senior-h14c6d3.md
    classification: new-upstream
  - path: context/bindings/BIND-research-scientist-specialist-r1-hf66b32.md
    classification: new-upstream
  - path: context/bindings/BIND-security-architect-expert-r1-h636811.md
    classification: new-upstream
  - path: context/bindings/BIND-security-architect-junior-hf79510.md
    classification: new-upstream
  - path: context/bindings/BIND-security-architect-principal-h2c8917.md
    classification: new-upstream
  - path: context/bindings/BIND-security-architect-senior-h142586.md
    classification: new-upstream
  - path: context/bindings/BIND-security-architect-specialist-r1-hc3ae4a.md
    classification: new-upstream
  - path: context/bindings/BIND-software-engineer-expert-r1-ha8eea0.md
    classification: new-upstream
  - path: context/bindings/BIND-software-engineer-junior-h92feb2.md
    classification: new-upstream
  - path: context/bindings/BIND-software-engineer-principal-ha71725.md
    classification: new-upstream
  - path: context/bindings/BIND-software-engineer-senior-h7bd319.md
    classification: new-upstream
  - path: context/bindings/BIND-software-engineer-senior-r1-h79acd6.md
    classification: new-upstream
  - path: context/bindings/BIND-software-engineer-senior-r1-h8ea632.md
    classification: new-upstream
  - path: context/bindings/BIND-software-engineer-specialist-r1-h5a5377.md
    classification: new-upstream
  - path: context/bindings/BIND-solutions-architect-expert-r1-h850329.md
    classification: new-upstream
  - path: context/bindings/BIND-solutions-architect-junior-heb66a8.md
    classification: new-upstream
  - path: context/bindings/BIND-solutions-architect-principal-h5264ce.md
    classification: new-upstream
  - path: context/bindings/BIND-solutions-architect-senior-h603b59.md
    classification: new-upstream
  - path: context/bindings/BIND-solutions-architect-specialist-r1-h1125a0.md
    classification: new-upstream
  - path: context/bindings/BIND-technical-writer-expert-r1-h103b62.md
    classification: new-upstream
  - path: context/bindings/BIND-technical-writer-junior-h431a38.md
    classification: new-upstream
  - path: context/bindings/BIND-technical-writer-principal-h56b361.md
    classification: new-upstream
  - path: context/bindings/BIND-technical-writer-senior-hb93488.md
    classification: new-upstream
  - path: context/bindings/BIND-technical-writer-senior-r1-h63fa44.md
    classification: new-upstream
  - path: context/bindings/BIND-technical-writer-senior-r1-h93e646.md
    classification: new-upstream
  - path: context/bindings/BIND-technical-writer-specialist-r1-h920232.md
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
  - path: context/skills/documents/data-storytelling/SKILL.md
    classification: new-upstream
  - path: context/skills/documents/data-visualization/SKILL.md
    classification: new-upstream
  - path: context/skills/documents/data-visualization/references/chart-selection.md
    classification: new-upstream
  - path: context/skills/documents/docx-authoring/SKILL.md
    classification: new-upstream
  - path: context/skills/documents/infographics/SKILL.md
    classification: new-upstream
  - path: context/skills/documents/infographics/references/best-practices.md
    classification: new-upstream
  - path: context/skills/documents/latex-authoring/SKILL.md
    classification: new-upstream
  - path: context/skills/documents/latex-authoring/references/math-reference.md
    classification: new-upstream
  - path: context/skills/documents/latex-authoring/references/packages.md
    classification: new-upstream
  - path: context/skills/documents/latex-authoring/references/tikz-reference.md
    classification: new-upstream
  - path: context/skills/documents/pdf-workflow/SKILL.md
    classification: new-upstream
  - path: context/skills/documents/pptx-authoring/SKILL.md
    classification: new-upstream
  - path: context/skills/documents/xlsx-modeling/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/api-design/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/api-design/references/openapi-patterns.md
    classification: new-upstream
  - path: context/skills/engineering/api-design/references/rest-conventions.md
    classification: new-upstream
  - path: context/skills/engineering/auth-patterns/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/auth-patterns/references/jwt-reference.md
    classification: new-upstream
  - path: context/skills/engineering/auth-patterns/references/oauth-flows.md
    classification: new-upstream
  - path: context/skills/engineering/caching-strategies/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/changelog/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/code-generation/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/code-review/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/concurrency-patterns/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/concurrency-patterns/references/patterns-catalog.md
    classification: new-upstream
  - path: context/skills/engineering/database-migration/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/database-modeling/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/database-modeling/references/modeling-patterns.md
    classification: new-upstream
  - path: context/skills/engineering/debugging/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/dependency-audit/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/dependency-management/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/domain-driven-design/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/domain-driven-design/references/ddd-building-blocks.md
    classification: new-upstream
  - path: context/skills/engineering/error-handling/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/event-driven-architecture/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/event-driven-architecture/references/messaging-patterns.md
    classification: new-upstream
  - path: context/skills/engineering/fastapi-patterns/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/fastapi-patterns/references/endpoint-patterns.md
    classification: new-upstream
  - path: context/skills/engineering/flutter-development/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/flutter-development/references/widget-catalog.md
    classification: new-upstream
  - path: context/skills/engineering/git-branching/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/git-branching/references/strategies.md
    classification: new-upstream
  - path: context/skills/engineering/git-workflow/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/go-conventions/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/go-conventions/references/go-patterns.md
    classification: new-upstream
  - path: context/skills/engineering/graphql-patterns/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/grpc-protobuf/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/grpc-protobuf/references/proto-conventions.md
    classification: new-upstream
  - path: context/skills/engineering/integration-testing/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/integration-testing/references/test-fixtures.md
    classification: new-upstream
  - path: context/skills/engineering/java-patterns/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/load-testing/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/microservice-creation/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/nosql-patterns/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/performance-profiling/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/performance-profiling/references/profiling-tools.md
    classification: new-upstream
  - path: context/skills/engineering/pixijs-gamedev/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/pixijs-gamedev/references/api-cheatsheet.md
    classification: new-upstream
  - path: context/skills/engineering/python-best-practices/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/refactoring/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/refactoring/references/code-smells.md
    classification: new-upstream
  - path: context/skills/engineering/refactoring/references/gof-patterns.md
    classification: new-upstream
  - path: context/skills/engineering/reflex-python/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/reflex-python/references/component-reference.md
    classification: new-upstream
  - path: context/skills/engineering/rust-conventions/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/secret-management/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/secure-coding/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/secure-coding/references/owasp-checklist.md
    classification: new-upstream
  - path: context/skills/engineering/shell-scripting/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/shell-scripting/references/bash-patterns.md
    classification: new-upstream
  - path: context/skills/engineering/software-architecture/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/software-architecture/references/patterns.md
    classification: new-upstream
  - path: context/skills/engineering/software-modularization/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/sql-patterns/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/sql-patterns/references/query-patterns.md
    classification: new-upstream
  - path: context/skills/engineering/sql-patterns/references/schema-design.md
    classification: new-upstream
  - path: context/skills/engineering/sql-style-guide/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/system-design/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/system-design/references/estimation-cheatsheet.md
    classification: new-upstream
  - path: context/skills/engineering/tailwind/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/tailwind/references/cheatsheet.md
    classification: new-upstream
  - path: context/skills/engineering/tdd-workflow/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/testing-strategy/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/threat-modeling/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/typescript-patterns/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/webhook-integration/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/actor-profile/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/actor-profile/assets/actor-agent.yaml
    classification: new-upstream
  - path: context/skills/processkit/actor-profile/assets/actor-human.yaml
    classification: new-upstream
  - path: context/skills/processkit/actor-profile/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/actor-profile/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/actor-profile/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/agent-card/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/agent-card/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/agent-card/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/agent-card/mcp/server.py
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
  - path: context/skills/processkit/category-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/category-management/assets/category.yaml
    classification: new-upstream
  - path: context/skills/processkit/constraint-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/constraint-management/assets/constraint.yaml
    classification: new-upstream
  - path: context/skills/processkit/context-archiving/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/context-archiving/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/context-archiving/scripts/test_context_archiving.py
    classification: new-upstream
  - path: context/skills/processkit/context-grooming/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/context-grooming/assets/grooming-report.md
    classification: new-upstream
  - path: context/skills/processkit/context-grooming/commands/pk-groom.md
    classification: new-upstream
  - path: context/skills/processkit/cross-reference-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/decision-record/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/decision-record/assets/decisionrecord.yaml
    classification: new-upstream
  - path: context/skills/processkit/decision-record/commands/pk-dec-find.md
    classification: new-upstream
  - path: context/skills/processkit/decision-record/commands/pk-dec.md
    classification: new-upstream
  - path: context/skills/processkit/decision-record/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/decision-record/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/decision-record/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/discussion-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/discussion-management/assets/discussion.yaml
    classification: new-upstream
  - path: context/skills/processkit/discussion-management/commands/pk-discuss.md
    classification: new-upstream
  - path: context/skills/processkit/discussion-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/discussion-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/discussion-management/mcp/server.py
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
  - path: context/skills/processkit/gate-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/gate-management/assets/gate.yaml
    classification: new-upstream
  - path: context/skills/processkit/gate-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/gate-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/gate-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/gate-management/scripts/test_gate_management.py
    classification: new-upstream
  - path: context/skills/processkit/id-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/id-management/config/settings.toml
    classification: new-upstream
  - path: context/skills/processkit/id-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/id-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/id-management/mcp/server.py
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
  - path: context/skills/processkit/migration-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/migration-management/assets/migration.yaml
    classification: new-upstream
  - path: context/skills/processkit/migration-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/migration-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/migration-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/migration-management/scripts/test_migration_management.py
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/commands/pk-explain-routing.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/commands/pk-model-refresh.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/commands/pk-model-setup.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/commands/pk-route.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/examples/profile-view.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/examples/task-routing.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/mcp/model_scores.json
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/mcp/user_config.json
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/references/dimension-specs.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/references/model-characteristics.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/references/model-profiles.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/references/roster-quick-ref.md
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/scripts/migrate_model_profiles.py
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/scripts/migrate_models.py
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/scripts/resolver.py
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/scripts/test_default_bindings_coverage.py
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/scripts/test_migrate_models.py
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/scripts/test_query_models_filters.py
    classification: new-upstream
  - path: context/skills/processkit/model-recommender/scripts/test_resolver.py
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
  - path: context/skills/processkit/pk-doctor/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/commands/pk-doctor.md
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/__init__.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/agents_md_hygiene.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/commands_consistency.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/common.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/context_consumption.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/context_hygiene.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/doctor_boundary.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/drift.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/entity_storage_hygiene.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/id_vocabulary.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/mcp_config_drift.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/mcp_gateway.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/migration_integrity.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/migrations.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/preauth_applied.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/release_integrity.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/runtime_health.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/schema_filename.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/schema_vocabulary.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/sensitive_data.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/server_header_drift.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/sharding.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/skill_dag.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/supply_chain.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/team_consistency.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/team_member_exports.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/v1_entity_drift.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/checks/v2_contracts.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/doctor.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/test_doctor.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/test_id_vocabulary.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/test_pk_doctor_json.py
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/test_pk_doctor_mcp.py
    classification: new-upstream
  - path: context/skills/processkit/process-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/processkit-gateway/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/processkit-gateway/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/processkit-gateway/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/processkit-gateway/mcp/tool-catalog.json
    classification: new-upstream
  - path: context/skills/processkit/processkit-gateway/scripts/test_daemon_runtime.py
    classification: new-upstream
  - path: context/skills/processkit/processkit-gateway/scripts/test_gateway.py
    classification: new-upstream
  - path: context/skills/processkit/processkit-gateway/scripts/test_proxy.py
    classification: new-upstream
  - path: context/skills/processkit/project-reconciliation/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/project-reconciliation/commands/pk-reconcile.md
    classification: new-upstream
  - path: context/skills/processkit/release-audit/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/release-audit/commands/pk-release-audit.md
    classification: new-upstream
  - path: context/skills/processkit/release-audit/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/release-audit/scripts/release_audit.py
    classification: new-upstream
  - path: context/skills/processkit/release-audit/scripts/test_release_audit.py
    classification: new-upstream
  - path: context/skills/processkit/release-audit/scripts/test_release_audit_json.py
    classification: new-upstream
  - path: context/skills/processkit/repository-portfolio-review/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/retrospective/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/retrospective/commands/pk-retro.md
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
  - path: context/skills/processkit/retrospective/scripts/test_retro.py
    classification: new-upstream
  - path: context/skills/processkit/role-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/role-management/assets/role.yaml
    classification: new-upstream
  - path: context/skills/processkit/role-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/role-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/role-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/runtime-prune/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/runtime-prune/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/runtime-prune/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/runtime-prune/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/runtime-prune/scripts/test_runtime_prune.py
    classification: new-upstream
  - path: context/skills/processkit/schedule-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/scope-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/scope-management/assets/scope.yaml
    classification: new-upstream
  - path: context/skills/processkit/scope-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/scope-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/scope-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/security-projections/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/security-projections/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/security-projections/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/security-projections/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/session-handover/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/session-handover/commands/pk-wrapup.md
    classification: new-upstream
  - path: context/skills/processkit/skill-builder/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/skill-builder/assets/skill-template.md
    classification: new-upstream
  - path: context/skills/processkit/skill-builder/commands/pk-skill-new.md
    classification: new-upstream
  - path: context/skills/processkit/skill-builder/references/library-expert-template.md
    classification: new-upstream
  - path: context/skills/processkit/skill-finder/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/skill-finder/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/skill-finder/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/skill-finder/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/skill-finder/scripts/test_catalog.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/assets/compliance-contract.md
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/assets/preauth.json
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/commands/pk-build.md
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/commands/pk-lint.md
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/commands/pk-review.md
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/commands/pk-test.md
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/README.md
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/check_decision_captured.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/check_entity_read.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/check_route_task_before_agent.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/check_route_task_called.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/decision_markers.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/decision_sweeper.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/emit_compliance_contract.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/fixtures/claude-code-pre-tool-use.json
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/fixtures/claude-code-pretooluse-sample.json
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/fixtures/claude-code-pretooluse-with-transcript.json
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/fixtures/claude-code-session-start.json
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/fixtures/claude-code-sessionend.json
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/fixtures/sample-transcript-no-decisions.jsonl
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/fixtures/sample-transcript-with-decisions.jsonl
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/fixtures/sample-transcript-with-poison-entries.jsonl
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/record_decision_observer.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/test_entity_read_and_agent_hooks.py
    classification: new-upstream
  - path: context/skills/processkit/skill-gate/scripts/test_hooks.py
    classification: new-upstream
  - path: context/skills/processkit/skill-reviewer/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/skill-reviewer/commands/pk-skill-audit.md
    classification: new-upstream
  - path: context/skills/processkit/standup-context/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/standup-context/commands/pk-standup.md
    classification: new-upstream
  - path: context/skills/processkit/standup-context/commands/pk-status.md
    classification: new-upstream
  - path: context/skills/processkit/state-machine-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/status-briefing/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/status-briefing/commands/pk-resume.md
    classification: new-upstream
  - path: context/skills/processkit/status-update-writer/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/supply-chain-audit/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/supply-chain-audit/commands/pk-supply-chain.md
    classification: new-upstream
  - path: context/skills/processkit/supply-chain-audit/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/supply-chain-audit/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/supply-chain-audit/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/supply-chain-audit/scripts/supply_chain_audit.py
    classification: new-upstream
  - path: context/skills/processkit/supply-chain-audit/scripts/test_supply_chain_audit.py
    classification: new-upstream
  - path: context/skills/processkit/task-router/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/task-router/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/task-router/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/task-router/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/task-router/mcp/user_config.json
    classification: new-upstream
  - path: context/skills/processkit/task-router/scripts/test_task_router.py
    classification: new-upstream
  - path: context/skills/processkit/team-creator/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/team-creator/assets/archetype-catalog-mapping.yaml
    classification: new-upstream
  - path: context/skills/processkit/team-creator/commands/pk-team-create.md
    classification: new-upstream
  - path: context/skills/processkit/team-creator/commands/pk-team-rebalance.md
    classification: new-upstream
  - path: context/skills/processkit/team-creator/commands/pk-team-review.md
    classification: new-upstream
  - path: context/skills/processkit/team-creator/references/landscape-resolution.md
    classification: new-upstream
  - path: context/skills/processkit/team-creator/references/role-archetypes-override.md
    classification: new-upstream
  - path: context/skills/processkit/team-creator/references/role-archetypes.md
    classification: new-upstream
  - path: context/skills/processkit/team-creator/references/team-weights-decision-schema.md
    classification: new-upstream
  - path: context/skills/processkit/team-creator/references/tiering-formula.md
    classification: new-upstream
  - path: context/skills/processkit/team-creator/scripts/team_creator_lib.py
    classification: new-upstream
  - path: context/skills/processkit/team-manager/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/team-manager/assets/agent-card.schema.json
    classification: new-upstream
  - path: context/skills/processkit/team-manager/assets/memory-file-header.schema.json
    classification: new-upstream
  - path: context/skills/processkit/team-manager/assets/team-member-ai-agent.yaml
    classification: new-upstream
  - path: context/skills/processkit/team-manager/assets/team-member-human.yaml
    classification: new-upstream
  - path: context/skills/processkit/team-manager/data/name-pool.yaml
    classification: new-upstream
  - path: context/skills/processkit/team-manager/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/team-manager/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/team-manager/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/team-manager/scripts/apply_migration_2139.py
    classification: new-upstream
  - path: context/skills/processkit/team-manager/scripts/consistency.py
    classification: new-upstream
  - path: context/skills/processkit/team-manager/scripts/export_import.py
    classification: new-upstream
  - path: context/skills/processkit/team-manager/scripts/memory_tree.py
    classification: new-upstream
  - path: context/skills/processkit/team-manager/scripts/name_pool.py
    classification: new-upstream
  - path: context/skills/processkit/team-manager/scripts/test_team_manager.py
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/assets/workitem-bug.yaml
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/assets/workitem-story.yaml
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/assets/workitem.yaml
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/commands/pk-work.md
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/examples/create-feature.md
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/mcp/SERVER.md
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/mcp/mcp-config.json
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/mcp/server.py
    classification: new-upstream
  - path: context/skills/processkit/workitem-management/scripts/test_workitem_management.py
    classification: new-upstream
  - path: context/skills/product/board-of-advisors/SKILL.md
    classification: new-upstream
  - path: context/skills/product/devils-advocate/SKILL.md
    classification: new-upstream
  - path: context/skills/product/documentation/SKILL.md
    classification: new-upstream
  - path: context/skills/product/email-drafter/SKILL.md
    classification: new-upstream
  - path: context/skills/product/estimation-planning/SKILL.md
    classification: new-upstream
  - path: context/skills/product/legal-review/SKILL.md
    classification: new-upstream
  - path: context/skills/product/onboarding-guide/SKILL.md
    classification: new-upstream
  - path: context/skills/product/prd-writing/SKILL.md
    classification: new-upstream
  - path: context/skills/product/research-with-confidence/SKILL.md
    classification: new-upstream
  - path: context/skills/product/research-with-confidence/commands/pk-research.md
    classification: new-upstream
  - path: context/skills/product/sprint-retrospective/SKILL.md
    classification: new-upstream
  - path: context/skills/product/user-research/SKILL.md
    classification: new-upstream
  - path: context/state-machines/INDEX.md
    classification: new-upstream
  - path: context/state-machines/decisionrecord.yaml
    classification: new-upstream
  - path: context/state-machines/discussion.yaml
    classification: new-upstream
  - path: context/state-machines/migration.yaml
    classification: new-upstream
  - path: context/state-machines/note.yaml
    classification: new-upstream
  - path: context/state-machines/scope.yaml
    classification: new-upstream
  - path: context/state-machines/workitem.yaml
    classification: new-upstream
  - path: context/team-members/cora/card.json
    classification: new-upstream
  - path: context/team-members/cora/persona.md
    classification: new-upstream
  - path: context/team-members/cora/relations/thrifty-otter.md
    classification: new-upstream
  - path: context/team-members/cora/team-member.md
    classification: new-upstream
  - path: context/team-members/thrifty-otter/card.json
    classification: new-upstream
  - path: context/team-members/thrifty-otter/persona.md
    classification: new-upstream
  - path: context/team-members/thrifty-otter/team-member.md
    classification: new-upstream
  - path: context/team/roster.md
    classification: new-upstream
  started_at: '2026-07-26T19:16:51+00:00'
  applied_at: '2026-07-26T19:16:52+00:00'
  progress_notes:
  - timestamp: '2026-07-26T19:16:52+00:00'
    actor: mcp
    note: 'Applied during pk-reconcile session-start: pure additive content sync (0
      conflicts, 722 new, 0 removed) — unambiguous.'
---

# Migration MIG-20260726_1903-ContentSync-processkit-content-sync

From `v0.28.3` to `v0.28.4` (source: `https://github.com/projectious-work/processkit.git`).

0 changed upstream, 0 conflicts, 722 new, 0 removed, 0 stale-removed (44 groups affected)

## Counts

- unchanged: 0
- changed-locally-only: 0
- changed-upstream-only: 0
- conflict: 0
- new-upstream: 722
- removed-upstream: 0
- removed-upstream-stale: 0

## Changes by group

### (ungrouped)

**new-upstream**

- `INDEX.md` → `context/INDEX.md`

### AGENTS

**new-upstream**

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
- `context/artifacts/ART-20260503_1424-ModelSpec-subquadratic-subq-1-1-small.md` → `context/artifacts/ART-20260503_1424-ModelSpec-subquadratic-subq-1-1-small.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-governed-deep.md` → `context/artifacts/ART-20260503_1832-ModelProfile-governed-deep.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-code-fast.md` → `context/artifacts/ART-20260503_1832-ModelProfile-code-fast.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-writing-balanced.md` → `context/artifacts/ART-20260503_1832-ModelProfile-writing-balanced.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-flash.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-flash.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-o3.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-o3.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-7-max.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-7-max.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-m.md` → `context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-m.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-general-deep.md` → `context/artifacts/ART-20260503_1832-ModelProfile-general-deep.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-nvidia-nvidia-nemotron-3-super-120b.md` → `context/artifacts/ART-20260503_1424-ModelSpec-nvidia-nvidia-nemotron-3-super-120b.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-code-balanced.md` → `context/artifacts/ART-20260503_1832-ModelProfile-code-balanced.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemma-3-27b.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemma-3-27b.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-research-deep.md` → `context/artifacts/ART-20260503_1832-ModelProfile-research-deep.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-r-plus.md` → `context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-r-plus.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-o4-mini.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-o4-mini.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-flash.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-flash.md`
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
- `context/artifacts/ART-20260503_1424-ModelSpec-moonshot-kimi-k3.md` → `context/artifacts/ART-20260503_1424-ModelSpec-moonshot-kimi-k3.md`
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
- `context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-m3.md` → `context/artifacts/ART-20260503_1424-ModelSpec-minimax-minimax-m3.md`
- `context/artifacts/ART-20260503_1832-ModelProfile-general-fast.md` → `context/artifacts/ART-20260503_1832-ModelProfile-general-fast.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-max-preview.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-6-max-preview.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-3-5.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-3-5.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xiaomi-mimo-7b.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xiaomi-mimo-7b.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-aws-amazon-nova-2-lite.md` → `context/artifacts/ART-20260503_1424-ModelSpec-aws-amazon-nova-2-lite.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-heavy.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4-heavy.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-a.md` → `context/artifacts/ART-20260503_1424-ModelSpec-cohere-command-a.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4.md` → `context/artifacts/ART-20260503_1424-ModelSpec-xai-grok-4.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-6-flash.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-6-flash.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-microsoft-phi.md` → `context/artifacts/ART-20260503_1424-ModelSpec-microsoft-phi.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-mistral-devstral.md` → `context/artifacts/ART-20260503_1424-ModelSpec-mistral-devstral.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v4-pro.md` → `context/artifacts/ART-20260503_1424-ModelSpec-deepseek-deepseek-v4-pro.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-moonshot-kimi-k2-6.md` → `context/artifacts/ART-20260503_1424-ModelSpec-moonshot-kimi-k2-6.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-medium.md` → `context/artifacts/ART-20260503_1424-ModelSpec-mistral-mistral-medium.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-3-codex-spark.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-3-codex-spark.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-4-scout.md` → `context/artifacts/ART-20260503_1424-ModelSpec-meta-llama-4-scout.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-pro-preview.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-pro-preview.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-8-max-preview.md` → `context/artifacts/ART-20260503_1424-ModelSpec-alibaba-qwen3-8-max-preview.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-2-5-flash.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-2-5-flash.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-pro.md` → `context/artifacts/ART-20260503_1424-ModelSpec-openai-gpt-5-pro.md`
- `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-flash.md` → `context/artifacts/ART-20260503_1424-ModelSpec-google-gemini-3-flash.md`

### context/bindings

**new-upstream**

- `context/bindings/BIND-assistant-expert-r1-h67ab7c.md` → `context/bindings/BIND-assistant-expert-r1-h67ab7c.md`
- `context/bindings/BIND-solutions-architect-expert-r1-h850329.md` → `context/bindings/BIND-solutions-architect-expert-r1-h850329.md`
- `context/bindings/BIND-research-scientist-expert-r1-h3e65e9.md` → `context/bindings/BIND-research-scientist-expert-r1-h3e65e9.md`
- `context/bindings/BIND-solutions-architect-junior-heb66a8.md` → `context/bindings/BIND-solutions-architect-junior-heb66a8.md`
- `context/bindings/BIND-security-architect-specialist-r1-hc3ae4a.md` → `context/bindings/BIND-security-architect-specialist-r1-hc3ae4a.md`
- `context/bindings/BIND-data-scientist-junior-he28d96.md` → `context/bindings/BIND-data-scientist-junior-he28d96.md`
- `context/bindings/BIND-qa-engineer-expert-r1-h68a730.md` → `context/bindings/BIND-qa-engineer-expert-r1-h68a730.md`
- `context/bindings/BIND-solutions-architect-specialist-r1-h1125a0.md` → `context/bindings/BIND-solutions-architect-specialist-r1-h1125a0.md`
- `context/bindings/BIND-research-scientist-principal-hb4e3bb.md` → `context/bindings/BIND-research-scientist-principal-hb4e3bb.md`
- `context/bindings/BIND-qa-engineer-junior-h796cc2.md` → `context/bindings/BIND-qa-engineer-junior-h796cc2.md`
- `context/bindings/BIND-software-engineer-senior-r1-h8ea632.md` → `context/bindings/BIND-software-engineer-senior-r1-h8ea632.md`
- `context/bindings/BIND-technical-writer-expert-r1-h103b62.md` → `context/bindings/BIND-technical-writer-expert-r1-h103b62.md`
- `context/bindings/BIND-assistant-junior-hae1bfb.md` → `context/bindings/BIND-assistant-junior-hae1bfb.md`
- `context/bindings/BIND-cora-default-r1-h299cb7.md` → `context/bindings/BIND-cora-default-r1-h299cb7.md`
- `context/bindings/BIND-product-manager-senior-hf3f8e9.md` → `context/bindings/BIND-product-manager-senior-hf3f8e9.md`
- `context/bindings/BIND-research-scientist-specialist-r1-hf66b32.md` → `context/bindings/BIND-research-scientist-specialist-r1-hf66b32.md`
- `context/bindings/BIND-software-engineer-junior-h92feb2.md` → `context/bindings/BIND-software-engineer-junior-h92feb2.md`
- `context/bindings/BIND-database-engineer-senior-r1-h39bdeb.md` → `context/bindings/BIND-database-engineer-senior-r1-h39bdeb.md`
- `context/bindings/BIND-research-scientist-junior-h87372f.md` → `context/bindings/BIND-research-scientist-junior-h87372f.md`
- `context/bindings/BIND-technical-writer-junior-h431a38.md` → `context/bindings/BIND-technical-writer-junior-h431a38.md`
- `context/bindings/BIND-ai-research-scientist-principal-h5e96d8.md` → `context/bindings/BIND-ai-research-scientist-principal-h5e96d8.md`
- `context/bindings/BIND-product-manager-expert-r1-hf330ef.md` → `context/bindings/BIND-product-manager-expert-r1-hf330ef.md`
- `context/bindings/BIND-qa-engineer-specialist-r1-he98ba3.md` → `context/bindings/BIND-qa-engineer-specialist-r1-he98ba3.md`
- `context/bindings/BIND-product-manager-junior-h9275a0.md` → `context/bindings/BIND-product-manager-junior-h9275a0.md`
- `context/bindings/BIND-security-architect-senior-h142586.md` → `context/bindings/BIND-security-architect-senior-h142586.md`
- `context/bindings/BIND-data-scientist-senior-h8816d2.md` → `context/bindings/BIND-data-scientist-senior-h8816d2.md`
- `context/bindings/BIND-technical-writer-principal-h56b361.md` → `context/bindings/BIND-technical-writer-principal-h56b361.md`
- `context/bindings/BIND-assistant-principal-hb5ac7b.md` → `context/bindings/BIND-assistant-principal-hb5ac7b.md`
- `context/bindings/BIND-qa-engineer-senior-h0436ab.md` → `context/bindings/BIND-qa-engineer-senior-h0436ab.md`
- `context/bindings/BIND-ai-research-scientist-expert-r1-h3ea4c0.md` → `context/bindings/BIND-ai-research-scientist-expert-r1-h3ea4c0.md`
- `context/bindings/BIND-software-engineer-senior-r1-h79acd6.md` → `context/bindings/BIND-software-engineer-senior-r1-h79acd6.md`
- `context/bindings/BIND-qa-engineer-senior-r1-h522915.md` → `context/bindings/BIND-qa-engineer-senior-r1-h522915.md`
- `context/bindings/BIND-security-architect-principal-h2c8917.md` → `context/bindings/BIND-security-architect-principal-h2c8917.md`
- `context/bindings/BIND-software-engineer-senior-h7bd319.md` → `context/bindings/BIND-software-engineer-senior-h7bd319.md`
- `context/bindings/BIND-product-manager-principal-h723e55.md` → `context/bindings/BIND-product-manager-principal-h723e55.md`
- `context/bindings/BIND-data-scientist-expert-r1-hb8654f.md` → `context/bindings/BIND-data-scientist-expert-r1-hb8654f.md`
- `context/bindings/BIND-technical-writer-senior-r1-h63fa44.md` → `context/bindings/BIND-technical-writer-senior-r1-h63fa44.md`
- `context/bindings/BIND-solutions-architect-principal-h5264ce.md` → `context/bindings/BIND-solutions-architect-principal-h5264ce.md`
- `context/bindings/BIND-software-engineer-specialist-r1-h5a5377.md` → `context/bindings/BIND-software-engineer-specialist-r1-h5a5377.md`
- `context/bindings/BIND-software-engineer-expert-r1-ha8eea0.md` → `context/bindings/BIND-software-engineer-expert-r1-ha8eea0.md`
- `context/bindings/BIND-technical-writer-senior-r1-h93e646.md` → `context/bindings/BIND-technical-writer-senior-r1-h93e646.md`
- `context/bindings/BIND-qa-engineer-senior-r1-h4db0e9.md` → `context/bindings/BIND-qa-engineer-senior-r1-h4db0e9.md`
- `context/bindings/BIND-technical-writer-specialist-r1-h920232.md` → `context/bindings/BIND-technical-writer-specialist-r1-h920232.md`
- `context/bindings/BIND-machine-learning-engineer-senior-r1-hd54ded.md` → `context/bindings/BIND-machine-learning-engineer-senior-r1-hd54ded.md`
- `context/bindings/BIND-research-scientist-senior-h14c6d3.md` → `context/bindings/BIND-research-scientist-senior-h14c6d3.md`
- `context/bindings/BIND-software-engineer-principal-ha71725.md` → `context/bindings/BIND-software-engineer-principal-ha71725.md`
- `context/bindings/BIND-technical-writer-senior-hb93488.md` → `context/bindings/BIND-technical-writer-senior-hb93488.md`
- `context/bindings/BIND-data-scientist-principal-h192434.md` → `context/bindings/BIND-data-scientist-principal-h192434.md`
- `context/bindings/BIND-security-architect-junior-hf79510.md` → `context/bindings/BIND-security-architect-junior-hf79510.md`
- `context/bindings/BIND-product-manager-senior-r1-hf96017.md` → `context/bindings/BIND-product-manager-senior-r1-hf96017.md`
- `context/bindings/BIND-data-scientist-specialist-r1-h5f7da2.md` → `context/bindings/BIND-data-scientist-specialist-r1-h5f7da2.md`
- `context/bindings/BIND-qa-engineer-principal-h07f4cd.md` → `context/bindings/BIND-qa-engineer-principal-h07f4cd.md`
- `context/bindings/BIND-ai-research-scientist-senior-h18c312.md` → `context/bindings/BIND-ai-research-scientist-senior-h18c312.md`
- `context/bindings/BIND-product-manager-specialist-r1-h42cae9.md` → `context/bindings/BIND-product-manager-specialist-r1-h42cae9.md`
- `context/bindings/BIND-ai-research-scientist-specialist-r1-h52964c.md` → `context/bindings/BIND-ai-research-scientist-specialist-r1-h52964c.md`
- `context/bindings/BIND-solutions-architect-senior-h603b59.md` → `context/bindings/BIND-solutions-architect-senior-h603b59.md`
- `context/bindings/BIND-ai-research-scientist-junior-ha25e5b.md` → `context/bindings/BIND-ai-research-scientist-junior-ha25e5b.md`
- `context/bindings/BIND-security-architect-expert-r1-h636811.md` → `context/bindings/BIND-security-architect-expert-r1-h636811.md`
- `context/bindings/BIND-assistant-senior-h771629.md` → `context/bindings/BIND-assistant-senior-h771629.md`
- `context/bindings/BIND-assistant-specialist-r1-he4f117.md` → `context/bindings/BIND-assistant-specialist-r1-he4f117.md`

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

### context/team

**new-upstream**

- `context/team/roster.md` → `context/team/roster.md`

### context/team-members/cora

**new-upstream**

- `context/team-members/cora/team-member.md` → `context/team-members/cora/team-member.md`
- `context/team-members/cora/card.json` → `context/team-members/cora/card.json`
- `context/team-members/cora/persona.md` → `context/team-members/cora/persona.md`

### context/team-members/cora/relations

**new-upstream**

- `context/team-members/cora/relations/thrifty-otter.md` → `context/team-members/cora/relations/thrifty-otter.md`

### context/team-members/thrifty-otter

**new-upstream**

- `context/team-members/thrifty-otter/team-member.md` → `context/team-members/thrifty-otter/team-member.md`
- `context/team-members/thrifty-otter/card.json` → `context/team-members/thrifty-otter/card.json`
- `context/team-members/thrifty-otter/persona.md` → `context/team-members/thrifty-otter/persona.md`

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

### skills/documents

**new-upstream**

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

**new-upstream**

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
- `context/skills/engineering/git-workflow/SKILL.md` → `context/skills/engineering/git-workflow/SKILL.md`
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
- `context/skills/engineering/git-branching/references/strategies.md` → `context/skills/engineering/git-branching/references/strategies.md`
- `context/skills/engineering/git-branching/SKILL.md` → `context/skills/engineering/git-branching/SKILL.md`

### skills/processkit

**new-upstream**

- `context/skills/processkit/session-handover/SKILL.md` → `context/skills/processkit/session-handover/SKILL.md`
- `context/skills/processkit/session-handover/commands/pk-wrapup.md` → `context/skills/processkit/session-handover/commands/pk-wrapup.md`
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
- `context/skills/processkit/skill-reviewer/SKILL.md` → `context/skills/processkit/skill-reviewer/SKILL.md`
- `context/skills/processkit/skill-reviewer/commands/pk-skill-audit.md` → `context/skills/processkit/skill-reviewer/commands/pk-skill-audit.md`
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
- `context/skills/processkit/task-router/mcp/server.py` → `context/skills/processkit/task-router/mcp/server.py`
- `context/skills/processkit/task-router/mcp/SERVER.md` → `context/skills/processkit/task-router/mcp/SERVER.md`
- `context/skills/processkit/task-router/mcp/mcp-config.json` → `context/skills/processkit/task-router/mcp/mcp-config.json`
- `context/skills/processkit/task-router/mcp/user_config.json` → `context/skills/processkit/task-router/mcp/user_config.json`
- `context/skills/processkit/task-router/scripts/test_task_router.py` → `context/skills/processkit/task-router/scripts/test_task_router.py`
- `context/skills/processkit/task-router/SKILL.md` → `context/skills/processkit/task-router/SKILL.md`
- `context/skills/processkit/binding-management/mcp/server.py` → `context/skills/processkit/binding-management/mcp/server.py`
- `context/skills/processkit/binding-management/mcp/SERVER.md` → `context/skills/processkit/binding-management/mcp/SERVER.md`
- `context/skills/processkit/binding-management/mcp/mcp-config.json` → `context/skills/processkit/binding-management/mcp/mcp-config.json`
- `context/skills/processkit/binding-management/scripts/test_binding_management.py` → `context/skills/processkit/binding-management/scripts/test_binding_management.py`
- `context/skills/processkit/binding-management/SKILL.md` → `context/skills/processkit/binding-management/SKILL.md`
- `context/skills/processkit/binding-management/assets/binding.yaml` → `context/skills/processkit/binding-management/assets/binding.yaml`
- `context/skills/processkit/schedule-management/SKILL.md` → `context/skills/processkit/schedule-management/SKILL.md`
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
- `context/skills/processkit/status-briefing/SKILL.md` → `context/skills/processkit/status-briefing/SKILL.md`
- `context/skills/processkit/status-briefing/commands/pk-resume.md` → `context/skills/processkit/status-briefing/commands/pk-resume.md`
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
- `context/skills/processkit/skill-builder/references/library-expert-template.md` → `context/skills/processkit/skill-builder/references/library-expert-template.md`
- `context/skills/processkit/skill-builder/SKILL.md` → `context/skills/processkit/skill-builder/SKILL.md`
- `context/skills/processkit/skill-builder/commands/pk-skill-new.md` → `context/skills/processkit/skill-builder/commands/pk-skill-new.md`
- `context/skills/processkit/skill-builder/assets/skill-template.md` → `context/skills/processkit/skill-builder/assets/skill-template.md`
- `context/skills/processkit/scope-management/mcp/server.py` → `context/skills/processkit/scope-management/mcp/server.py`
- `context/skills/processkit/scope-management/mcp/SERVER.md` → `context/skills/processkit/scope-management/mcp/SERVER.md`
- `context/skills/processkit/scope-management/mcp/mcp-config.json` → `context/skills/processkit/scope-management/mcp/mcp-config.json`
- `context/skills/processkit/scope-management/SKILL.md` → `context/skills/processkit/scope-management/SKILL.md`
- `context/skills/processkit/scope-management/assets/scope.yaml` → `context/skills/processkit/scope-management/assets/scope.yaml`
- `context/skills/processkit/retrospective/scripts/test_retro.py` → `context/skills/processkit/retrospective/scripts/test_retro.py`
- `context/skills/processkit/retrospective/scripts/signals/timeline.py` → `context/skills/processkit/retrospective/scripts/signals/timeline.py`
- `context/skills/processkit/retrospective/scripts/signals/drift.py` → `context/skills/processkit/retrospective/scripts/signals/drift.py`
- `context/skills/processkit/retrospective/scripts/signals/__init__.py` → `context/skills/processkit/retrospective/scripts/signals/__init__.py`
- `context/skills/processkit/retrospective/scripts/signals/workitems.py` → `context/skills/processkit/retrospective/scripts/signals/workitems.py`
- `context/skills/processkit/retrospective/scripts/signals/release_summary.py` → `context/skills/processkit/retrospective/scripts/signals/release_summary.py`
- `context/skills/processkit/retrospective/scripts/pk_retro.py` → `context/skills/processkit/retrospective/scripts/pk_retro.py`
- `context/skills/processkit/retrospective/SKILL.md` → `context/skills/processkit/retrospective/SKILL.md`
- `context/skills/processkit/retrospective/commands/pk-retro.md` → `context/skills/processkit/retrospective/commands/pk-retro.md`
- `context/skills/processkit/state-machine-management/SKILL.md` → `context/skills/processkit/state-machine-management/SKILL.md`
- `context/skills/processkit/security-projections/mcp/server.py` → `context/skills/processkit/security-projections/mcp/server.py`
- `context/skills/processkit/security-projections/mcp/SERVER.md` → `context/skills/processkit/security-projections/mcp/SERVER.md`
- `context/skills/processkit/security-projections/mcp/mcp-config.json` → `context/skills/processkit/security-projections/mcp/mcp-config.json`
- `context/skills/processkit/security-projections/SKILL.md` → `context/skills/processkit/security-projections/SKILL.md`
- `context/skills/processkit/category-management/SKILL.md` → `context/skills/processkit/category-management/SKILL.md`
- `context/skills/processkit/category-management/assets/category.yaml` → `context/skills/processkit/category-management/assets/category.yaml`
- `context/skills/processkit/processkit-gateway/mcp/server.py` → `context/skills/processkit/processkit-gateway/mcp/server.py`
- `context/skills/processkit/processkit-gateway/mcp/tool-catalog.json` → `context/skills/processkit/processkit-gateway/mcp/tool-catalog.json`
- `context/skills/processkit/processkit-gateway/mcp/mcp-config.json` → `context/skills/processkit/processkit-gateway/mcp/mcp-config.json`
- `context/skills/processkit/processkit-gateway/scripts/test_gateway.py` → `context/skills/processkit/processkit-gateway/scripts/test_gateway.py`
- `context/skills/processkit/processkit-gateway/scripts/test_daemon_runtime.py` → `context/skills/processkit/processkit-gateway/scripts/test_daemon_runtime.py`
- `context/skills/processkit/processkit-gateway/scripts/test_proxy.py` → `context/skills/processkit/processkit-gateway/scripts/test_proxy.py`
- `context/skills/processkit/processkit-gateway/SKILL.md` → `context/skills/processkit/processkit-gateway/SKILL.md`
- `context/skills/processkit/release-audit/mcp/server.py` → `context/skills/processkit/release-audit/mcp/server.py`
- `context/skills/processkit/release-audit/scripts/release_audit.py` → `context/skills/processkit/release-audit/scripts/release_audit.py`
- `context/skills/processkit/release-audit/scripts/test_release_audit_json.py` → `context/skills/processkit/release-audit/scripts/test_release_audit_json.py`
- `context/skills/processkit/release-audit/scripts/test_release_audit.py` → `context/skills/processkit/release-audit/scripts/test_release_audit.py`
- `context/skills/processkit/release-audit/SKILL.md` → `context/skills/processkit/release-audit/SKILL.md`
- `context/skills/processkit/release-audit/commands/pk-release-audit.md` → `context/skills/processkit/release-audit/commands/pk-release-audit.md`
- `context/skills/processkit/team-manager/mcp/server.py` → `context/skills/processkit/team-manager/mcp/server.py`
- `context/skills/processkit/team-manager/mcp/SERVER.md` → `context/skills/processkit/team-manager/mcp/SERVER.md`
- `context/skills/processkit/team-manager/mcp/mcp-config.json` → `context/skills/processkit/team-manager/mcp/mcp-config.json`
- `context/skills/processkit/team-manager/scripts/name_pool.py` → `context/skills/processkit/team-manager/scripts/name_pool.py`
- `context/skills/processkit/team-manager/scripts/consistency.py` → `context/skills/processkit/team-manager/scripts/consistency.py`
- `context/skills/processkit/team-manager/scripts/export_import.py` → `context/skills/processkit/team-manager/scripts/export_import.py`
- `context/skills/processkit/team-manager/scripts/apply_migration_2139.py` → `context/skills/processkit/team-manager/scripts/apply_migration_2139.py`
- `context/skills/processkit/team-manager/scripts/memory_tree.py` → `context/skills/processkit/team-manager/scripts/memory_tree.py`
- `context/skills/processkit/team-manager/scripts/test_team_manager.py` → `context/skills/processkit/team-manager/scripts/test_team_manager.py`
- `context/skills/processkit/team-manager/SKILL.md` → `context/skills/processkit/team-manager/SKILL.md`
- `context/skills/processkit/team-manager/data/name-pool.yaml` → `context/skills/processkit/team-manager/data/name-pool.yaml`
- `context/skills/processkit/team-manager/assets/agent-card.schema.json` → `context/skills/processkit/team-manager/assets/agent-card.schema.json`
- `context/skills/processkit/team-manager/assets/team-member-human.yaml` → `context/skills/processkit/team-manager/assets/team-member-human.yaml`
- `context/skills/processkit/team-manager/assets/memory-file-header.schema.json` → `context/skills/processkit/team-manager/assets/memory-file-header.schema.json`
- `context/skills/processkit/team-manager/assets/team-member-ai-agent.yaml` → `context/skills/processkit/team-manager/assets/team-member-ai-agent.yaml`
- `context/skills/processkit/migration-management/mcp/server.py` → `context/skills/processkit/migration-management/mcp/server.py`
- `context/skills/processkit/migration-management/mcp/SERVER.md` → `context/skills/processkit/migration-management/mcp/SERVER.md`
- `context/skills/processkit/migration-management/mcp/mcp-config.json` → `context/skills/processkit/migration-management/mcp/mcp-config.json`
- `context/skills/processkit/migration-management/scripts/test_migration_management.py` → `context/skills/processkit/migration-management/scripts/test_migration_management.py`
- `context/skills/processkit/migration-management/SKILL.md` → `context/skills/processkit/migration-management/SKILL.md`
- `context/skills/processkit/migration-management/assets/migration.yaml` → `context/skills/processkit/migration-management/assets/migration.yaml`
- `context/skills/processkit/workitem-management/mcp/server.py` → `context/skills/processkit/workitem-management/mcp/server.py`
- `context/skills/processkit/workitem-management/mcp/SERVER.md` → `context/skills/processkit/workitem-management/mcp/SERVER.md`
- `context/skills/processkit/workitem-management/mcp/mcp-config.json` → `context/skills/processkit/workitem-management/mcp/mcp-config.json`
- `context/skills/processkit/workitem-management/examples/create-feature.md` → `context/skills/processkit/workitem-management/examples/create-feature.md`
- `context/skills/processkit/workitem-management/scripts/test_workitem_management.py` → `context/skills/processkit/workitem-management/scripts/test_workitem_management.py`
- `context/skills/processkit/workitem-management/SKILL.md` → `context/skills/processkit/workitem-management/SKILL.md`
- `context/skills/processkit/workitem-management/commands/pk-work.md` → `context/skills/processkit/workitem-management/commands/pk-work.md`
- `context/skills/processkit/workitem-management/assets/workitem-story.yaml` → `context/skills/processkit/workitem-management/assets/workitem-story.yaml`
- `context/skills/processkit/workitem-management/assets/workitem.yaml` → `context/skills/processkit/workitem-management/assets/workitem.yaml`
- `context/skills/processkit/workitem-management/assets/workitem-bug.yaml` → `context/skills/processkit/workitem-management/assets/workitem-bug.yaml`
- `context/skills/processkit/context-archiving/mcp/server.py` → `context/skills/processkit/context-archiving/mcp/server.py`
- `context/skills/processkit/context-archiving/scripts/test_context_archiving.py` → `context/skills/processkit/context-archiving/scripts/test_context_archiving.py`
- `context/skills/processkit/context-archiving/SKILL.md` → `context/skills/processkit/context-archiving/SKILL.md`
- `context/skills/processkit/actor-profile/mcp/server.py` → `context/skills/processkit/actor-profile/mcp/server.py`
- `context/skills/processkit/actor-profile/mcp/SERVER.md` → `context/skills/processkit/actor-profile/mcp/SERVER.md`
- `context/skills/processkit/actor-profile/mcp/mcp-config.json` → `context/skills/processkit/actor-profile/mcp/mcp-config.json`
- `context/skills/processkit/actor-profile/SKILL.md` → `context/skills/processkit/actor-profile/SKILL.md`
- `context/skills/processkit/actor-profile/assets/actor-agent.yaml` → `context/skills/processkit/actor-profile/assets/actor-agent.yaml`
- `context/skills/processkit/actor-profile/assets/actor-human.yaml` → `context/skills/processkit/actor-profile/assets/actor-human.yaml`
- `context/skills/processkit/project-reconciliation/SKILL.md` → `context/skills/processkit/project-reconciliation/SKILL.md`
- `context/skills/processkit/project-reconciliation/commands/pk-reconcile.md` → `context/skills/processkit/project-reconciliation/commands/pk-reconcile.md`
- `context/skills/processkit/standup-context/SKILL.md` → `context/skills/processkit/standup-context/SKILL.md`
- `context/skills/processkit/standup-context/commands/pk-status.md` → `context/skills/processkit/standup-context/commands/pk-status.md`
- `context/skills/processkit/standup-context/commands/pk-standup.md` → `context/skills/processkit/standup-context/commands/pk-standup.md`
- `context/skills/processkit/pk-doctor/mcp/server.py` → `context/skills/processkit/pk-doctor/mcp/server.py`
- `context/skills/processkit/pk-doctor/mcp/mcp-config.json` → `context/skills/processkit/pk-doctor/mcp/mcp-config.json`
- `context/skills/processkit/pk-doctor/scripts/test_pk_doctor_json.py` → `context/skills/processkit/pk-doctor/scripts/test_pk_doctor_json.py`
- `context/skills/processkit/pk-doctor/scripts/checks/sensitive_data.py` → `context/skills/processkit/pk-doctor/scripts/checks/sensitive_data.py`
- `context/skills/processkit/pk-doctor/scripts/checks/server_header_drift.py` → `context/skills/processkit/pk-doctor/scripts/checks/server_header_drift.py`
- `context/skills/processkit/pk-doctor/scripts/checks/v2_contracts.py` → `context/skills/processkit/pk-doctor/scripts/checks/v2_contracts.py`
- `context/skills/processkit/pk-doctor/scripts/checks/mcp_config_drift.py` → `context/skills/processkit/pk-doctor/scripts/checks/mcp_config_drift.py`
- `context/skills/processkit/pk-doctor/scripts/checks/team_member_exports.py` → `context/skills/processkit/pk-doctor/scripts/checks/team_member_exports.py`
- `context/skills/processkit/pk-doctor/scripts/checks/sharding.py` → `context/skills/processkit/pk-doctor/scripts/checks/sharding.py`
- `context/skills/processkit/pk-doctor/scripts/checks/mcp_gateway.py` → `context/skills/processkit/pk-doctor/scripts/checks/mcp_gateway.py`
- `context/skills/processkit/pk-doctor/scripts/checks/release_integrity.py` → `context/skills/processkit/pk-doctor/scripts/checks/release_integrity.py`
- `context/skills/processkit/pk-doctor/scripts/checks/v1_entity_drift.py` → `context/skills/processkit/pk-doctor/scripts/checks/v1_entity_drift.py`
- `context/skills/processkit/pk-doctor/scripts/checks/migration_integrity.py` → `context/skills/processkit/pk-doctor/scripts/checks/migration_integrity.py`
- `context/skills/processkit/pk-doctor/scripts/checks/context_hygiene.py` → `context/skills/processkit/pk-doctor/scripts/checks/context_hygiene.py`
- `context/skills/processkit/pk-doctor/scripts/checks/drift.py` → `context/skills/processkit/pk-doctor/scripts/checks/drift.py`
- `context/skills/processkit/pk-doctor/scripts/checks/__init__.py` → `context/skills/processkit/pk-doctor/scripts/checks/__init__.py`
- `context/skills/processkit/pk-doctor/scripts/checks/id_vocabulary.py` → `context/skills/processkit/pk-doctor/scripts/checks/id_vocabulary.py`
- `context/skills/processkit/pk-doctor/scripts/checks/supply_chain.py` → `context/skills/processkit/pk-doctor/scripts/checks/supply_chain.py`
- `context/skills/processkit/pk-doctor/scripts/checks/runtime_health.py` → `context/skills/processkit/pk-doctor/scripts/checks/runtime_health.py`
- `context/skills/processkit/pk-doctor/scripts/checks/agents_md_hygiene.py` → `context/skills/processkit/pk-doctor/scripts/checks/agents_md_hygiene.py`
- `context/skills/processkit/pk-doctor/scripts/checks/skill_dag.py` → `context/skills/processkit/pk-doctor/scripts/checks/skill_dag.py`
- `context/skills/processkit/pk-doctor/scripts/checks/context_consumption.py` → `context/skills/processkit/pk-doctor/scripts/checks/context_consumption.py`
- `context/skills/processkit/pk-doctor/scripts/checks/preauth_applied.py` → `context/skills/processkit/pk-doctor/scripts/checks/preauth_applied.py`
- `context/skills/processkit/pk-doctor/scripts/checks/common.py` → `context/skills/processkit/pk-doctor/scripts/checks/common.py`
- `context/skills/processkit/pk-doctor/scripts/checks/migrations.py` → `context/skills/processkit/pk-doctor/scripts/checks/migrations.py`
- `context/skills/processkit/pk-doctor/scripts/checks/team_consistency.py` → `context/skills/processkit/pk-doctor/scripts/checks/team_consistency.py`
- `context/skills/processkit/pk-doctor/scripts/checks/doctor_boundary.py` → `context/skills/processkit/pk-doctor/scripts/checks/doctor_boundary.py`
- `context/skills/processkit/pk-doctor/scripts/checks/schema_filename.py` → `context/skills/processkit/pk-doctor/scripts/checks/schema_filename.py`
- `context/skills/processkit/pk-doctor/scripts/checks/entity_storage_hygiene.py` → `context/skills/processkit/pk-doctor/scripts/checks/entity_storage_hygiene.py`
- `context/skills/processkit/pk-doctor/scripts/checks/schema_vocabulary.py` → `context/skills/processkit/pk-doctor/scripts/checks/schema_vocabulary.py`
- `context/skills/processkit/pk-doctor/scripts/checks/commands_consistency.py` → `context/skills/processkit/pk-doctor/scripts/checks/commands_consistency.py`
- `context/skills/processkit/pk-doctor/scripts/test_id_vocabulary.py` → `context/skills/processkit/pk-doctor/scripts/test_id_vocabulary.py`
- `context/skills/processkit/pk-doctor/scripts/test_doctor.py` → `context/skills/processkit/pk-doctor/scripts/test_doctor.py`
- `context/skills/processkit/pk-doctor/scripts/test_pk_doctor_mcp.py` → `context/skills/processkit/pk-doctor/scripts/test_pk_doctor_mcp.py`
- `context/skills/processkit/pk-doctor/scripts/doctor.py` → `context/skills/processkit/pk-doctor/scripts/doctor.py`
- `context/skills/processkit/pk-doctor/SKILL.md` → `context/skills/processkit/pk-doctor/SKILL.md`
- `context/skills/processkit/pk-doctor/commands/pk-doctor.md` → `context/skills/processkit/pk-doctor/commands/pk-doctor.md`
- `context/skills/processkit/decision-record/mcp/server.py` → `context/skills/processkit/decision-record/mcp/server.py`
- `context/skills/processkit/decision-record/mcp/SERVER.md` → `context/skills/processkit/decision-record/mcp/SERVER.md`
- `context/skills/processkit/decision-record/mcp/mcp-config.json` → `context/skills/processkit/decision-record/mcp/mcp-config.json`
- `context/skills/processkit/decision-record/SKILL.md` → `context/skills/processkit/decision-record/SKILL.md`
- `context/skills/processkit/decision-record/commands/pk-dec.md` → `context/skills/processkit/decision-record/commands/pk-dec.md`
- `context/skills/processkit/decision-record/commands/pk-dec-find.md` → `context/skills/processkit/decision-record/commands/pk-dec-find.md`
- `context/skills/processkit/decision-record/assets/decisionrecord.yaml` → `context/skills/processkit/decision-record/assets/decisionrecord.yaml`
- `context/skills/processkit/constraint-management/SKILL.md` → `context/skills/processkit/constraint-management/SKILL.md`
- `context/skills/processkit/constraint-management/assets/constraint.yaml` → `context/skills/processkit/constraint-management/assets/constraint.yaml`
- `context/skills/processkit/cross-reference-management/SKILL.md` → `context/skills/processkit/cross-reference-management/SKILL.md`
- `context/skills/processkit/supply-chain-audit/mcp/server.py` → `context/skills/processkit/supply-chain-audit/mcp/server.py`
- `context/skills/processkit/supply-chain-audit/mcp/SERVER.md` → `context/skills/processkit/supply-chain-audit/mcp/SERVER.md`
- `context/skills/processkit/supply-chain-audit/mcp/mcp-config.json` → `context/skills/processkit/supply-chain-audit/mcp/mcp-config.json`
- `context/skills/processkit/supply-chain-audit/scripts/supply_chain_audit.py` → `context/skills/processkit/supply-chain-audit/scripts/supply_chain_audit.py`
- `context/skills/processkit/supply-chain-audit/scripts/test_supply_chain_audit.py` → `context/skills/processkit/supply-chain-audit/scripts/test_supply_chain_audit.py`
- `context/skills/processkit/supply-chain-audit/SKILL.md` → `context/skills/processkit/supply-chain-audit/SKILL.md`
- `context/skills/processkit/supply-chain-audit/commands/pk-supply-chain.md` → `context/skills/processkit/supply-chain-audit/commands/pk-supply-chain.md`
- `context/skills/processkit/role-management/mcp/server.py` → `context/skills/processkit/role-management/mcp/server.py`
- `context/skills/processkit/role-management/mcp/SERVER.md` → `context/skills/processkit/role-management/mcp/SERVER.md`
- `context/skills/processkit/role-management/mcp/mcp-config.json` → `context/skills/processkit/role-management/mcp/mcp-config.json`
- `context/skills/processkit/role-management/SKILL.md` → `context/skills/processkit/role-management/SKILL.md`
- `context/skills/processkit/role-management/assets/role.yaml` → `context/skills/processkit/role-management/assets/role.yaml`
- `context/skills/processkit/skill-gate/mcp/server.py` → `context/skills/processkit/skill-gate/mcp/server.py`
- `context/skills/processkit/skill-gate/mcp/SERVER.md` → `context/skills/processkit/skill-gate/mcp/SERVER.md`
- `context/skills/processkit/skill-gate/mcp/mcp-config.json` → `context/skills/processkit/skill-gate/mcp/mcp-config.json`
- `context/skills/processkit/skill-gate/scripts/test_entity_read_and_agent_hooks.py` → `context/skills/processkit/skill-gate/scripts/test_entity_read_and_agent_hooks.py`
- `context/skills/processkit/skill-gate/scripts/emit_compliance_contract.py` → `context/skills/processkit/skill-gate/scripts/emit_compliance_contract.py`
- `context/skills/processkit/skill-gate/scripts/README.md` → `context/skills/processkit/skill-gate/scripts/README.md`
- `context/skills/processkit/skill-gate/scripts/test_hooks.py` → `context/skills/processkit/skill-gate/scripts/test_hooks.py`
- `context/skills/processkit/skill-gate/scripts/check_route_task_before_agent.py` → `context/skills/processkit/skill-gate/scripts/check_route_task_before_agent.py`
- `context/skills/processkit/skill-gate/scripts/fixtures/sample-transcript-no-decisions.jsonl` → `context/skills/processkit/skill-gate/scripts/fixtures/sample-transcript-no-decisions.jsonl`
- `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-pretooluse-sample.json` → `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-pretooluse-sample.json`
- `context/skills/processkit/skill-gate/scripts/fixtures/sample-transcript-with-poison-entries.jsonl` → `context/skills/processkit/skill-gate/scripts/fixtures/sample-transcript-with-poison-entries.jsonl`
- `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-session-start.json` → `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-session-start.json`
- `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-pre-tool-use.json` → `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-pre-tool-use.json`
- `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-sessionend.json` → `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-sessionend.json`
- `context/skills/processkit/skill-gate/scripts/fixtures/sample-transcript-with-decisions.jsonl` → `context/skills/processkit/skill-gate/scripts/fixtures/sample-transcript-with-decisions.jsonl`
- `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-pretooluse-with-transcript.json` → `context/skills/processkit/skill-gate/scripts/fixtures/claude-code-pretooluse-with-transcript.json`
- `context/skills/processkit/skill-gate/scripts/check_route_task_called.py` → `context/skills/processkit/skill-gate/scripts/check_route_task_called.py`
- `context/skills/processkit/skill-gate/scripts/decision_markers.py` → `context/skills/processkit/skill-gate/scripts/decision_markers.py`
- `context/skills/processkit/skill-gate/scripts/record_decision_observer.py` → `context/skills/processkit/skill-gate/scripts/record_decision_observer.py`
- `context/skills/processkit/skill-gate/scripts/decision_sweeper.py` → `context/skills/processkit/skill-gate/scripts/decision_sweeper.py`
- `context/skills/processkit/skill-gate/scripts/check_decision_captured.py` → `context/skills/processkit/skill-gate/scripts/check_decision_captured.py`
- `context/skills/processkit/skill-gate/scripts/check_entity_read.py` → `context/skills/processkit/skill-gate/scripts/check_entity_read.py`
- `context/skills/processkit/skill-gate/SKILL.md` → `context/skills/processkit/skill-gate/SKILL.md`
- `context/skills/processkit/skill-gate/commands/pk-lint.md` → `context/skills/processkit/skill-gate/commands/pk-lint.md`
- `context/skills/processkit/skill-gate/commands/pk-test.md` → `context/skills/processkit/skill-gate/commands/pk-test.md`
- `context/skills/processkit/skill-gate/commands/pk-build.md` → `context/skills/processkit/skill-gate/commands/pk-build.md`
- `context/skills/processkit/skill-gate/commands/pk-review.md` → `context/skills/processkit/skill-gate/commands/pk-review.md`
- `context/skills/processkit/skill-gate/assets/compliance-contract.md` → `context/skills/processkit/skill-gate/assets/compliance-contract.md`
- `context/skills/processkit/skill-gate/assets/preauth.json` → `context/skills/processkit/skill-gate/assets/preauth.json`
- `context/skills/processkit/skill-finder/mcp/server.py` → `context/skills/processkit/skill-finder/mcp/server.py`
- `context/skills/processkit/skill-finder/mcp/SERVER.md` → `context/skills/processkit/skill-finder/mcp/SERVER.md`
- `context/skills/processkit/skill-finder/mcp/mcp-config.json` → `context/skills/processkit/skill-finder/mcp/mcp-config.json`
- `context/skills/processkit/skill-finder/scripts/test_catalog.py` → `context/skills/processkit/skill-finder/scripts/test_catalog.py`
- `context/skills/processkit/skill-finder/SKILL.md` → `context/skills/processkit/skill-finder/SKILL.md`
- `context/skills/processkit/agent-card/mcp/server.py` → `context/skills/processkit/agent-card/mcp/server.py`
- `context/skills/processkit/agent-card/mcp/SERVER.md` → `context/skills/processkit/agent-card/mcp/SERVER.md`
- `context/skills/processkit/agent-card/mcp/mcp-config.json` → `context/skills/processkit/agent-card/mcp/mcp-config.json`
- `context/skills/processkit/agent-card/SKILL.md` → `context/skills/processkit/agent-card/SKILL.md`
- `context/skills/processkit/id-management/config/settings.toml` → `context/skills/processkit/id-management/config/settings.toml`
- `context/skills/processkit/id-management/mcp/server.py` → `context/skills/processkit/id-management/mcp/server.py`
- `context/skills/processkit/id-management/mcp/SERVER.md` → `context/skills/processkit/id-management/mcp/SERVER.md`
- `context/skills/processkit/id-management/mcp/mcp-config.json` → `context/skills/processkit/id-management/mcp/mcp-config.json`
- `context/skills/processkit/id-management/SKILL.md` → `context/skills/processkit/id-management/SKILL.md`
- `context/skills/processkit/status-update-writer/SKILL.md` → `context/skills/processkit/status-update-writer/SKILL.md`
- `context/skills/processkit/repository-portfolio-review/SKILL.md` → `context/skills/processkit/repository-portfolio-review/SKILL.md`
- `context/skills/processkit/discussion-management/mcp/server.py` → `context/skills/processkit/discussion-management/mcp/server.py`
- `context/skills/processkit/discussion-management/mcp/SERVER.md` → `context/skills/processkit/discussion-management/mcp/SERVER.md`
- `context/skills/processkit/discussion-management/mcp/mcp-config.json` → `context/skills/processkit/discussion-management/mcp/mcp-config.json`
- `context/skills/processkit/discussion-management/SKILL.md` → `context/skills/processkit/discussion-management/SKILL.md`
- `context/skills/processkit/discussion-management/commands/pk-discuss.md` → `context/skills/processkit/discussion-management/commands/pk-discuss.md`
- `context/skills/processkit/discussion-management/assets/discussion.yaml` → `context/skills/processkit/discussion-management/assets/discussion.yaml`
- `context/skills/processkit/context-grooming/SKILL.md` → `context/skills/processkit/context-grooming/SKILL.md`
- `context/skills/processkit/context-grooming/commands/pk-groom.md` → `context/skills/processkit/context-grooming/commands/pk-groom.md`
- `context/skills/processkit/context-grooming/assets/grooming-report.md` → `context/skills/processkit/context-grooming/assets/grooming-report.md`
- `context/skills/processkit/runtime-prune/mcp/server.py` → `context/skills/processkit/runtime-prune/mcp/server.py`
- `context/skills/processkit/runtime-prune/mcp/SERVER.md` → `context/skills/processkit/runtime-prune/mcp/SERVER.md`
- `context/skills/processkit/runtime-prune/mcp/mcp-config.json` → `context/skills/processkit/runtime-prune/mcp/mcp-config.json`
- `context/skills/processkit/runtime-prune/scripts/test_runtime_prune.py` → `context/skills/processkit/runtime-prune/scripts/test_runtime_prune.py`
- `context/skills/processkit/runtime-prune/SKILL.md` → `context/skills/processkit/runtime-prune/SKILL.md`
- `context/skills/processkit/model-recommender/references/model-profiles.md` → `context/skills/processkit/model-recommender/references/model-profiles.md`
- `context/skills/processkit/model-recommender/references/roster-quick-ref.md` → `context/skills/processkit/model-recommender/references/roster-quick-ref.md`
- `context/skills/processkit/model-recommender/references/model-characteristics.md` → `context/skills/processkit/model-recommender/references/model-characteristics.md`
- `context/skills/processkit/model-recommender/references/dimension-specs.md` → `context/skills/processkit/model-recommender/references/dimension-specs.md`
- `context/skills/processkit/model-recommender/mcp/server.py` → `context/skills/processkit/model-recommender/mcp/server.py`
- `context/skills/processkit/model-recommender/mcp/model_scores.json` → `context/skills/processkit/model-recommender/mcp/model_scores.json`
- `context/skills/processkit/model-recommender/mcp/SERVER.md` → `context/skills/processkit/model-recommender/mcp/SERVER.md`
- `context/skills/processkit/model-recommender/mcp/mcp-config.json` → `context/skills/processkit/model-recommender/mcp/mcp-config.json`
- `context/skills/processkit/model-recommender/mcp/user_config.json` → `context/skills/processkit/model-recommender/mcp/user_config.json`
- `context/skills/processkit/model-recommender/examples/profile-view.md` → `context/skills/processkit/model-recommender/examples/profile-view.md`
- `context/skills/processkit/model-recommender/examples/task-routing.md` → `context/skills/processkit/model-recommender/examples/task-routing.md`
- `context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml` → `context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml`
- `context/skills/processkit/model-recommender/scripts/migrate_model_profiles.py` → `context/skills/processkit/model-recommender/scripts/migrate_model_profiles.py`
- `context/skills/processkit/model-recommender/scripts/test_migrate_models.py` → `context/skills/processkit/model-recommender/scripts/test_migrate_models.py`
- `context/skills/processkit/model-recommender/scripts/migrate_models.py` → `context/skills/processkit/model-recommender/scripts/migrate_models.py`
- `context/skills/processkit/model-recommender/scripts/resolver.py` → `context/skills/processkit/model-recommender/scripts/resolver.py`
- `context/skills/processkit/model-recommender/scripts/test_default_bindings_coverage.py` → `context/skills/processkit/model-recommender/scripts/test_default_bindings_coverage.py`
- `context/skills/processkit/model-recommender/scripts/test_resolver.py` → `context/skills/processkit/model-recommender/scripts/test_resolver.py`
- `context/skills/processkit/model-recommender/scripts/test_query_models_filters.py` → `context/skills/processkit/model-recommender/scripts/test_query_models_filters.py`
- `context/skills/processkit/model-recommender/SKILL.md` → `context/skills/processkit/model-recommender/SKILL.md`
- `context/skills/processkit/model-recommender/commands/pk-model-setup.md` → `context/skills/processkit/model-recommender/commands/pk-model-setup.md`
- `context/skills/processkit/model-recommender/commands/pk-model-refresh.md` → `context/skills/processkit/model-recommender/commands/pk-model-refresh.md`
- `context/skills/processkit/model-recommender/commands/pk-explain-routing.md` → `context/skills/processkit/model-recommender/commands/pk-explain-routing.md`
- `context/skills/processkit/model-recommender/commands/pk-route.md` → `context/skills/processkit/model-recommender/commands/pk-route.md`
- `context/skills/processkit/gate-management/mcp/server.py` → `context/skills/processkit/gate-management/mcp/server.py`
- `context/skills/processkit/gate-management/mcp/SERVER.md` → `context/skills/processkit/gate-management/mcp/SERVER.md`
- `context/skills/processkit/gate-management/mcp/mcp-config.json` → `context/skills/processkit/gate-management/mcp/mcp-config.json`
- `context/skills/processkit/gate-management/scripts/test_gate_management.py` → `context/skills/processkit/gate-management/scripts/test_gate_management.py`
- `context/skills/processkit/gate-management/SKILL.md` → `context/skills/processkit/gate-management/SKILL.md`
- `context/skills/processkit/gate-management/assets/gate.yaml` → `context/skills/processkit/gate-management/assets/gate.yaml`
- `context/skills/processkit/team-creator/references/tiering-formula.md` → `context/skills/processkit/team-creator/references/tiering-formula.md`
- `context/skills/processkit/team-creator/references/landscape-resolution.md` → `context/skills/processkit/team-creator/references/landscape-resolution.md`
- `context/skills/processkit/team-creator/references/role-archetypes.md` → `context/skills/processkit/team-creator/references/role-archetypes.md`
- `context/skills/processkit/team-creator/references/role-archetypes-override.md` → `context/skills/processkit/team-creator/references/role-archetypes-override.md`
- `context/skills/processkit/team-creator/references/team-weights-decision-schema.md` → `context/skills/processkit/team-creator/references/team-weights-decision-schema.md`
- `context/skills/processkit/team-creator/scripts/team_creator_lib.py` → `context/skills/processkit/team-creator/scripts/team_creator_lib.py`
- `context/skills/processkit/team-creator/SKILL.md` → `context/skills/processkit/team-creator/SKILL.md`
- `context/skills/processkit/team-creator/commands/pk-team-review.md` → `context/skills/processkit/team-creator/commands/pk-team-review.md`
- `context/skills/processkit/team-creator/commands/pk-team-rebalance.md` → `context/skills/processkit/team-creator/commands/pk-team-rebalance.md`
- `context/skills/processkit/team-creator/commands/pk-team-create.md` → `context/skills/processkit/team-creator/commands/pk-team-create.md`
- `context/skills/processkit/team-creator/assets/archetype-catalog-mapping.yaml` → `context/skills/processkit/team-creator/assets/archetype-catalog-mapping.yaml`

### skills/product

**new-upstream**

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

### state-machines/INDEX

**new-upstream**

- `context/state-machines/INDEX.md` → `context/state-machines/INDEX.md`

### state-machines/decisionrecord

**new-upstream**

- `context/state-machines/decisionrecord.yaml` → `context/state-machines/decisionrecord.yaml`

### state-machines/discussion

**new-upstream**

- `context/state-machines/discussion.yaml` → `context/state-machines/discussion.yaml`

### state-machines/migration

**new-upstream**

- `context/state-machines/migration.yaml` → `context/state-machines/migration.yaml`

### state-machines/note

**new-upstream**

- `context/state-machines/note.yaml` → `context/state-machines/note.yaml`

### state-machines/scope

**new-upstream**

- `context/state-machines/scope.yaml` → `context/state-machines/scope.yaml`

### state-machines/workitem

**new-upstream**

- `context/state-machines/workitem.yaml` → `context/state-machines/workitem.yaml`
