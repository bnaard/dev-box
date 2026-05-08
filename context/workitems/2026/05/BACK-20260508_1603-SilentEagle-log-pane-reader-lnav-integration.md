---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1603-SilentEagle-log-pane-reader-lnav-integration
  created: '2026-05-08T16:03:45+00:00'
  labels:
    track: log-panel
    release: v0.25.6
spec:
  title: 'v0.25.6: Log pane reader (lnav) and counter freshness fix'
  state: backlog
  type: task
  priority: high
  description: |
    ## Goal
    Add a built-in, readable log viewer for `.aibox/aibox.log` (NDJSON) accessible via tmux keybinding, plus fix the status bar log-counter to use a recent-window sample rather than the whole log.

    ## Why now
    Owner observed `318/0/37` on the status bar and could not navigate to the underlying log. Investigation showed:
    - 37 errors are stale (April→May historical command-failures across 0.17.x / 0.23.x / 0.25.2 / 0.25.4 host CLIs); zero are from v0.25.5 or v0.25.6
    - `aibox_status_core.rs:390-422` reads the last 256 KiB of the log without time-windowing → stale errors persist on the bar indefinitely
    - No keybinding exists; previous-session attempt did not land

    ## Scope

    ### 1. Bundle `lnav` in the base image
    - Add `lnav` to the apt install in `images/base-debian/Dockerfile` (Debian package; mature TUI log navigator with native JSON-line format support, search, scrolling, filtering, copy)
    - Ship a project lnav format file at `.aibox-home/.config/lnav/formats/aibox.json` (or equivalent) declaring the aibox NDJSON shape (`ts`, `level`/`exit_code`, `cmd`, `version`, `duration_ms`, `msg`) so lnav highlights, filters, and timestamps correctly
    - `aibox apply` emits the format file as part of the runtime-home seed

    ### 2. Tmux keybinding for the log pane
    - Bind `Prefix L` (uppercase L; `Prefix l` is already `last-window`) to:
      `display-popup -E -w 90% -h 80% -d "$(workspace_path)" "lnav /workspace/.aibox/aibox.log /workspace/.aibox/aibox.log.1 2>/dev/null"`
    - Add to the `Prefix ?` keybindings overview as: "Open log pane (lnav)"

    ### 3. Status-bar log-counter freshness
    - `aibox_status_core.rs::read_log_counts` — accept a recent-window cutoff (e.g. last 24h) and skip lines older than that
    - Render: when window-filtered count is 0/0/0 but a non-zero historic count exists, show a subtle indicator (e.g. dim "·") rather than the inflated old number
    - Status bar should reflect "current session health", not "every error since project init"

    ### 4. Doctor + tests
    - `aibox doctor`: check that `lnav` is installed; warn if missing with the apt-install hint
    - New e2e: pre-seed `.aibox/aibox.log` with mixed-age entries, run `aibox apply`, assert format file emitted, lnav launches under the popup binding
    - New unit test for the windowed `read_log_counts`

    ## Alternatives considered
    - `tail -f` in a tmux pane — rejected: NDJSON is not human-readable, no search/filter
    - Custom Rust TUI viewer — rejected: lnav is mature, well-maintained, free
    - `multitail` — rejected: no native JSON support
    - `humanlog` — rejected: less feature-complete than lnav, fewer distros
---
