---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260512_1900-CheerfulEagle-session-handover
  created: '2026-05-12T19:00:01+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-12T19:00:01+00:00'
  summary: Session handover - runtime-home mount architecture fix implemented and
    validated
  actor: codex
  subject: aibox runtime-home mounts
  subject_kind: implementation
  details:
    session_date: '2026-05-12'
    current_state: 'Implemented an uncommitted source-level fix for derived aibox
      projects so generated containers mount broad writable runtime-home XDG parents
      from project-local .aibox-home: .config, .cache, and .local, while keeping .ssh
      read-only. Added centralized cli/src/runtime_home.rs as the source of truth
      for runtime-home mounts, scaffold directories, writable destinations, legacy
      destination checks, and extra-volume conflict detection. Generated docker-compose
      now uses that contract instead of hardcoded per-tool mount fragments. Doctor/start
      now inspect live mount tables and report stale, missing, legacy, or read-only
      runtime-home mounts. PowerKit render helpers now use managed XDG cache. apply/theme
      sync now refreshes Yazi git/status plugin files so lost Yazi git symbols are
      restored in derived projects. Entry point re-owns /tmp/aibox after UID/GID remap
      for uv/tmp cache writability.'
    git:
      branch: main
      head: 'b4d3fcd6 docs: record v0.25.11 session handover'
      remote: origin/main aligned
      working_tree: dirty WIP, not committed
      modified_files:
      - aibox.toml
      - cli/src/config.rs
      - cli/src/container.rs
      - cli/src/doctor.rs
      - cli/src/generate.rs
      - cli/src/main.rs
      - cli/src/runtime.rs
      - cli/src/seed.rs
      - cli/src/templates/docker-compose.yml.j2
      - cli/src/tmux/status.rs
      - cli/tests/e2e/config_coverage.rs
      - cli/tests/e2e/infra/mock-docker.sh
      - images/base-debian/config/entrypoint.sh
      untracked_files:
      - cli/src/runtime_home.rs
      note: aibox.toml contains recovered unrelated prompt change arrow -> pastel;
        left untouched.
    validation:
      passed:
      - cargo test --manifest-path cli/Cargo.toml
      - cargo clippy --all-targets -- -D warnings
      - git diff --check
      focused_checks:
      - runtime_home tests
      - compose mount tests
      - PowerKit cache helper test
      - Yazi stale git plugin refresh test
      not_run:
      - cargo audit
      - release flow
      - live aibox apply/recreate in this repo
    architecture_judgement: After fresh research, broad-parent mounts are appropriate
      for aibox because the bind source is project-local gitignored .aibox-home, not
      the user's real home. Separate mounts remain the safer general pattern for real
      host config/secrets. The current design keeps SSH read-only and blocks extra
      volumes from shadowing managed runtime paths.
    next_recommended_actions:
    - Review the WIP diff, especially runtime_home.rs and generated compose changes.
    - Decide whether to keep or revert the unrelated aibox.toml prompt change before
      commit.
    - Commit the runtime-home fix.
    - Create a patch release so derived projects receive the fix, then tell users
      to run aibox apply and recreate containers; rebuild alone is insufficient for
      old mount tables.
    - Optionally run live aibox apply/delete runtime/up in this repo after commit
      if the user wants to repair the current container too.
---
