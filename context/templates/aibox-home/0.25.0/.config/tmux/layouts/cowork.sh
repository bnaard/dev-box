#!/usr/bin/env bash
set -euo pipefail

session="${AIBOX_TMUX_SESSION:-aibox}"
workspace="${AIBOX_WORKSPACE:-/workspace}"
config="${AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}"

if tmux -f "$config" has-session -t "$session" 2>/dev/null; then
  exec tmux -f "$config" attach-session -t "$session"
fi

tmux -f "$config" new-session -d -s "$session" -n cowork -c "$workspace" "vim"
tmux split-window -t "$session:cowork" -h -p 50 -c "$workspace" "codex"
tmux split-window -t "$session:cowork.1" -v -p 35 -c "$workspace" "yazi"
tmux select-pane -t "$session:cowork.1"
tmux select-window -t "$session:1"
exec tmux -f "$config" attach-session -t "$session"
