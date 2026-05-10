---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_2320-GrandHawk-notes-zellij-references-truebrook-extension-sweep
  created: '2026-05-08T23:20:45+00:00'
  labels:
    track: zellij-excise
    release: v0.25.6
    amends: BACK-20260508_1517-TrueBrook
  updated: '2026-05-09T22:19:10+00:00'
spec:
  title: 'v0.25.6 amendment: context/notes/ Zellij references — TrueBrook scope-extension
    sweep'
  state: done
  type: task
  priority: medium
  description: |
    ## Goal

    Sweep `context/notes/` for stale Zellij references that survived TrueBrook's (BACK-20260508_1517) scoped excision because notes/ was outside TrueBrook's grep perimeter (`cli/src/`, `addons/`, `scripts/`, `images/`, `docs-site/`, `schemas/`, `tests/`, `.devcontainer/`, root markdown).

    ## Trigger

    The owner caught a stale reference in NOTE-20260411_0000-LoyalSpruce-aibox-release-process.md — "real Zellij/Yazi/asciinema sessions" describing the visual E2E system, when Zellij was fully excised in v0.25.6. The release-process doc is the canonical reference cited by AGENTS.md:139, so the staleness directly misled the release agent in this session.

    ## Scope of stale references found 2026-05-09

    ### Already fixed in this turn
    - `NOTE-20260411_0000-LoyalSpruce-aibox-release-process.md` — 4 references in:
      - Phase 0 pinned-tools table row (replaced with tmux row)
      - Phase 1 visual-E2E description ("real Zellij/Yazi/asciinema" → "real tmux/Yazi/asciinema")
      - test-e2e-visual-status trigger description ("native Zellij status/key rows, Zellij version" → "tmux PowerKit status line + key bindings, tmux version")
      - test-e2e-doc-captures output description ("Zellij logs" → "tmux capture-pane logs")

    ### Remaining live refs that warrant updating
    - **`NOTE-20260411_0000-NobleCrane-screencast-recording-and-visual.md`** (5 hits, mtime 2026-05-07, status: permanent reference). Describes the screencast architecture with Zellij as "the foreground process" requirement, `zellij action go-to-tab N` for tab switching, `pkill -x zellij` for parallel-recording isolation. The whole architecture section needs translation to the tmux equivalents (tmux send-keys, tmux select-window, tmux kill-server). **NEEDS DOMAIN-KNOWLEDGE EDIT — not pure find/replace.**

    ### Historical records — leave as-is (pre-excision context)
    - `NOTE-20260410_2335-GrandCrow-competitive-analysis-dev-environments.md` (1 hit, 2026-04-11)
    - `NOTE-20260411_0000-GentleJay-remote-development-devpod-compatibility.md` (15 hits, 2026-04-11)
    - `NOTE-20260411_0000-LuckyTiger-kubernetes-deployment-patterns-for.md` (2 hits, 2026-04-11)
    - `NOTE-20260411_0000-ProudDawn-dockerfile-best-practices.md` (2 hits, 2026-04-11)
    - `NOTE-20260411_0000-SleekFjord-cli-architecture-config-spec.md` (1 hit, 2026-04-11)
    - `NOTE-20260411_0000-SpryAnt-preview-companion-design-in.md` (9 hits, 2026-04-11)
    - `NOTE-20260503_1104-SureSwan-aibox-v0-23-2-release-handover.md` (6 hits, 2026-05-03 — pre-v0.25.0 handover; historical)

    ## Action plan

    1. **NobleCrane (must-do):** rewrite the architecture section to describe the tmux-based screencast flow. Verify the live `./scripts/maintain.sh test-e2e-visual*` scripts actually work with tmux today (if they don't, that's a real bug, not just a doc-fix).
    2. **Historical pre-excision notes (leave):** these are time-stamped records; rewriting them rewrites history. Do not touch.
    3. **Optional:** add a top-of-NOTE marker on NobleCrane indicating "post-Zellij-excision update applied 2026-05-09" so future readers can date the rewrite.

    ## Why a v0.25.6 amendment, not v0.25.7

    LoyalSpruce was fixed in this commit and rides v0.25.6. NobleCrane describes the visual E2E architecture; if the architecture itself migrated to tmux when Zellij was excised, the doc must follow. If the architecture didn't migrate, the visual E2E in Phase 1 step 3 is broken and v0.25.6 cannot ship the visual sweep with confidence — making this a release blocker that must be resolved before tag.

    ## Dispatch hint

    Avery — needs to (a) verify the current `./scripts/maintain.sh test-e2e-visual-status` actually drives tmux today, (b) translate NobleCrane's architecture section, (c) confirm with a test run that the Phase 1 visual sweep produces meaningful output.
  started_at: '2026-05-09T22:18:29+00:00'
  completed_at: '2026-05-09T22:19:10+00:00'
---

## Transition note (2026-05-09T22:19:10+00:00)

Implemented and merged in commit eca028d + merge b36241f. 6 notes inline-edited or banner'd; 2 frontmatter'd notes captured as follow-up Notes (NOTE-SolidField, NOTE-SilentBear).
