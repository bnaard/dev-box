---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1519-LuckyLily-code-quality-aibox-toml-seed-split
  created: '2026-05-08T15:19:10+00:00'
  labels:
    track: code-quality
    release: v0.25.6
  updated: '2026-05-08T21:28:30+00:00'
spec:
  title: 'v0.25.6: Code quality — aibox.toml dedup and seed.rs split'
  state: done
  type: task
  priority: medium
  description: |
    ## Goal
    Apply the code-quality items from the v0.25.6 review per owner approval: dedup [skills] in aibox.toml without losing inline discoverability, split seed.rs tmux/PowerKit functions, document the boundary between the two `aibox-tmux-session.sh` files, and surface release-smoke status diffs in CI.

    ## Items

    ### Q1 — aibox.toml [skills] dedup (owner-approved direction)
    - File: `cli/src/container.rs` and `cli/src/seed.rs` (whichever emits the [skills] block); root `aibox.toml`.
    - Today: every skill appears twice — once in `enabled[]` (uncommented = on) and once in `disabled[]` (uncommented = explicit-off). Remove the duplication.
    - Target shape: a single `[skills]` block where each known skill is one line with a one-line inline comment (`# processkit; <one-line description>` truncated to ~120 chars). Default state is encoded by whether the line is uncommented.
      - Example shape (illustrative):
        ```
        [skills]
        enabled = [
            "actor-profile",  # processkit; Actor entities — humans/AI agents in the project
            # "ai-fundamentals",  # data-ai; Core ML/AI concepts — model types, eval, neural archs
            ...
        ]
        ```
      - Comment-out = disabled; uncomment = enabled. No separate `disabled[]` array.
    - Owner constraint: keep the heavy commenting (one line per skill so users can configure without docs). Streamline only where the comment is wrong/outdated; do not shorten further.

    ### Q2 — Add `aibox skills add/remove` CLI surface (optional, follow-up)
    - File: `cli/src/cli.rs`, new module `cli/src/skills_cmd.rs`.
    - `aibox skills add <name>` uncomments the line in [skills]; `aibox skills remove <name>` comments it. Idempotent. Driven by the same catalog used by skill-finder.
    - Do this only if it lands cleanly within the same release window.

    ### Q3 — Split seed.rs tmux/PowerKit module
    - File: `cli/src/seed.rs` (3,100+ lines today).
    - Move tmux/PowerKit functions to `cli/src/tmux/mod.rs` + submodules: `tmux::status` (PowerKit settings + status-format rendering, currently `seed.rs:1275-1397`), `tmux::layouts` (currently `seed.rs:1400-1502`), `tmux::sync` (`sync_tmux_runtime_files`, currently `seed.rs:1606-1656`).
    - Public API surface unchanged; just module reorganization. Update call sites in `container.rs:660-668`.

    ### Q4 — Document the two `aibox-tmux-session.sh` boundary
    - Files: `images/base-debian/config/bin/aibox-tmux-session.sh` and `cli/src/templates/aibox-home/.config/tmux/aibox-session.sh` (if present in the template tree).
    - Add a header banner to each explaining: "this is the IMAGE/RUNTIME variant — DO NOT edit the GENERATED variant" and vice versa.
    - Add the same explanation to `AGENTS.md` so contributors don't edit the wrong one.

    ### Q5 — Surface release-smoke status diffs in CI
    - File: `.github/workflows/*.yml` (find the existing release/CI workflow) or a new step.
    - After `cargo build --release`, run a minimal `release-runtime-smoke.sh` slice and emit a per-PR diff against the latest `dist/release-smoke/v0.25.X/` baseline for `tmux-state.txt` and `up-forget-tmux-state.log`. Failure on regex regression of known stable lines.

    ### Q6 — Decision-record fixed PowerKit slot order
    - File: `context/decisions/` — short follow-up note pointing at DEC-20260508_1515-SilentAsh that documents the current slot order is intentionally fixed and reordering requires a schema bump.

    ### Q7 — Streamline / fact-check the [skills] inline comments
    - After Q1 lands, walk the comment list once and fix any descriptions that are stale or contradicted by the current SKILL.md.

    ## Acceptance criteria
    - `wc -l aibox.toml` drops materially (target: -70 to -100 lines on the [skills] block).
    - `cli/src/seed.rs` < 2,400 lines; new `cli/src/tmux/` directory exists with well-scoped modules.
    - All existing tests still green after the seed split.
    - Banner comments added; AGENTS.md updated.

    ## Dispatch hint for next session
    One general-purpose subagent for Q1+Q7 (toml dedup + comment streamline) — purely text editing.
    One general-purpose subagent for Q3 (seed.rs split) — mechanical move; rely on rust-analyzer/compiler for correctness.
    Q2, Q4, Q5, Q6 can be bundled with the seed-split agent if time permits, otherwise next release.
  started_at: '2026-05-08T21:14:39+00:00'
  completed_at: '2026-05-08T21:28:30+00:00'
---

## Transition note (2026-05-08T21:14:39+00:00)

Q1 already shipped (commit ce35a4d). Now dispatching Q3 (seed.rs split into cli/src/tmux/) and Q4 (banner comments) to Avery. Q2/Q5/Q6/Q7 deferred to follow-up tracks.


## Transition note (2026-05-08T21:28:25+00:00)

Q1 (toml dedup) shipped in commit ce35a4d. Q3 (seed split) + Q4 (banners) shipped in commit (this batch). Q6 (slot order DEC) recorded as DEC-20260508_2115-SilentFern. Q2 (skills CLI), Q5 (CI workflow), Q7 (comment fact-check) deferred to follow-up tracks.


## Transition note (2026-05-08T21:28:30+00:00)

Accepted as done. Q2/Q5/Q7 follow-up tracks to file separately if needed.
