---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0727-QuietCliff-tmux-statusline-rename-external-ip-externalip
  created: '2026-05-10T07:27:54+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-statusline
  updated: '2026-05-10T07:40:02+00:00'
spec:
  title: 'tmux statusline: rename external_ip → externalip in line1-right config to
    match upstream plugin filename'
  state: done
  type: bug
  priority: medium
  description: |
    ## Symptom

    SleekGrove (commit `7a094a1`, merge `a1a1e9e`) wrote `@powerkit_line1_right "hostname,external_ip,ssh,uptime,weather,datetime"` in `images/base-debian/config/tmux/tmux.conf`. Upstream tmux-powerkit ships the plugin as `externalip.sh` (no underscore), so the dispatch lookup misses and that section silently shows nothing.

    Verified by inspecting `/usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/`:
    - `externalip.sh` ✓ (file exists)
    - `external_ip.sh` ✗ (does not exist)

    All 19 other plugins from the SleekGrove four-section spec match upstream filenames exactly — verified end-to-end. **Only `external_ip` is mis-named.**

    ## Fix

    Rename in three places:
    1. `images/base-debian/config/tmux/tmux.conf` — `@powerkit_line1_right "hostname,external_ip,...` → `"hostname,externalip,...` AND `@powerkit_plugins "...,external_ip,...` → `"...,externalip,...`.
    2. `cli/src/tmux/status.rs` — if any Rust-side constant references `external_ip`, rename to `externalip`. Grep first.
    3. `context/migrations/pending/MIG-STATUSLINE-20260510T000000.md` — clean up the misleading "skipped (plugin not implemented)" section. The correct framing is: "all 20 owner-spec plugins ship in upstream tmux-powerkit; the renames `external_ip → externalip` (and verify `mem` vs `memory`) are the only fix needed". Replace the speculative "likely absent" warnings with the verified status.

    ## Verification

    - After fix + container restart, line 1 right shows hostname, external IP, ssh user, uptime, weather, datetime in order.
    - No silent-empty plugin slots in any of the four sections (line1-left, line1-right, line2-left, line2-right).
    - `cargo check` clean.

    ## Refs

    - BACK-20260510_0336-SleekGrove (predecessor)
    - MIG-STATUSLINE-20260510T000000 (paired migration; needs body update)
    - Upstream: https://github.com/fabioluciano/tmux-powerkit (file list under `src/plugins/`)
    - Verified ground truth: `ls /usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins/` (this session)
  started_at: '2026-05-10T07:39:44+00:00'
  completed_at: '2026-05-10T07:40:02+00:00'
---

## Transition note (2026-05-10T07:40:02+00:00)

Implemented and merged in commit dc3dd36 + merge 08954fc. Renamed external_ip → externalip in tmux.conf @powerkit_plugins / @powerkit_line1_right, cli/src/tmux/status.rs LINE1_RIGHT_ORDER + 4 test strings, cli/src/doctor.rs doc + required array. Migration body cleaned up — replaced misleading 'skipped' section with verified facts. 13 plugin-name hits → 0; 5 remaining external_ip occurrences are Rust struct fields (TOML key 'external-ip') correctly preserved. 878/879 tests pass (1 pre-existing LivelyFinch).
