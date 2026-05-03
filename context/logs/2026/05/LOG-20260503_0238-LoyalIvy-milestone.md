---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260503_0238-LoyalIvy-milestone
  created: '2026-05-03T02:38:49+00:00'
spec:
  event_type: milestone
  timestamp: '2026-05-03T02:38:49+00:00'
  summary: 'Fixed post-recreate runtime issues: preserve Compose init, remove disabled
    lazygit from generated runtime, and refresh ai layout without lazygit tab.'
  actor: Codex
  subject: devcontainer runtime verification and fixes
  subject_kind: runtime
  details:
    verification_before:
      pid1: sleep infinity
      memory_events_oom_kill: 0
      bwrap: 1181 zombies, 2 sleeping at latest count
      lazygit_present_in_live_container: true
      workspace_manifest_git_ui_tools:
      - gh
    changes:
    - Changed generated devcontainer.json overrideCommand to false so devcontainer
      clients preserve Compose command/init=true.
    - Updated git-ui addon rendering to purge lazygit when explicitly disabled, covering
      older base images that shipped lazygit in the base layer.
    - Made Zellij lazygit tab/config generation honor effective addon tool state and
      omit lazygit when disabled.
    - Regenerated .devcontainer files and .aibox-home/context runtime templates with
      AIBOX_ADDONS_DIR=/workspace/addons cargo run --manifest-path cli/Cargo.toml
      -- apply --no-container.
    validation:
    - 'cargo test --manifest-path cli/Cargo.toml generate::tests:: -- --nocapture
      passed.'
    - 'cargo test --manifest-path cli/Cargo.toml seed::tests:: -- --nocapture passed.'
    - cargo test --manifest-path cli/Cargo.toml workspace_manifest -- --nocapture
      passed before the final generate test setup patch.
    - 'cargo test --manifest-path cli/Cargo.toml addons::tests:: -- --nocapture passed
      before the final generate test setup patch.'
    - cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings passed.
    - AIBOX_ADDONS_DIR=/workspace/addons cli/target/debug/aibox describe workspace-manifest
      -o json shows git-ui tools only gh.
    remaining_runtime_boundary: Current live container still has PID 1 sleep infinity
      and existing bwrap zombies; a recreate with the regenerated devcontainer config
      is required before PID 1 and zombie counts can improve.
---
