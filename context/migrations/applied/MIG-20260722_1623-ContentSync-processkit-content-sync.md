---
apiVersion: processkit.projectious.work/v1
kind: Migration
metadata:
  id: MIG-20260722_1623-ContentSync-processkit-content-sync
  created: 2026-07-22 16:23:25+00:00
  updated: '2026-07-22T17:47:31+00:00'
spec:
  source: processkit
  source_url: https://github.com/projectious-work/processkit.git
  from_version: v0.27.5
  to_version: v0.28.1
  state: applied
  generated_by: aibox apply
  generated_at: 2026-07-22 16:23:25+00:00
  summary: 0 changed upstream, 0 conflicts, 5 new, 16 removed, 0 stale-removed (6
    groups affected)
  affected_groups:
  - AGENTS
  - lib
  - schemas/_generated
  - schemas/src
  - skills/engineering
  - skills/processkit
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
  - path: context/skills/_lib/processkit/schema_generation.py
    classification: removed-upstream
  - path: context/skills/engineering/git-branching/SKILL.md
    classification: new-upstream
  - path: context/skills/engineering/git-branching/references/strategies.md
    classification: new-upstream
  - path: context/skills/processkit/pk-doctor/scripts/test_pk_doctor_mcp.py
    classification: new-upstream
  - path: context/skills/processkit/project-reconciliation/SKILL.md
    classification: new-upstream
  - path: context/skills/processkit/project-reconciliation/commands/pk-reconcile.md
    classification: new-upstream
  started_at: '2026-07-22T17:47:30+00:00'
  applied_at: '2026-07-22T17:47:31+00:00'
  progress_notes:
  - timestamp: '2026-07-22T17:47:31+00:00'
    actor: mcp
    note: Applied with owner authorization during pk-reconcile; removes upstream-retired
      schema generation files and adds processkit reconciliation content.
---

# Migration MIG-20260722_1623-ContentSync-processkit-content-sync

From `v0.27.5` to `v0.28.1` (source: `https://github.com/projectious-work/processkit.git`).

0 changed upstream, 0 conflicts, 5 new, 16 removed, 0 stale-removed (6 groups affected)

## Counts

- unchanged: 707
- changed-locally-only: 1
- changed-upstream-only: 0
- conflict: 0
- new-upstream: 5
- removed-upstream: 16
- removed-upstream-stale: 0

## Changes by group

### AGENTS

**changed-locally-only**

- `AGENTS.md` → `AGENTS.md`

### lib

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

### skills/engineering

**new-upstream**

- `context/skills/engineering/git-branching/references/strategies.md` → `context/skills/engineering/git-branching/references/strategies.md`
- `context/skills/engineering/git-branching/SKILL.md` → `context/skills/engineering/git-branching/SKILL.md`

### skills/processkit

**new-upstream**

- `context/skills/processkit/project-reconciliation/SKILL.md` → `context/skills/processkit/project-reconciliation/SKILL.md`
- `context/skills/processkit/project-reconciliation/commands/pk-reconcile.md` → `context/skills/processkit/project-reconciliation/commands/pk-reconcile.md`
- `context/skills/processkit/pk-doctor/scripts/test_pk_doctor_mcp.py` → `context/skills/processkit/pk-doctor/scripts/test_pk_doctor_mcp.py`
