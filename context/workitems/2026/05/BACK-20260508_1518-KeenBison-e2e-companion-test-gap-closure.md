---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1518-KeenBison-e2e-companion-test-gap-closure
  created: '2026-05-08T15:18:09+00:00'
  labels:
    track: test-gaps
    release: v0.25.6
  updated: '2026-05-08T15:20:27+00:00'
spec:
  title: 'v0.25.6: E2E and companion test gap closure'
  state: backlog
  type: task
  priority: high
  description: |
    ## Goal
    Close the six high-priority test gaps so the v0.25.6 changes can never silently regress and the symptoms that shipped in v0.25.4/0.25.5 would have been caught in CI.

    ## Tests to add

    ### H1 — Disabled-tool absence (lazygit + every other addon tool)
    - File: `cli/tests/e2e/addon.rs` (or new `addon_disablement.rs`).
    - Fixture: derived project with `[addons.git-ui.tools.lazygit] enabled = false`.
    - Assert: generated `.devcontainer/Dockerfile` does NOT contain `apt-get install ... lazygit` AND contains the purge block.
    - Parameterize across kubernetes (kubectl/helm), cloud-aws/azure/gcp, infrastructure (opentofu/packer), audio-voice, preview-archive, preview-enhanced, data-preview, yazi-omp.
    - Companion test: build the image with `--no-cache` and run `dpkg-query -W -f='${Status}' lazygit` — assert "no packages found".
    - Note: `cli/tests/e2e/runtime_generated.rs:73` currently unconditionally requires `lazygit --version` — adjust that to be conditional on the addon enabled state (don't simply remove the existing assertion).

    ### H2 — Legacy `powerline` deprecation warning
    - File: `cli/tests/e2e/config_coverage.rs` (extend existing).
    - Fixture: aibox.toml with `[customization.tmux.status] mode = "powerline"`.
    - Assert: `aibox apply` exits 0 but writes a deprecation warning to stderr; `aibox doctor` returns a warning row matching the new doctor lint code.

    ### H3 — PowerKit status renders
    - File: `cli/tests/e2e/runtime_generated.rs` (extend).
    - Fixture: derived project with default `mode = "extended"`.
    - Assert (companion test): after `aibox up`, run `tmux -S "$AIBOX_TMUX_SOCKET" show -gv status-format` and assert the captured string contains the expected hostname/external_ip/datetime/git/aibox tokens (mirrors release-runtime-smoke).

    ### M1 — `--forget-tmux-state` clean attach (no stderr noise)
    - File: `cli/tests/e2e/lifecycle.rs` (extend).
    - Fixture: running container.
    - Assert: `aibox up --forget-tmux-state` stderr does NOT contain `error connecting to /tmp/tmux-1000/default`.
    - Companion: lift the relevant assertions from `scripts/release-runtime-smoke.sh:303-340` into Rust e2e so it runs every PR not only at release.

    ### M2 — Corrupted tmux.conf is recovered by apply
    - File: new `cli/tests/e2e/runtime_recovery.rs`.
    - Fixture: place a file with `set -g status off` + `set -g status-right " off_RIGHT "` into `.aibox-home/.config/tmux/tmux.conf`.
    - Assert: after `aibox apply`, the file is rewritten to the current generated content and contains `tmux-powerkit.tmux` if-shell guard.

    ### M3 — Yazi clean startup
    - File: `cli/tests/e2e/runtime_generated.rs` (extend).
    - Assert: `timeout 6s yazi --debug` inside the container exits without writing `Terminal response timeout`. Run after `aibox up` so the regenerated `layouts/ai.sh` wait-loop is in effect.

    ## Acceptance criteria
    - All new tests pass on a fresh checkout.
    - `cargo test --workspace` time budget: do not extend by more than ~30%; run companion-only tests under a feature flag if needed.

    ## Dispatch hint for next session
    One general-purpose subagent. After BR-CLEANUP-ARCH and BR-DOCTOR-GAPS land, this agent picks up. Tests should call into the new doctor codes and the new cleanup pathways.
  blocked_by:
  - BACK-20260508_1516-BrightStream-stale-state-cleanup-architecture-foundational
  - BACK-20260508_1517-SnowyWillow-doctor-coverage-gap-closure-work
---
