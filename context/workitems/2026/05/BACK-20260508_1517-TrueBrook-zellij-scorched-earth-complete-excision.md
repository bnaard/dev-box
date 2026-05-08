---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1517-TrueBrook-zellij-scorched-earth-complete-excision
  created: '2026-05-08T15:17:19+00:00'
  labels:
    track: zellij-excise
    release: v0.25.6
spec:
  title: 'v0.25.6: Zellij scorched-earth excision'
  state: backlog
  type: task
  priority: high
  description: |
    ## Goal
    Remove every trace of zellij from the codebase, generated artifacts, runtime mounts, caches, and helper binaries. Zellij was fully replaced by tmux; nothing of zellij must survive. Variant 1 (hard purge) per DEC-20260508_1515-SilentAsh.

    ## Scope

    ### 1. Source-code excision
    - Grep for `zellij` (case-insensitive) across `/workspace/cli/src/`, `/workspace/addons/`, `/workspace/scripts/`, `/workspace/images/`, `/workspace/docs-site/`, `/workspace/schemas/`, `/workspace/cli/tests/`, `/workspace/.devcontainer/`, root markdown.
    - Known hits to start from (verified during review):
      - `cli/src/seed.rs:1070-1114` — already removes `.aibox-home/.cache/zellij/...` and `.config/zellij/config.kdl` and `.local/bin/aibox-status`. Keep these but make them unconditional (currently part of `cleanup_disabled_runtime_files`).
      - `cli/src/sync_perimeter.rs` — any zellij paths in the perimeter list.
      - `cli/src/compat.rs` — release notes mention zellij. Keep historical notes (don't rewrite history) but ensure no live code path depends on the strings.
      - `cli/src/cli.rs` — search for `zellij` in flags / value enums.
      - `cli/src/themes.rs` / `cli/src/generate.rs` / `cli/src/templates/` — any zellij-themed branches.
      - `addons/` — any addon yaml that mentions zellij in `runtime` or `description`.
      - `images/base-debian/Dockerfile` and `images/base-debian/config/` — any zellij install or config copy.
      - `cli/tests/e2e/*` and `cli/tests/integration.rs` — strip zellij assertions.
      - `docs-site/`, `README.md`, `AGENTS.md`, `CONTRIBUTING.md` — purge or migrate prose.
      - `context/templates/aibox-home/*/.config/zellij/` — leave the historical archives in place (read-only mirror) but stop emitting them in current generation.

    ### 2. Hard runtime purge on apply
    - Ensure `aibox apply` always (unconditionally) deletes from .aibox-home: `.config/zellij/`, `.cache/zellij/`, `.local/share/zellij/`, `.local/bin/aibox-status`, any `zellij` entry in `.tmux/plugins/`.
    - Add to `cli/src/seed.rs::cleanup_disabled_runtime_files` an unconditional `cleanup_legacy_zellij_files` step (separate from disabled-tool cleanup).

    ### 3. Doctor surface
    - `aibox doctor` errors loudly (not warns) if any zellij artifact is detected on disk after apply has run, with actionable remediation: "run `aibox apply` from a v0.25.6+ host CLI; if the warning persists, remove `<paths>` manually."

    ### 4. Compose / mounts
    - Audit `cli/src/templates/docker-compose.yml.j2` for any zellij-related volume mount or env var.

    ## Acceptance criteria
    - `grep -rli "zellij" /workspace/cli/src /workspace/addons /workspace/images /workspace/scripts` returns 0 hits except (a) `compat.rs` historical release notes and (b) the `cleanup_legacy_zellij_files` function name itself.
    - New e2e: start container with a fake `.aibox-home/.config/zellij/config.kdl` and `.cache/zellij/cache.bin`, run `aibox apply`, assert files are gone and `aibox doctor` returns clean.
    - New doctor test: with surviving zellij artifact, doctor reports an error.

    ## Dispatch hint for next session
    One general-purpose subagent. Sequential file-by-file edit pass; provide the `grep -rli` output to the agent and ask for a careful per-occurrence judgement (delete vs. migrate). After done, the agent runs the test suite once.
---
