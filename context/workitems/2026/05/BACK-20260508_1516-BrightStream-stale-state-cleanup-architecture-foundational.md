---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1516-BrightStream-stale-state-cleanup-architecture-foundational
  created: '2026-05-08T15:16:52+00:00'
  labels:
    track: cleanup-arch
    release: v0.25.6
    blocks_others: true
spec:
  title: 'v0.25.6: Stale-state cleanup architecture (foundational)'
  state: backlog
  type: epic
  priority: high
  description: |
    ## Goal
    Make `aibox apply` the trustworthy single command that makes a project converge on its `aibox.toml`. Implement the per-category cleanup-variant policy approved in DEC-20260508_1515-SilentAsh.

    ## Scope (next-session implementation)

    ### 1. Lockfile schema bump (PREREQUISITE for everything else)
    - File: `cli/src/lock.rs:60-86`
    - Add `[addons.<name>.tools]` previous-selection map and previous harness selection to `AibxLockFile`.
    - Migration: on first apply with the new schema, backfill `previous_selection` from current selection and emit a Migration entity in `context/migrations/pending/` so derived projects acknowledge.

    ### 2. Cross-version managed-runtime auto-overwrite recognizer (Variant 1)
    - File: `cli/src/runtime_sync.rs:119-198`
    - Add `live_matches_historical_managed_tmux_file` analogue to the existing zellij/yazi helpers.
    - Hash-recognize every archived `context/templates/aibox-home/*/.config/tmux/tmux.conf`, `aibox-session.sh`, `layouts/*.sh`.
    - Hand-coded recognizer for the v0.25.3 `off_RIGHT` corruption signature (`set -g status off` + `set -g status-right " off_RIGHT "`).
    - On match: hard-overwrite with current generated content, no user prompt.

    ### 3. Generalize purge-on-disable to all addons (Variant 1)
    - Files: `addons/tools/*.yaml` schema; `cli/src/addon_loader.rs:556-589` (`render_runtime`).
    - Pattern: every yaml may declare a `purge_template` (or compute from tools) — emit a `{% if not tools.X.enabled %}` block analogous to `git-ui.yaml:33-41` for: kubernetes (kubectl/helm/kustomize/k9s), cloud-aws, cloud-azure, cloud-gcp, infrastructure (opentofu/packer/ansible), audio-voice, preview-archive, preview-enhanced, data-preview, yazi-omp.
    - Each purge must use `dpkg-query` / `rm -f` / `pip uninstall` / `npm uninstall -g` matching how the tool is installed.

    ### 4. Per-harness state cleanup (Variant 2)
    - Files: `cli/src/seed.rs:1053-1117` (`cleanup_disabled_runtime_files`); `cli/src/mcp_registration.rs`.
    - New `[apply].purge_disabled_harness_state` toml key (default `false`).
    - When `true`: hard-delete `.aibox-home/.gemini`, `.codex`, `.aider`, `.continue`, `.opencode/plugins/`, plus `.mcp.json`, `.cursor/mcp.json`, `.gemini/settings.json`, `.codex/config.toml`, `.continue/mcpServers/*.json` for any disabled harness.
    - When `false`: emit a Migration entity in `context/migrations/pending/` describing exactly what would be removed.

    ### 5. PowerKit + tmux plugin cache purge (Variant 1)
    - Hard-purge `.aibox-home/.cache/tmux-powerkit/` on apply.
    - Walk `.aibox-home/.tmux/plugins/<plugin>` and remove any plugin no longer referenced by the generated tmux.conf.

    ### 6. Drifted-but-possibly-intentional files (Variant 3)
    - For files in the sync perimeter that match neither archived versions nor current generation, emit a Migration entity with a per-file recommendation, surfaced by /pk-resume and /pk-doctor.

    ## Acceptance criteria
    - New e2e: start with `off_RIGHT` corruption, run `aibox apply`, assert tmux.conf is clean and references PowerKit.
    - New e2e: disable `[addons.kubernetes]` after build, run `aibox apply`, assert kubectl/helm purged from generated Dockerfile and (with --no-cache) absent from image.
    - New e2e: toggle harness on/off with both `purge_disabled_harness_state` values, assert correct file cleanup vs Migration emission.
    - All existing tests still green.

    ## Dispatch hint for next session
    One general-purpose subagent for items 1-2 (lockfile + recognizer), one for items 3-5 (purge surfaces), one for item 6 (migration emission). Run in parallel after item 1 lands. ~4 agent-runs total.
---
