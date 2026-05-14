---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260514_1916-FitMeadow-session-handover
  created: '2026-05-14T19:16:53+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-05-14T19:16:53+00:00'
  summary: Session handover — v0.26.1 + v0.26.2 + v0.26.3 patch releases shipped end-to-end
    (Phase 1 + Phase 2 + GHCR verified); 161 orphaned skill files left in working
    tree from earlier sync.
  actor: claude-opus-4-7-1m
  details:
    session_date: '2026-05-14'
    current_state: 'Three patch releases shipped today (v0.26.1, v0.26.2, v0.26.3),
      all with Phase 1 + Phase 2 complete except v0.26.1 which has Phase 1 only and
      is superseded by v0.26.2. GHCR confirmed live for v0.26.2 + v0.26.3 + latest
      tag; verify_release_images_in_ghcr guard from v0.26.1 passed cleanly during
      both v0.26.2 and v0.26.3 host-side runs. End-to-end sanity check: v0.26.3 binary
      now resolves ''Resolved aibox image latest -> v0.26.3'' (pre-fix it was stuck
      at v0.25.12 due to GHCR pagination cap). 966/966 unit tests + 123 Tier-2 E2E
      + full Visual matrix (3 tiers) + Tier-3 Starship all green. Branch main is in
      sync with origin (0 ahead, 0 behind, HEAD at 6753e61b ''chore: refresh generated
      runtime for v0.26.3'').'
    open_threads:
    - 'Working tree has 161 unstaged deletes under context/skills/{data-ai,design,devops,documents,engineering,product}/
      - these are processkit v0.26.7+ skill categories NOT in this project''s aibox.toml
      [skills].include allowlist. prune_unselected_live_skills (cli/src/content_init.rs:651)
      intentionally strips them on aibox apply. Either commit the deletes as ''chore:
      prune unselected processkit skill categories'' (correct) or expand [skills].include
      in aibox.toml to keep them. The original sync commit 249cc8b0 (processkit v0.26.7
      -> v0.26.9) shouldn''t have committed these in the first place; recommend just
      committing the deletes.'
    - BACK-20260514_0924-ActiveSummit (in-progress) - tmux layout chooser. Implemented
      + shipped across v0.26.0-v0.26.3. State could move to done.
    - BACK-20260514_0925-VastHare (in-progress) - tmux theme switch. Implemented +
      shipped across v0.26.0-v0.26.3. State could move to done.
    - 'BACK-20260514_1752-EarnestMoss (backlog) - rewrite live tmux layout-switch
      + theme-switch e2e tests with asciinema capture (replaces the broken visual_rendered_tmux/yazi
      suites deleted in v0.26.2). reference: cli/tests/e2e/visual.rs::record_tmux
      + tmux_driver.'
    - BACK-20260514_1902-ShinyLake (backlog) - CLOSED BY v0.26.3. fetch_latest_image_version
      GHCR pagination fix landed; recommend transition to done next session.
    - Stash@{0} pre-v0.25.14-release-unrelated-dirty-state predates this branch -
      original author should git stash show -p stash@{0} and decide whether to drop
      or apply. Still present.
    - v0.26.1 release on GitHub has Phase 1 (Linux binaries) only - no macOS binaries,
      no GHCR image. v0.26.2 supersedes it. Optionally mark v0.26.1 release as a pre-release
      on GitHub to deprioritize visually, but no functional issue (users on 'latest'
      jump straight to v0.26.3).
    - pk-doctor still surfaces 21 accepted-policy WARNs (20 lexical-token-ambiguous
      + 1 AGENTS.md pk-commands managed-block drift) - all decision-recorded as accepted
      via DEC-20260514_1305-MellowBison and DEC-20260514_1318-SoundHawk. No action
      needed.
    next_recommended_action: 'Commit the 161 working-tree deletes under context/skills/{data-ai,design,devops,documents,engineering,product}/
      - the prune_unselected_live_skills behavior is correct, those categories aren''t
      in this project''s [skills].include allowlist. Run `git add -u context/skills/
      && git commit -m ''chore: prune unselected processkit skill categories from
      v0.26.7 sync'' && git push`. After that, run aibox apply once and verify a fresh
      apply has a clean working tree.'
    branch: main
    commit: 6753e61b
    release_published: v0.26.3
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.26.3
    behavioral_retrospective:
    - 'Initially misdiagnosed the recurring ''Repairing processkit template mirror''
      as a missing-then-restored hash, when the actual cause was that the install_hash
      fingerprint included agent-mutable per-skill config/ files. User would have
      spent the rest of v0.26.1 chasing the wrong fix if I hadn''t traced the install
      path carefully. Lesson encoded in mcp_registration::compute_processkit_install_fingerprint
      comment + the v2: format-tag scheme so future format changes don''t repeat the
      silent-upgrade problem.'
    - 'User explicitly told me ''I haven''t seen you running all the tests'' after
      I shipped v0.26.1 without running Tier 3. That was right: I''d been skipping
      e2e/visual/render on every release with --skip e2e,visual. Lesson: always run
      the full test pipeline at least once per release (even if partial-skip for iterative
      fixes inside a release); on v0.26.2 + v0.26.3 I ran Tier 2 (123 tests) + Visual
      matrix (3 tiers) + Tier 3 Starship in addition to the unit suite. Without that
      pass I would not have caught the labels-duplicate-header bug nor the broken
      Tier 3 capture-pane assumption.'
    - 'Spent ~hour digging into the Tier 3 tmux failures before noticing the redundant
      asciinema suite in visual.rs covers the same regression class correctly. User
      caught it: ''consider asciinema as solution and check, if existing asciinema
      based tests already cover this tests''. Lesson: before rewriting a broken test,
      grep the repo for an existing one that does the same thing - especially in projects
      with multiple e2e tiers.'
    - 'The Tier 3 capture-pane / status-bar mismatch had been latent since v0.26.0
      (compat note explicitly claimed Tier 3 catches ''status line 1 silently empty'',
      but capture-pane cannot see status bar). Suggests the v0.26.0 release process
      either skipped Tier 3 or its ''passing'' was vacuous (no asserts firing because
      captures were empty). Filed for follow-up: the BACK-20260514_1752-EarnestMoss
      rewrite should also audit which historical regressions Tier 3 ACTUALLY caught
      vs. claimed to catch.'
---
