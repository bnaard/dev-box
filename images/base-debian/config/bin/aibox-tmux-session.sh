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

tool_or_shell() {
    local tool="$1"
    printf 'if command -v %q >/dev/null 2>&1; then %q; fi; exec bash' "${tool}" "${tool}"
}

set_title() {
    local target="$1"
    local title="$2"
    tmux select-pane -t "${target}" -T "${title}" 2>/dev/null || true
}

new_window() {
    tmux new-window -t "${session}:" -n "$1" -c "${workspace}" "bash -lc '$(tool_or_shell "$2")'"
}

case "${layout}" in
    focus)
        tmux -f "${config}" new-session -d -s "${session}" -n editor -c "${workspace}" "bash -lc '$(tool_or_shell vim-loop)'"
        editor_pane="$(tmux display-message -p -t "${session}:editor" '#{pane_id}')"
        set_title "${editor_pane}" editor
        ;;
    browse)
        tmux -f "${config}" new-session -d -s "${session}" -n files -c "${workspace}" "bash -lc '$(tool_or_shell yazi)'"
        files_pane="$(tmux display-message -p -t "${session}:files" '#{pane_id}')"
        set_title "${files_pane}" files
        new_window shell "bash"
        ;;
    cowork|cowork-swap)
        tmux -f "${config}" new-session -d -s "${session}" -n work -c "${workspace}" "bash -lc '$(tool_or_shell yazi)'"
        files_pane="$(tmux display-message -p -t "${session}:work" '#{pane_id}')"
        set_title "${files_pane}" files
        editor_pane="$(tmux split-window -t "${session}:work" -v -P -F '#{pane_id}' -c "${workspace}" "bash -lc '$(tool_or_shell vim-loop)'")"
        set_title "${editor_pane}" editor
        shell_pane="$(tmux split-window -t "${session}:work" -h -P -F '#{pane_id}' -c "${workspace}" "bash")"
        set_title "${shell_pane}" shell
        tmux select-layout -t "${session}:work" tiled
        new_window shell "bash"
        ;;
    dev|ai|*)
        tmux -f "${config}" new-session -d -s "${session}" -n dev -c "${workspace}" "bash -lc '$(tool_or_shell yazi)'"
        files_pane="$(tmux display-message -p -t "${session}:dev" '#{pane_id}')"
        set_title "${files_pane}" files
        editor_pane="$(tmux split-window -t "${session}:dev" -h -l 60% -P -F '#{pane_id}' -c "${workspace}" "bash -lc '$(tool_or_shell vim-loop)'")"
        set_title "${editor_pane}" editor
        shell_pane="$(tmux split-window -t "${editor_pane}" -v -l 35% -P -F '#{pane_id}' -c "${workspace}" "bash")"
        set_title "${shell_pane}" shell
        tmux select-pane -t "${files_pane}"
        new_window shell "bash"
        tmux new-window -t "${session}:" -n help -c "${workspace}" "bash -lc 'less \"$HOME/.config/cheatsheet.txt\" 2>/dev/null; exec bash'"
        ;;
esac

tmux select-window -t "${session}:1"
exec tmux -f "${config}" attach-session -t "${session}"
