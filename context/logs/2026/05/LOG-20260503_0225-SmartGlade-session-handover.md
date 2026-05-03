---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260503_0225-SmartGlade-session-handover
  created: '2026-05-03T02:25:19+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-03T02:25:19+00:00'
  summary: Session handover before host runs aibox apply --no-cache and refreshes
    the devcontainer.
  actor: Codex
  subject: Post-upgrade runtime triage and patch-release readiness
  subject_kind: session
  details:
    state: Working tree contains runtime/migration cleanup changes plus lazygit opt-out
      and Yazi editor-helper fixes. No pending or in-progress migrations remain.
    changes_made:
    - Added ToolEntry.enabled support so default-enabled addon tools can be disabled
      with `{ enabled = false }`.
    - Set `[addons.git-ui.tools].lazygit = { enabled = false }` in aibox.toml.
    - Regenerated devcontainer files; Dockerfile now uses base-debian-v0.23.0 and
      no longer installs lazygit.
    - Filtered disabled tools from workspace manifest output.
    - Adjusted open-in-editor helper to wait 0.5s for suspended Zellij editor tabs;
      installed patched helper into /home/aibox/.local/bin for the current container.
    validation:
    - 'cargo test --manifest-path cli/Cargo.toml addons:: -- --nocapture passed.'
    - cargo test --manifest-path cli/Cargo.toml workspace_manifest -- --nocapture
      passed earlier in the session.
    - bash -n passed for source and live open-in-editor helpers.
    - AIBOX_ADDONS_DIR=/workspace/addons cargo run --manifest-path cli/Cargo.toml
      -- apply --no-container succeeded.
    runtime_evidence:
      memory_current: 516.7 MiB
      oom_kills: 0
      processes: 1180
      manual_proc_total: 1176
      zombies: 1157
      bwrap_total: 1151
      bwrap_zombies: 1147
      processkit_gateway_python: 1
      pid1: sleep
    important_next_steps:
    - Host will run aibox apply --no-cache and refresh/recreate the devcontainer.
    - After recreate, verify PID 1 is an init/reaper rather than sleep and that bwrap
      zombies do not accumulate.
    - Verify Yazi e opens selected file in Vim from the ai layout.
    - Verify lazygit is absent in the rebuilt image and workspace manifest still lists
      git-ui tools with only gh.
    - If fresh-container verification passes, prepare patch release v0.23.1 so derived
      projects receive lazygit opt-out and Yazi helper fixes.
    uncommitted_paths:
    - .devcontainer/Dockerfile
    - aibox.lock
    - aibox.toml
    - cli/src/addons.rs
    - cli/src/config.rs
    - cli/src/container.rs
    - cli/src/workspace_manifest.rs
    - images/base-debian/config/bin/open-in-editor.sh
    - context/migrations/* prior migration status cleanups
    - context/logs/2026/05/LOG-20260503_0203-QuietSeal-migration-applied.md
    - context/logs/2026/05/LOG-20260503_0215-FierceJay-milestone.md
    - context/migrations/20260503_0352_0.22.0-to-0.23.0.md
    - context/templates/aibox-home/0.23.0/
---
