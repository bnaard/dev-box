#!/usr/bin/env bash
set -euo pipefail

state_dir="${XDG_STATE_HOME:-$HOME/.local/state}/aibox"
state_file="$state_dir/status-line-hidden"
layout_dir="${ZELLIJ_CONFIG_DIR:-$HOME/.config/zellij}/layouts"
hidden_layout="$layout_dir/aibox-status-hidden.kdl"
visible_layout="$layout_dir/aibox-status-visible.kdl"
current_pane_id="${ZELLIJ_PANE_ID:-}"

mkdir -p "$state_dir"

if [ -n "$current_pane_id" ] && [ -r /dev/tty ]; then
  panes_json="$(zellij action list-panes --json --all --command --state --tab </dev/tty 2>/dev/null || true)"
  if [ -n "$panes_json" ]; then
    status_pane_id="$(
      printf '%s' "$panes_json" |
        python3 -c '
import json, sys
current_pane_id = int(sys.argv[1])
panes = json.load(sys.stdin)
tab_id = None
for pane in panes:
    if not pane.get("is_plugin") and pane.get("id") == current_pane_id:
        tab_id = pane.get("tab_id")
        break
if tab_id is None:
    sys.exit(0)
for pane in panes:
    if pane.get("tab_id") != tab_id or pane.get("is_plugin"):
        continue
    text = " ".join(str(pane.get(key) or "") for key in ("terminal_command", "title", "pane_command"))
    if "aibox-status" in text:
        print(f"terminal_{pane[\"id\"]}")
        break
' "$current_pane_id"
    )"

    if [ -n "$status_pane_id" ]; then
      zellij action close-pane --pane-id "$status_pane_id"
      : >"$state_file"
    else
      zellij action override-layout "$visible_layout" --apply-only-to-active-tab
      rm -f "$state_file"
    fi
    exit 0
  fi
fi

if [ -f "$state_file" ]; then
  zellij action override-layout "$visible_layout" --apply-only-to-active-tab
  rm -f "$state_file"
else
  zellij action override-layout "$hidden_layout" --apply-only-to-active-tab
  : >"$state_file"
fi
