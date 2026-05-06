---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260506_0311-LoyalQuail-session-handover
  created: '2026-05-06T03:11:10+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-06T03:11:10+00:00'
  summary: 'Handover: Phase 1 now wires SSH companion E2E tests, generated-runtime
    probes were added, and the active companion image is stale and must be rebuilt
    before the full suite can pass.'
  actor: codex
  subject: aibox companion E2E release gate wiring
  subject_kind: release-test-workflow
  details:
    current_state:
    - 'Implemented but not committed: Tier 2 SSH companion E2E is wired into scripts/maintain.sh
      release Phase 1 via cmd_test_e2e running cargo test --features e2e --test e2e.'
    - Added cli/tests/e2e/runtime_generated.rs with generated Yazi/lazygit/status
      helper probe and native Zellij key/status plugin asciinema probe.
    - Added lifecycle_apply_starts_generated_container to exercise init -> apply ->
      compose up -> container exec probe -> compose down on the companion runtime.
    - Updated .devcontainer/Dockerfile.e2e to pin Zellij 0.44.2, Yazi 26.5.6, Starship
      1.25.1 and add slirp4netns.
    - Updated .devcontainer/docker-compose.override.yml so the companion has seccomp=unconfined,
      SYS_ADMIN, NET_ADMIN, and /dev/net/tun for rootless Podman.
    - Added E2E runner deployment for aibox-status and aibox-status-toggle plus an
      early stale-companion version guard.
    - 'Fixed compose override handling so comment-only docker-compose.override.yml
      files are not passed to compose providers; scaffold now writes services: {}
      to make new overrides valid YAML.'
    - Added seed.rs guard that generated dev/focus/cowork/cowork-swap/browse/ai layouts
      start primary lazygit and bash tabs eagerly, not suspended.
    - Updated E2E and maintenance docs for Phase 1 companion E2E and host-side release-runtime-smoke.
    verification:
    - cargo test --manifest-path cli/Cargo.toml --features e2e --test e2e --no-run
      passed.
    - generated_runtime_yazi_lazygit_and_status_are_usable passed before adding the
      stale-companion version guard.
    - 'generated_runtime_zellij_status_plugin_renders_key_and_status_rows failed on
      the live companion with Zellij plugin load errors; follow-up inspection showed
      the companion was stale: zellij 0.44.1 and Yazi 26.1.22.'
    - companion_is_reachable now fails early with a clear stale-image message expecting
      zellij 0.44.2 and Yazi 26.5.6.
    - cargo test --manifest-path cli/Cargo.toml generated_layouts_start_primary_runtime_tabs_immediately
      passed.
    - cargo test --manifest-path cli/Cargo.toml compose_file_args_skip_comment_only_override
      passed.
    - bash -n scripts/maintain.sh scripts/release-runtime-smoke.sh passed.
    - git diff --check passed.
    next_actions:
    - Rebuild/recreate the aibox-e2e-testrunner companion service from .devcontainer/Dockerfile.e2e
      on the host or by rebuilding the devcontainer sidecar.
    - Rerun ./scripts/maintain.sh test-e2e after the companion is refreshed.
    - If the refreshed companion passes Tier 2, run normal cargo test/clippy/audit
      as needed, then commit the current changes.
    - If the native Zellij plugin test still fails after rebuild, inspect /tmp/zellij-*/zellij-log/zellij.log
      and the generated recording.cast; the test workspace name is runtime-generated-zellij.
    working_tree:
      modified:
      - .devcontainer/Dockerfile.e2e
      - .devcontainer/docker-compose.override.yml
      - cli/src/context.rs
      - cli/src/runtime.rs
      - cli/src/seed.rs
      - cli/tests/e2e/lifecycle.rs
      - cli/tests/e2e/main.rs
      - cli/tests/e2e/runner.rs
      - docs-site/docs/contributing/e2e-tests.md
      - docs-site/docs/contributing/maintenance.md
      - scripts/maintain.sh
      untracked:
      - cli/tests/e2e/runtime_generated.rs
      - context/decisions/DEC-20260506_0243-HonestOtter-run-generated-runtime-smoke-during-host.md
      - context/logs/2026/05/LOG-20260506_0243-SleekOak-decision-created.md
      - scripts/release-runtime-smoke.sh
---
