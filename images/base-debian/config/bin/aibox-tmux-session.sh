#!/usr/bin/env bash
set -euo pipefail

layout="${1:-${AIBOX_TMUX_LAYOUT:-ai}}"
session="${2:-${AIBOX_TMUX_SESSION:-aibox}}"
workspace="${AIBOX_WORKSPACE:-/workspace}"
config="${AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}"
layout_script="${AIBOX_TMUX_LAYOUT_SCRIPT:-$HOME/.config/tmux/layouts/${layout}.sh}"
managed_session="${AIBOX_TMUX_MANAGED_SESSION:-$HOME/.config/tmux/aibox-session.sh}"

if [[ "${AIBOX_TMUX_NO_MANAGED_SESSION:-}" != "1" ]]; then
    if [[ -x "${layout_script}" ]]; then
        AIBOX_TMUX_SESSION="${session}" AIBOX_WORKSPACE="${workspace}" exec "${layout_script}"
    fi
    if [[ -x "${managed_session}" ]]; then
        AIBOX_TMUX_SESSION="${session}" AIBOX_WORKSPACE="${workspace}" exec "${managed_session}"
    fi
fi

if tmux has-session -t "${session}" 2>/dev/null; then
    exec tmux -f "${config}" attach-session -t "${session}"
fi

new_window() {
    tmux new-window -t "${session}:" -n "$1" -c "${workspace}" "$2"
}

case "${layout}" in
    focus)
        tmux -f "${config}" new-session -d -s "${session}" -n editor -c "${workspace}" "vim-loop"
        tmux select-pane -t "${session}:editor.1" -T editor
        ;;
    browse)
        tmux -f "${config}" new-session -d -s "${session}" -n files -c "${workspace}" "yazi"
        tmux select-pane -t "${session}:files.1" -T files
        new_window shell "bash"
        ;;
    cowork|cowork-swap)
        tmux -f "${config}" new-session -d -s "${session}" -n work -c "${workspace}" "yazi"
        tmux select-pane -t "${session}:work.1" -T files
        tmux split-window -t "${session}:work" -v -c "${workspace}" "vim-loop"
        tmux select-pane -t "${session}:work.2" -T editor
        tmux split-window -t "${session}:work" -h -c "${workspace}" "bash"
        tmux select-pane -t "${session}:work.3" -T shell
        tmux select-layout -t "${session}:work" tiled
        new_window shell "bash"
        ;;
    dev|ai|*)
        tmux -f "${config}" new-session -d -s "${session}" -n dev -c "${workspace}" "yazi"
        tmux select-pane -t "${session}:dev.1" -T files
        tmux split-window -t "${session}:dev" -h -l 60% -c "${workspace}" "vim-loop"
        tmux select-pane -t "${session}:dev.2" -T editor
        tmux split-window -t "${session}:dev.2" -v -l 35% -c "${workspace}" "bash"
        tmux select-pane -t "${session}:dev.3" -T shell
        tmux select-pane -t "${session}:dev.1"
        new_window shell "bash"
        new_window help "less $HOME/.config/cheatsheet.txt"
        ;;
esac

tmux select-window -t "${session}:1"
exec tmux -f "${config}" attach-session -t "${session}"
