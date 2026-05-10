---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_1316-SilentFjord-tmux-statusline-l1-left-oom-double
  created: '2026-05-09T13:16:40+00:00'
  updated: '2026-05-09T22:19:16+00:00'
spec:
  title: 'tmux status-line: add window list to line1-left, fix OOM/LOG/PROC/AI/MCP/MIG
    label doubling'
  state: done
  type: bug
  priority: high
  description: |
    ## Bug A — line1-left missing window list
    Current `cli/src/tmux/status.rs` `DEFAULT_TMUX_CONF` has `set -g status-left " #S "` — only session name. User wants line1-left = "aibox" (session name) **plus** the window/screen list.

    PowerKit-driven config has `@powerkit_status_order "session,plugins"` and `@powerkit_bar_layout "double"` — neither references windows. Window list currently renders in tmux's middle band (default `status-justify "centre"`), not in line1-left.

    ### Fix proposal (awaiting owner approval)
    Investigate powerkit support for a `windows` segment in `@powerkit_status_order`. If unsupported, override line1 directly via tmux `status-format[0]` carrying `#S` + `#{W:#{?window_active,#[reverse],}#I:#W#[default] }`. Either way, line2 left/right and line1-right should remain powerkit-rendered.

    ## Bug B — OOM / LOG / PROC / AI / MCP / MIG label doubled
    Cached plugin data `OOMOOM 0/0` confirms the doubling. Each `images/base-debian/config/tmux/powerkit-plugins/aibox_<metric>.sh` has both `plugin_get_icon() { printf 'OOM'; }` AND `plugin_render() { printf 'OOM %s/%s' ...; }` — PowerKit concatenates icon + render → `OOM OOM 0/0`.

    ### Fix proposal (awaiting owner approval)
    Drop the label prefix from `plugin_render` in all six aibox metric plugins:
    - `aibox_log.sh`: `printf '%s/%s/%s' "$(plugin_data_get log_info)" "$(plugin_data_get log_warn)" "$(plugin_data_get log_error)"`
    - `aibox_oom.sh`: `printf '%s/%s' "$(plugin_data_get oom_events)" "$(plugin_data_get oom_kill)"`
    - `aibox_proc.sh`: similar pattern
    - `aibox_ai.sh`: similar
    - `aibox_mcp.sh`: similar
    - `aibox_mig.sh`: similar

    The icon (rendered by powerkit as the styled segment label) keeps the topic word.

    ## Bug C — confirm right-align rendering
    Code already sets `@powerkit_line1_right` and `@powerkit_line2_right` — should already render right-aligned in `double` bar mode. Verify visually after Bug A is fixed.

    ## Acceptance criteria
    - Line 1 left: "aibox" + active windows (e.g., `aibox 1:ai* 2:shell 3:git`)
    - Line 1 right unchanged: hostname → external_ip → ssh → uptime → weather → datetime
    - Line 2 left unchanged: git → github → kubernetes → terraform → cloud → cloudstatus
    - Line 2 right unchanged but no doubled labels: `cpu loadavg mem swap disk gpu netspeed ping LOG x/y/z OOM x/y PROC x AI x MCP x/y MIG x` (each topic word styled once as the segment label)
    - All six `aibox_*` powerkit plugins emit value-only render

    ## References
    - `cli/src/tmux/status.rs:14-71` — DEFAULT_TMUX_CONF status-left placement
    - `cli/src/tmux/status.rs:119-159` — fixed slot order (DEC-20260508_2115-SilentFern; line1-left is currently NOT in scope of that decision)
    - `images/base-debian/config/tmux/powerkit-plugins/aibox_*.sh` — render functions
    - `.aibox-home/.cache/tmux-powerkit/data/plugin_aibox_oom_data` (`OOMOOM 0/0`) — confirms cache-level doubling
    - DEC-20260508_2115-SilentFern — slot order is fixed; this WorkItem extends scope to line1-left and aibox-metric render strings (will need a paired `record_decision` once approved)
  started_at: '2026-05-09T22:18:30+00:00'
  completed_at: '2026-05-09T22:19:16+00:00'
---

## Transition note (2026-05-09T22:19:16+00:00)

Implemented and merged in commit e6e1e9a + merge e427e62. Six powerkit plugins de-doubled; status-left extended with window list. Paired DEC-20260509_2125-CoolFrog records contract. Visual verification post container restart still pending.
