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
  title: Soft reset that rebuilds processes/hooks/skills while preserving content entities
  state: backlog
  type: feature
  priority: medium
  description: "## Idea\n\nToday `aibox` offers only a *hard* reset (wipe and re-install from\nscratch). We need a complementary **soft reset** that re-synchronises\nthe process layer \u2014 skills, processes, schemas, state machines, hooks,\nAGENTS.md, MCP wiring, compliance rules \u2014 without touching project\ncontent that the user and agents have accumulated.\n\n## Preserve\n\n- `context/workitems/` (backlog, tasks, bugs)\n- `context/decisions/` (decision records)\n- `context/notes/` (notes)\n- `context/artifacts/` (artefacts)\n- `context/discussions/` (discussion entities)\n- `context/logs/` (event log, session handovers)\n- `context/research/` (research)\n- `context/migrations/applied/` and `context/migrations/INDEX.md`\n- Anything custom the user has dropped under `context/` that is not\n  a processkit-installed file\n- `.aibox-home/` (runtime seed + agent credentials)\n\n## Rebuild (with pre-reset backup)\n\n- `context/skills/**`\n- `context/processes/**`\n- `context/schemas/**`\n\
    - `context/state-machines/**`\n- `context/templates/processkit/**`\n- `context/roles/`, `context/actors/`, `context/bindings/` template\n  actors (user clones preserved)\n- `AGENTS.md` and provider pointers (`CLAUDE.md`, \u2026)\n- `.claude/settings.json`, `.codex/hooks.json`, `.cursor/hooks.json`,\n  `.cursor/rules/processkit-compliance.md`, `.aider.conf.yml`\n- `.devcontainer/` generated files (delegates to existing sync path)\n\n## Behaviour\n\n- Before overwriting, snapshot the to-be-replaced subtree under\n  `context/.backups/<ISO-timestamp>/` so a user can diff / roll back.\n- Use the same three-way diff the existing sync uses \u2014 but with the\n  \"take upstream\" resolution strategy on every conflict (no migration\n  doc generation).\n- CLI: `aibox reset --soft` (pair with the existing hard reset flag).\n- Print a summary of what was overwritten vs kept vs backed up.\n\n## Why\n\n- Recovering from a corrupted or drifting process layer without\n  losing the accumulated body\
    \ of work.\n- Bringing old projects up to the latest process conventions when a\n  full migration path is overkill.\n- Safer onboarding for teams who want to realign without paying the\n  cost of a full hard reset.\n\n## Open questions\n\n- How do we detect \"user-authored files under context/\" that are not\n  in any processkit manifest (keep them vs. stash them)?\n- Should the backup expire automatically after N days?\n- Does this belong in `aibox` (container/process layer) or in a\n  processkit skill (because it manipulates processkit content)?"
---
