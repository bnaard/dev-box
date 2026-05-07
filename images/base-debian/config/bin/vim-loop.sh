#!/bin/bash
# vim-loop.sh — Run vim in a loop so the editor pane never dies.
# When vim exits (:q), it restarts with an empty buffer.
# Exit the loop with :cq (exit with error code) or Ctrl+C.

while true; do
    vim --cmd "set t_u7=" --cmd "set t_RV=" "$@"
    exit_code=$?
    # :cq exits with code 1 — use this to truly quit
    if [ "$exit_code" -ne 0 ]; then
        break
    fi
    # Normal :q — return focus to yazi pane, then restart vim
    dir="${AIBOX_EDITOR_DIR:-right}"
    case "$dir" in
        down) tmux select-pane -U 2>/dev/null ;;
        tab)  tmux select-window -t files 2>/dev/null ;;
        *)    tmux select-pane -L 2>/dev/null ;;
    esac
done
