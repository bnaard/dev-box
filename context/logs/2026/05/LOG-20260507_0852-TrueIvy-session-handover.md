---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260507_0852-TrueIvy-session-handover
  created: '2026-05-07T08:52:29+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-07T08:52:29+00:00'
  summary: Investigated devcontainer startup PID/CPU explosion traced to stale Zellij native status plugin session state and overlapping aibox-status subprocess refreshes.
  actor: codex
  subject: aibox startup
  subject_kind: devcontainer
  details:
    root_causes:
    - Native Zellij status plugin scheduled a new aibox-status --plugin-json command on every timer even when a prior command was still blocked, allowing overlapping sh/bash/git/tr/sed subprocess trees.
    - Changing customization.zellij_status to shell/hidden did not take effect because .aibox-home Zellij serialized session metadata still referenced file:/usr/local/share/aibox/zellij/aibox-status.wasm.
    - 'Generated runtime had drifted around cache/user startup: normal startup needed /home/aibox/.cache mounted as a parent cache and plain attach paths needed to run as user aibox, not root.'
    measures_taken:
    - Set customization.zellij_status.mode to hidden for this project.
    - Removed live stale Zellij native permission caches and saved session metadata under .aibox-home/.cache/zellij/contract_version_1/session_info/aibox.
    - Added in-flight guard to images/base-debian/zellij-plugins/aibox-status/src/zellij_plugin.rs so native status refreshes cannot overlap.
    - Made non-native status cleanup remove managed native Zellij permission caches.
    - 'Generated compose now includes user: aibox; project override adds pids_limit: 1200 and no longer depends on the E2E companion for normal startup.'
    - scripts/maintain.sh attach now execs Zellij as aibox with HOME, SHELL, and XDG_CACHE_HOME set.
    verification:
    - cargo test generate::tests::compose passed 19 compose tests.
    - cargo test seed::tests::cleanup_removes_native_zellij_permission_cache_when_status_not_native passed.
    - cargo test seed::tests::managed_runtime_files_omit_zellij_permission_cache_for_shell_status passed.
    - cargo test for aibox-zellij-status default feature passed 9 tests.
    - cargo build --manifest-path images/base-debian/zellij-plugins/aibox-status/Cargo.toml --release --target wasm32-wasip1 --features zellij passed after installing wasm32-wasip1 target.
    - Sanity scan found no remaining aibox-status/native status references in .aibox-home/.config/zellij or .aibox-home/.cache after cleanup.
---
