---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_1515-SilentAsh-v0-25-6-cross-cutting-cleanup
  created: '2026-05-08T15:15:56+00:00'
  updated: '2026-05-08T16:04:45+00:00'
spec:
  title: v0.25.6 cross-cutting cleanup, doctor/test/security closure, and aibox.toml streamlining
  state: accepted
  decision: 'For v0.25.6 we adopt: (1) a per-category cleanup-variant policy enforced by `aibox apply` and surfaced by `aibox doctor`; (2) a complete zellij eradication (scorched earth) with no surviving config, cache, mount, or code reference; (3) closure of all six doctor coverage gaps and all six identified e2e/companion test gaps; (4) integrity hardening of every addon download path and the published scripts/install.sh; (5) an aibox.toml dedup of the [skills] block to a single uncomment-to-enable list while keeping the heavy inline commenting; (6) the seed.rs tmux/PowerKit functions split into cli/src/tmux/. Implemented as 6 WorkItems executed by parallel subagents next session: BR-CLEANUP-ARCH (foundational, blocks others), BR-ZELLIJ-EXCISE, BR-DOCTOR-GAPS, BR-TEST-GAPS, BR-SEC-HARDEN, BR-CODE-QUALITY.'
  context: 'Out-of-band code & UX review (2026-05-08) of aibox at /workspace against six reported runtime/UX symptoms (lazygit fetch in container build; yazi terminal-response timeout flash; tmux mode=powerline naming + two-line layout request; tmux socket connect error on `aibox up --forget-tmux-state`; tmux-powerkit status not rendering; "stale state never gets removed by aibox apply") found that bugs 2, 4, and 5-cosmetic-half are already fixed in v0.25.5 but the user was building with the v0.25.4 host CLI and runtime image v0.25.2, plus a real architectural defect: cross-version `aibox apply` does not auto-overwrite drifted managed runtime files (notably tmux.conf) and the lockfile records no previous addon/tool/harness selection, so general "purge-on-removal" cannot be implemented without it. Doctor, e2e tests, and addon download integrity have specific gaps that allowed each symptom to ship.'
  rationale: "Per-category cleanup-variant assignment (owner-approved):\n\nVARIANT 1 \u2014 HARD PURGE (no question to user):\n- Generated managed runtime files when content is recognizably stale or corrupted: .aibox-home/.config/tmux/tmux.conf, aibox-session.sh, .config/tmux/layouts/*.sh \u2014 100% generated, no user investment.\n- Addon binaries baked into the image layer when the addon/tool is disabled \u2014 emit purge blocks in generated Dockerfile for kubectl/helm/aws-cli/azure-cli/gcloud/opentofu/packer/audio-voice/preview-*/yazi-omp, generalizing the existing git-ui pattern.\n- PowerKit + tmux plugin caches: .aibox-home/.cache/tmux-powerkit/, .tmux/plugins/<plugin> for any plugin no longer referenced by the generated tmux.conf.\n- Legacy zellij surface: every file path, env var, generator branch, addon hint, template reference, mount, config, cache, helper binary, decision/comment reference. Zellij was fully replaced by tmux; nothing of zellij must survive.\n- Files for processkit\
    \ versions older than the lockfile's processkit_version baseline once a Migration entity confirms migration applied.\n\nVARIANT 2 \u2014 OPT-IN HARD PURGE (default: leave + migration note):\n- Per-harness directories under .aibox-home/ when a harness is disabled (.gemini, .codex, .aider, .continue, .opencode/plugins/) \u2014 may carry auth tokens or personal config.\n- Addon-related runtime config dirs under .aibox-home/.config/<tool>/ when the addon is disabled (.config/aws/, .config/gh/) \u2014 may carry user credentials/customizations.\n- Per-harness MCP config files when the harness is disabled (.mcp.json, .cursor/mcp.json, .gemini/settings.json, .codex/config.toml, .continue/mcpServers/*.json).\n- Surfaces as [apply].purge_disabled_harness_state = false (default) in aibox.toml; when true, aibox apply hard-deletes; when false, emits a Migration entity in context/migrations/pending/ describing exactly what would be removed.\n\nVARIANT 3 \u2014 MIGRATION NOTE ONLY (derived project's\
    \ agent + user decide):\n- Anything aibox could classify as drifted but possibly intentional user customization \u2014 apply emits a pending Migration with per-file recommendation; the derived project's agent surfaces it on /pk-resume and /pk-doctor and walks it with the user via migration-management.\n\nLockfile schema must be extended to record the previous [addons.<name>.tools] selection and previous harness selection so apply can compute a removal diff (prerequisite for Variants 1 and 2). Backfill on first apply via auto-generated Migration.\n\naibox.toml [skills] dedup direction: replace the dual enabled[]/disabled[] block with a single deduplicated catalog list where each skill is one line. Default state per skill encoded by whether the line is uncommented. Keep heavy inline commenting (one-line description + category) so users can configure without reading docs. Streamline comments only where demonstrably wrong or outdated.\n\nPowerKit two-line status line: keep current hard-coded\
    \ element ordering (matches owner's requested order byte-for-byte). Defer ordered-vector schema (order = [...]) until a user actually requests reordering; document as fixed-order in a follow-up note.\n"
  alternatives:
  - option: Ship only the v0.25.5 host CLI bump and let users migrate gradually
    rejected_because: Bug 5 (cross-version sync preserves corrupted tmux.conf) means even with v0.25.5 the user's running project never recovers. The architectural fix is non-negotiable.
  - option: Always hard-purge everything (Variant 1 universal)
    rejected_because: Per-harness dirs may hold auth tokens; addon config dirs may hold user customizations. Silent destruction is a trust breach.
  - option: Always migration-note (Variant 3 universal)
    rejected_because: Generated managed files (tmux.conf, layouts) and legacy zellij artifacts have zero user investment; migration prompts there are pure friction.
  - option: Add ordered slot vectors (line1_right=[...]) to TmuxStatusElementsSection now
    rejected_because: Owner's requested order matches the hard-coded order byte-for-byte. Defer until a user wants reordering.
  consequences: "POSITIVE:\n- aibox apply becomes the trustworthy single command to make a project converge on its toml.\n- Doctor checks for legacy aliases, drift signatures, missing PowerKit plugin tree, lockfile-vs-CLI skew, and yazi/tmux startup hygiene give users actionable warnings.\n- E2E coverage of disabled-tool absence, PowerKit rendering, --forget-tmux-state attach, and corrupted-tmux.conf recovery closes the regression surface.\n- Addon supply-chain risk reduced via SHA-256 verification on every binary download; replace curl | bash for ai-hermes; checksum verification in scripts/install.sh.\n- aibox.toml halves on the [skills] section while keeping discoverability.\n\nNEGATIVE / RISKS:\n- Lockfile schema bump requires a migration: existing v0.25.x lockfiles need a previous_selection backfill on first apply.\n- Variant 2 default (leave + migration note) means users who disable a harness still see leftover dirs unless they opt in \u2014 must be communicated in release notes.\n\
    - Zellij excision is destructive; migration message must be loud for any project still on a pre-tmux-cut version.\n- seed.rs split is mechanically large but low risk if guarded by the new e2e tests.\n"
  deciders:
  - TEAMMEMBER-20260422_0832-MigratedMember-cora
  decided_at: '2026-05-08T15:15:56+00:00'
  related_workitems:
  - BACK-20260508_1516-BrightStream-stale-state-cleanup-architecture-foundational
  - BACK-20260508_1517-TrueBrook-zellij-scorched-earth-complete-excision
  - BACK-20260508_1517-SnowyWillow-doctor-coverage-gap-closure-work
  - BACK-20260508_1518-KeenBison-e2e-companion-test-gap-closure
  - BACK-20260508_1518-HonestAnt-addon-installer-security-hardening-checksum
  - BACK-20260508_1519-LuckyLily-code-quality-aibox-toml-seed-split
  - BACK-20260508_1519-PluckyThorn-release-host-orchestration-rollout-cutover
  - BACK-20260508_1603-QuietCedar-status-bar-visual-rework-powerline
  - BACK-20260508_1603-SilentEagle-log-pane-reader-lnav-integration
  - BACK-20260508_1604-GrandWillow-yazi-vim-pane-hard-cut
---
