---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260417_1705-BoldAnchor-soft-reset-process-layer
  created: '2026-04-17T17:05:00+00:00'
  labels:
    area: cli
    type: feature
  updated: '2026-04-25T09:53:31+00:00'
spec:
  title: Soft reset that rebuilds processes/hooks/skills while preserving content
    entities
  state: backlog
  type: feature
  priority: medium
  description: |
    ## Idea

    Today `aibox` offers only a *hard* reset (wipe and re-install from
    scratch). We need a complementary **soft reset** that re-synchronises
    the process layer — skills, processes, schemas, state machines, hooks,
    AGENTS.md, MCP wiring, compliance rules — without touching project
    content that the user and agents have accumulated.

    ## Preserve

    - `context/workitems/` (backlog, tasks, bugs)
    - `context/decisions/` (decision records)
    - `context/notes/` (notes)
    - `context/artifacts/` (artefacts)
    - `context/discussions/` (discussion entities)
    - `context/logs/` (event log, session handovers)
    - `context/research/` (research)
    - `context/migrations/applied/` and `context/migrations/INDEX.md`
    - Anything custom the user has dropped under `context/` that is not
      a processkit-installed file
    - `.aibox-home/` (runtime seed + agent credentials)

    ## Rebuild (with pre-reset backup)

    - `context/skills/**`
    - `context/processes/**`
    - `context/schemas/**`
    - `context/state-machines/**`
    - `context/templates/processkit/**`
    - `context/roles/`, `context/actors/`, `context/bindings/` template
      actors (user clones preserved)
    - `AGENTS.md` and provider pointers (`CLAUDE.md`, …)
    - `.claude/settings.json`, `.codex/hooks.json`, `.cursor/hooks.json`,
      `.cursor/rules/processkit-compliance.md`, `.aider.conf.yml`
    - `.devcontainer/` generated files (delegates to existing sync path)

    ## Behaviour

    - Before overwriting, snapshot the to-be-replaced subtree under
      `context/.backups/<ISO-timestamp>/` so a user can diff / roll back.
    - Use the same three-way diff the existing sync uses — but with the
      "take upstream" resolution strategy on every conflict (no migration
      doc generation).
    - CLI: `aibox reset --soft` (pair with the existing hard reset flag).
    - Print a summary of what was overwritten vs kept vs backed up.

    ## Why

    - Recovering from a corrupted or drifting process layer without
      losing the accumulated body of work.
    - Bringing old projects up to the latest process conventions when a
      full migration path is overkill.
    - Safer onboarding for teams who want to realign without paying the
      cost of a full hard reset.

    ## Open questions

    - How do we detect "user-authored files under context/" that are not
      in any processkit manifest (keep them vs. stash them)?
    - Should the backup expire automatically after N days?
    - Does this belong in `aibox` (container/process layer) or in a
      processkit skill (because it manipulates processkit content)?
---
