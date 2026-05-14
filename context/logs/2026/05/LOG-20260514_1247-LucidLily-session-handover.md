---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260514_1247-LucidLily-session-handover
  created: '2026-05-14T12:47:45+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-14T12:47:45+00:00'
  summary: Session handover — v0.26.0 minor release shipped end-to-end (Linux+macOS+GHCR+docs);
    only post-release tmux.separators block drift left in working tree
  actor: claude-opus-4-7-1m
  details:
    session_date: '2026-05-14'
    current_state: 'v0.26.0 fully published: GitHub release with both Linux (aarch64/x86_64)
      and macOS binaries, container images on ghcr.io, gh-pages refreshed, runtime
      smoke green. Working tree has one uncommitted addition (aibox.toml gained a
      `[customization.tmux.status.separators]` block from the release-host phase''s
      `aibox apply` regenerate — clean post-release drift, not a regression). One
      unrelated stash from before this branch (`pre-v0.25.14-release-unrelated-dirty-state`)
      is still present. 951 unit + 101 Tier-1+3 e2e + Tier-2 SSH companion suite all
      green at v0.26.0.'
    open_threads:
    - BACK-20260514_0924-ActiveSummit (in-progress) — tmux layout chooser. Implemented
      + shipped in v0.26.0. State could move to 'done' next session.
    - BACK-20260514_0925-VastHare (in-progress) — tmux theme switch. Implemented +
      shipped in v0.26.0. State could move to 'done' next session.
    - 'Working tree: `aibox.toml` has an uncommitted `[customization.tmux.status.separators]`
      block addition from `aibox apply` during release-host. Either commit as `chore:
      record post-release aibox.toml drift` or revert if undesired in the canonical
      config.'
    - Stash@{0} `pre-v0.25.14-release-unrelated-dirty-state` predates this branch
      — original author should `git stash show -p stash@{0}` and decide whether to
      drop or apply.
    - 'Phase-2 follow-up known-good: `release-runtime-smoke` logs at `dist/release-smoke/v0.26.0/`.
      Worth scanning for any non-fatal warnings the next session may want to triage.'
    - Phase-3 model-provider segment work (admin usage rollup) is opt-in but only
      Anthropic + OpenAI plugins are fully wired; Mistral/DeepSeek/Cohere/xAI stubs
      are forward-compatible but inactive. Could be a follow-up minor.
    - Tier-2 visual_keybindings suite is now reliably green, but it took 3 separate
      fixes (tmux vim-style `"` comment aborting conf, `set <M-:>=^[:` consuming Esc-colon,
      `set <A-Left>=` E518 in vim 9.x). Worth a post-mortem note in the vim/tmux config-emission
      code so future contributors don't reintroduce them.
    next_recommended_action: 'Decide on the uncommitted `aibox.toml` separators block:
      either commit it as `chore: record post-release config drift` (recommended —
      it''s the canonical layout the new CLI emits) or run `git checkout aibox.toml`
      to keep the working tree pristine. Then transition the two in-progress WorkItems
      (BACK-20260514_0924-ActiveSummit + BACK-20260514_0925-VastHare) from `in-progress`
      → `done` via `transition_workitem`.'
    branch: main
    commit: 8ea132a5
    release_published: v0.26.0
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.26.0
    behavioral_retrospective:
    - 'Initially dismissed the companion as ''no companion in this sandbox'' after
      my first SSH probe attempt failed; user had to correct me. Lesson: when an SSH
      key path resolves under a $PWD-relative form, retry from the canonical /workspace
      path before declaring unavailability. Encoded as: ''Probe companion at /workspace/.aibox-e2e-runner-home/.ssh/id_ed25519
      with `testuser@aibox-e2e-testrunner` before concluding it''s offline.'''
    - 'Spent ~3 release iterations chasing 13 → 1 → 0 tier-2 e2e failures. The root
      cause of 12/13 was a single `"` (vim-style) comment in the tmux.conf template
      that aborted the parser silently. Filed as a hard rule: tmux.conf template lines
      must use `#` comments. The vt100 Tier-3 suite I added would have caught the
      resulting status-bar mis-render (per-row surface assertion), but the bug only
      surfaced because tier-2 caught the secondary symptom (window-index drift). Both
      tiers earn their keep.'
    - 'Vim 9.x rejects `:set <A-Left>=…` with E518 — I assumed it worked because the
      syntax is widely documented. Lesson encoded in the new vimrc comment + unit
      test: never re-introduce `set <A-…>=…`; rely on terminal auto-mapping via xterm-keys,
      or use `set <M-letter>=…` (which DOES work).'
    - User asked twice to `commit and push` before I integrated the request fully
      into the release flow. Should have noticed `commit and push` is implicit in
      the release pipeline (push-main step) and not asked them again. Minor — no lost
      work.
---
