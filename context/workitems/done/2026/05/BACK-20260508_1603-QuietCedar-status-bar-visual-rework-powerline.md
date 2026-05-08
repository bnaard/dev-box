---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1603-QuietCedar-status-bar-visual-rework-powerline
  created: '2026-05-08T16:03:24+00:00'
  labels:
    track: status-rework
    release: v0.25.6
    supersedes_in_v0_25_6: BACK-20260507_1341-SharpCrow-powerline-status-tabbar-redesign
  updated: '2026-05-08T21:41:12+00:00'
spec:
  title: 'v0.25.6: Status bar powerline visual rework with explicit element ordering'
  state: done
  type: task
  priority: high
  description: |
    ## Goal
    Implement the powerline-style two-line status bar with the concrete element ordering owner specified for v0.25.6. Supersedes the v0.25.6-relevant scope of BACK-20260507_1341-SharpCrow (which remains as the broader runtime-ui story); the chevron design intent from SharpCrow stays — this WorkItem makes it concrete and unblocks it (zellij-sidecar-stability gating is moot post-BR-ZELLIJ-EXCISE).

    ## Element ordering (owner-specified, byte-equivalent)

    ### Line 1
    - **Left** (unchanged): aibox identity + screen list
    - **Right** (this order): `hostname`, `external_ip`, `ssh`, `uptime`, `weather`, `datetime` — all PowerKit plugins

    ### Line 2
    - **Left** (this order): `git`, `github`, `kubernetes`, `terraform`, `cloud`, `cloudstatus` — all PowerKit plugins
    - **Right** (this order): `cpu`, `loadavg`, `mem`, `swap`, `disk`, `gpu`, `netspeed`, `ping`, then aibox metrics block: `log`, `oom`, `proc`, `ai`, `mcp`, `mig`

    The aibox metrics block must use the same powerline section/subsection styling (chevron separators, color rotation) as native PowerKit segments — currently it renders flat text.

    ## Implementation pointers
    - `cli/src/seed.rs:1275-1397` — current `tmux_powerkit_settings` (will be moved to `cli/src/tmux/status.rs` by BR-CODE-QUALITY Q3; this WorkItem may merge after that move)
    - `set -g @powerkit_plugin_aibox_metrics "log,oom,proc,ai,mcp,mig"` in current tmux.conf
    - PowerKit chevron rendering: investigate the powerkit_plugin contract (look in `.aibox-home/.cache/tmux-powerkit/` and the tmux plugin sources) to know how to register the aibox segment so it inherits the segment-style chain
    - `images/base-debian/config/bin/aibox_status_core.rs` — produces the metric values surfaced by the plugin
    - Owner reference design lives in `/tmp/design-input/dual-line-status-bar.png` (per SharpCrow); preferred over `zellij-style-bars.svg`

    ## Acceptance
    - Generated tmux.conf emits the exact element ordering above for both lines
    - aibox metrics block uses chevron styling matching adjacent PowerKit segments — visually contiguous
    - New e2e/snapshot test asserts the rendered status string matches a baseline for a known sample state
    - `aibox doctor` warns if any required plugin is missing

    ## Notes
    - Powerline-style tab bar (also from SharpCrow) is OUT of scope for this WorkItem; track separately if needed
    - Coordinate with BR-CODE-QUALITY (Q3 seed.rs split) — touch the new `tmux::status` module rather than `seed.rs` directly
  blocked_by:
  - BACK-20260508_1519-LuckyLily-code-quality-aibox-toml-seed-split
  started_at: '2026-05-08T21:28:54+00:00'
  completed_at: '2026-05-08T21:41:12+00:00'
---

## Transition note (2026-05-08T21:28:54+00:00)

LuckyLily Q3 (seed.rs → cli/src/tmux/status.rs) shipped, now unblocked. Dispatching to Avery to land the explicit two-line element ordering and chevron styling for the aibox metrics block.


## Transition note (2026-05-08T21:41:08+00:00)

Implementation complete in commit (this batch). Two-line layout with byte-exact element ordering; aibox metrics block split into 6 independent chevron-styled plugins; LINT-POWERKIT-STATUS-PLUGINS doctor check added; DEC-SilentFern slot-order reference comment in code. 948 green.


## Transition note (2026-05-08T21:41:12+00:00)

Accepted as done.
