---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_0336-SleekGrove-tmux-statusline-implement-full-owner-spec
  created: '2026-05-10T03:36:09+00:00'
  labels:
    version: v0.25.7-followup
    area: tmux-statusline
    needs-migration: 'true'
  updated: '2026-05-10T07:14:25+00:00'
spec:
  title: 'tmux statusline: implement full owner-spec reorganization (line1-right +
    line2-left + line2-right) — paired Migration required'
  state: done
  type: bug
  priority: high
  description: |
    ## Background

    SilentFjord (commit `e6e1e9a`, merge `e427e62`) shipped the label-doubling fix and added a window list to line1-left. The owner had specified a much fuller layout reorganization that was NOT implemented and remains open.

    DEC-20260508_2115-SilentFern (extended by DEC-20260509_2125-CoolFrog) requires that any reordering / addition / removal of statusline slots be paired with a Migration entity. This WorkItem must be paired with such a Migration before merge.

    ## Owner-specified target layout

    ### Line 1
    - **Left:** session name (`#S` — currently `aibox`, will become project name once `BACK-AmberField` lands) + window list (already shipped in SilentFjord)
    - **Right** (in this order):
      1. hostname (powerkit plugin)
      2. external_ip (powerkit plugin)
      3. ssh (powerkit plugin)
      4. uptime (powerkit plugin)
      5. weather (powerkit plugin)
      6. datetime (powerkit plugin)

    ### Line 2
    - **Left** (in this order):
      1. git status (powerkit plugin git)
      2. github notifications (powerkit plugin github)
      3. kubernetes
      4. terraform
      5. cloud
      6. cloudstatus
    - **Right** (in this order):
      1. cpu
      2. loadavg
      3. mem
      4. swap
      5. disk
      6. gpu
      7. netspeed
      8. ping
      9. **aibox stack:** log, oom, proc, ai, mcp, mig

    ## Implementation hints

    - Source-of-truth: `images/base-debian/config/tmux/tmux.conf` (status-left, status-right, status-format[0], status-format[1] / similar two-line PowerKit layout)
    - Each plugin lives at `images/base-debian/config/tmux/powerkit-plugins/<plugin>.sh` (or similar)
    - Verify each plugin in the spec actually exists; flag missing ones (e.g. `weather`, `external_ip`, `cloudstatus` may not have plugins)
    - The de-doubling fix from SilentFjord must NOT regress

    ## Paired Migration requirement (per DEC-CoolFrog)

    Emit a Migration entity (kind: `aibox-statusline`) under `context/migrations/pending/` documenting the slot-order change. Body should list before-state and after-state slot ordering for each of the four sections (line1-left, line1-right, line2-left, line2-right) and reference DEC-CoolFrog and DEC-SilentFern.

    ## Acceptance

    - Statusline matches the owner spec after `aibox apply` + tmux reload.
    - Plugin label-doubling stays fixed (no regression).
    - Paired Migration entity created.
    - Plugins missing from source (weather, external_ip, cloudstatus, etc. if absent) listed in the Migration body as "skipped — plugin not implemented; add as follow-up WorkItem".
    - Cargo tests pass.

    ## Refs

    - DEC-20260508_2115-SilentFern (slot-order discipline)
    - DEC-20260509_2125-CoolFrog (extension to line1-left + label-emission contract)
    - BACK-20260509_1316-SilentFjord (predecessor; partial fix)
    - BACK-20260510_0329-AmberField (tmux session name; sibling)
  started_at: '2026-05-10T07:14:06+00:00'
  completed_at: '2026-05-10T07:14:25+00:00'
---

## Transition note (2026-05-10T07:14:25+00:00)

Implemented and merged in commit 7a094a1 + merge a1a1e9e. Full four-section statusline layout in images/base-debian/config/tmux/tmux.conf; aibox-side cli/src/tmux/status.rs already routed plugin selection so this lit it up end-to-end. Paired Migration MIG-STATUSLINE-20260510T000000 placed in context/migrations/pending/. 31 tmux tests pass. 18 plugins flagged in the Migration body as missing-from-source — file follow-up WorkItems for each.
