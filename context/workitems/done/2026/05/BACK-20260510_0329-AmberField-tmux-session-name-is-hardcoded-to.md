---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0329-AmberField-tmux-session-name-is-hardcoded-to
  created: '2026-05-10T03:29:22+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux
  updated: '2026-05-10T07:14:20+00:00'
spec:
  title: tmux session name is hardcoded to 'aibox' — should derive from project name
  state: done
  type: bug
  priority: medium
  description: |
    ## Symptom

    When aibox starts a tmux session inside the container, the session name is always literally `aibox`, regardless of which derived project the user is working on. This is wrong — the session name should reflect the project, so a developer running multiple projects can tell at-a-glance which session belongs to which project (statusline `#S`, `tmux ls`, attached client titles all reflect this).

    ## Expected behaviour

    The tmux session name should be derived from the project. Source of truth, in order of preference:

    1. `aibox.toml` `[project] name = "..."` field (if present and non-empty).
    2. The basename of the working directory (e.g. `/workspace/aibox` → `aibox`, `/workspace/foo-app` → `foo-app`).
    3. Fallback to the literal `aibox` only if neither of the above resolves.

    ## Investigation hints

    - Likely locations of the hardcoded value: `cli/src/tmux/`, `cli/src/container.rs`, or the runtime tmux launcher script (under `images/base-debian/config/tmux/` or generated via `cli/src/seed.rs`). Grep for: `new-session`, `-s aibox`, `session_name`, `"aibox"` in tmux contexts.
    - Any sanitization of the project name should match tmux's session-name rules (no `:` or `.`; suggest replacing illegal chars with `-`).

    ## Acceptance

    - A project named `foo-bar` (via aibox.toml `[project] name = "foo-bar"` OR via being in `/workspace/foo-bar`) starts a tmux session named `foo-bar`.
    - The aibox dogfood project (which has `name = "aibox"`) still gets `aibox`.
    - A project with no `[project] name` field falls back to the working-dir basename.
    - Sanitization handles edge cases (empty, non-ASCII, illegal tmux chars).
    - Unit test covers the resolution function.

    ## Refs

    - Likely files: `cli/src/tmux/*.rs`, `cli/src/container.rs::cmd_apply` or `cmd_sync`, possibly `images/base-debian/config/tmux/launcher.sh` style scripts if they exist.
    - Statusline displays `#S` already (per SilentFjord's recent line1-left change in `images/base-debian/config/tmux/tmux.conf:39`) — so the bug is visually surfaced everywhere session ID shows.
  started_at: '2026-05-10T07:14:06+00:00'
  completed_at: '2026-05-10T07:14:20+00:00'
---

## Transition note (2026-05-10T07:14:20+00:00)

Implemented and merged in commit f39302c + merge 59d7e0d. New resolve_tmux_session_name + sanitize_tmux_session_name in cli/src/config.rs; wired through sync_grouped_sections so all callers pick it up. 9 new tests pass. Schema field is [aibox] project_name (not [project] name as the WorkItem assumed); aibox dogfood unchanged.
