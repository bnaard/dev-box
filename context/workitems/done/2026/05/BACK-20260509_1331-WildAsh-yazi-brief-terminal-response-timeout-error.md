---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260509_1331-WildAsh-yazi-brief-terminal-response-timeout-error
  created: '2026-05-09T13:31:09+00:00'
  labels:
    version: v0.25.7
    area: yazi
    surface: terminal-stack
  updated: '2026-05-09T22:19:18+00:00'
spec:
  title: 'yazi: brief ''Terminal response timeout'' error flashes in panel on launch'
  state: done
  type: bug
  priority: medium
  description: |
    ## Symptom

    When yazi opens (in the running v0.25.6 session), an error message flashes in the yazi panel for ~1 second before disappearing. The message reads roughly:

    > Terminal response timeout: The request sent by Yazi didn't receive the correct response. Please check your terminal environment per yazi docs/faq#rt

    The error self-clears within ~1s, so functionality appears unaffected, but the flash is visible at every yazi launch and is distracting.

    ## Reference

    - Upstream FAQ entry: yazi docs/faq#rt (terminal-response-timeout). Common causes per yazi docs include: terminal not supporting the queried CSI sequence, a multiplexer (tmux) intercepting the response, or a slow PTY.

    ## Suspected cause (to verify)

    Yazi sends a CSI query at startup (image-protocol / cell-size detection) and expects a response within a short timeout. Likely culprits in our stack:

    1. **tmux passthrough** — yazi runs inside tmux; tmux may be swallowing or delaying the terminal response unless `allow-passthrough on` (or equivalent) is set.
    2. **Terminal capability detection** — host terminal may not respond to the query yazi sends; yazi should be configured to skip the probe (e.g. `image_filter` / preview backend setting in `yazi.toml`).
    3. **Tmux + ttymouse=sgr interaction** — recent vim fix (commit 17c2143) added `ttymouse=sgr` for trackpad scroll under tmux mouse forwarding; worth checking whether yazi's CSI probe is similarly affected by tmux's mouse/keyboard passthrough config.

    ## Proposed investigation steps

    1. Reproduce: open yazi in current session, capture the exact error string.
    2. Check `~/.config/yazi/yazi.toml` for `[manager]`/`[preview]` settings and image protocol selection.
    3. Check tmux config (`~/.config/tmux/tmux.conf` shipped via aibox-home template) for `allow-passthrough` and any DCS/CSI passthrough rules.
    4. If tmux is the culprit: add `set -g allow-passthrough on` (or scoped equivalent) to the aibox-home tmux template, ship via context/templates and runtime sync.
    5. If yazi config is the culprit: pin yazi to a probe-disabled image protocol (e.g. force `kgp` off, or `image_filter = "lanczos3"` with explicit protocol).

    ## Acceptance

    - yazi opens cleanly with no error flash in the panel.
    - Fix delivered as a config update (yazi.toml or tmux.conf) shipped through the aibox-home template, not a one-off home-dir hand edit.
    - Test under the standard aibox session (tmux + the shipped terminal stack).

    ## Notes

    - Discovered 2026-05-09 in active session by owner (Bernhard / TEAMMEMBER-thrifty-otter).
    - Slot in alongside other v0.25.7 tmux/terminal-stack bugs (SnappyWolf, SilentFjord) — same surface area, may share fix branch.
  started_at: '2026-05-09T22:18:31+00:00'
  completed_at: '2026-05-09T22:19:18+00:00'
---

## Transition note (2026-05-09T22:19:18+00:00)

Diagnosed and fixed in commit 6abd17c + merge e427e62. Terminal-emulator-agnostic env passthrough across docker-compose + tmux update-environment. Host-flash verification post `aibox up` still pending.
