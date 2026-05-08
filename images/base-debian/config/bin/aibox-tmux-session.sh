#!/usr/bin/env bash
# IMAGE/RUNTIME variant of aibox-tmux-session.sh
#
# This file is baked into the container image and runs inside the container
# when a new tmux session is started.  It is the authoritative fallback
# launcher used by the image entrypoint BEFORE any apply-time managed files
# are present.
#
# There is a separate GENERATED variant (cli/src/templates/aibox-home/.config/
# tmux/aibox-session.sh) that is copied into .aibox-home at `aibox apply` time
# and takes precedence at runtime via AIBOX_TMUX_MANAGED_SESSION.  The two
# files have different roles:
#   IMAGE variant  — baked into /usr/local/bin; always available; no
#                    per-project customisation.
#   GENERATED variant — written from aibox.toml; reflects the project owner's
#                       configured layout, session name, etc.
#
# DO NOT edit the GENERATED variant (under cli/src/templates/) without also
# reviewing this file for parity — and vice-versa.  The two must remain
# behaviourally compatible so a container started without a managed session
# script still falls back gracefully.
set -euo pipefail

layout="${1:-${AIBOX_TMUX_LAYOUT:-ai}}"
session="${2:-${AIBOX_TMUX_SESSION:-aibox}}"
workspace="${AIBOX_WORKSPACE:-/workspace}"
config="${AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}"
layout_script="${AIBOX_TMUX_LAYOUT_SCRIPT:-$HOME/.config/tmux/layouts/${layout}.sh}"
managed_session="${AIBOX_TMUX_MANAGED_SESSION:-$HOME/.config/tmux/aibox-session.sh}"
socket="${AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}"
mkdir -p "$(dirname "$socket")"

if [[ "${AIBOX_TMUX_NO_MANAGED_SESSION:-}" != "1" ]]; then
    if [[ -x "${layout_script}" ]]; then
        AIBOX_TMUX_SESSION="${session}" AIBOX_WORKSPACE="${workspace}" AIBOX_TMUX_SOCKET="${socket}" exec "${layout_script}"
    fi
    if [[ -x "${managed_session}" ]]; then
        AIBOX_TMUX_SESSION="${session}" AIBOX_WORKSPACE="${workspace}" AIBOX_TMUX_SOCKET="${socket}" exec "${managed_session}"
    fi
fi

tmuxc() {
    tmux -S "${socket}" "$@"
}

tmuxcf() {
    tmux -S "${socket}" -f "${config}" "$@"
}

if tmuxc has-session -t "${session}" 2>/dev/null; then
    exec tmuxcf attach-session -t "${session}"
fi

tool_or_shell() {
    local tool="$1"
    if [[ "${tool}" == "yazi" ]]; then
        printf 'for _ in {1..50}; do tmux -S %q list-clients -t %q >/dev/null 2>&1 && break; sleep 0.1; done; if command -v yazi >/dev/null 2>&1; then exec yazi; fi; exec bash' "${socket}" "${session}"
        return
    fi
    printf 'if command -v %q >/dev/null 2>&1; then %q; fi; exec bash' "${tool}" "${tool}"
}

set_title() {
    local target="$1"
    local title="$2"
    tmuxc select-pane -t "${target}" -T "${title}" 2>/dev/null || true
}

new_window() {
    tmuxc new-window -t "${session}:" -n "$1" -c "${workspace}" "bash -lc '$(tool_or_shell "$2")'"
}

# BR-VIM-HARDCUT (DEC-20260508_1604-LuckySeal, v0.25.6): the persistent
# vim/editor pane has been removed from every layout. Yazi `e` opens vim
# in a full-screen tmux popup that closes on `:q`; `Enter` runs vim via
# yazi's `[opener.edit]` (suspends yazi until `:q`).
case "${layout}" in
    focus)
        tmuxcf new-session -d -s "${session}" -n focus -c "${workspace}" "bash -lc '$(tool_or_shell bash)'"
        ;;
    browse)
        tmuxcf new-session -d -s "${session}" -n files -c "${workspace}" "bash -lc '$(tool_or_shell yazi)'"
        files_pane="$(tmuxc display-message -p -t "${session}:files" '#{pane_id}')"
        set_title "${files_pane}" files
        new_window shell "bash"
        ;;
    cowork|cowork-swap)
        tmuxcf new-session -d -s "${session}" -n work -c "${workspace}" "bash -lc '$(tool_or_shell yazi)'"
        files_pane="$(tmuxc display-message -p -t "${session}:work" '#{pane_id}')"
        set_title "${files_pane}" files
        shell_pane="$(tmuxc split-window -t "${session}:work" -h -P -F '#{pane_id}' -c "${workspace}" "bash")"
        set_title "${shell_pane}" shell
        new_window shell "bash"
        ;;
    dev|ai|*)
        tmuxcf new-session -d -s "${session}" -n dev -c "${workspace}" "bash -lc '$(tool_or_shell yazi)'"
        files_pane="$(tmuxc display-message -p -t "${session}:dev" '#{pane_id}')"
        set_title "${files_pane}" files
        shell_pane="$(tmuxc split-window -t "${session}:dev" -h -l 50% -P -F '#{pane_id}' -c "${workspace}" "bash")"
        set_title "${shell_pane}" shell
        tmuxc select-pane -t "${files_pane}"
        new_window shell "bash"
        tmuxc new-window -t "${session}:" -n help -c "${workspace}" "bash -lc 'less \"$HOME/.config/cheatsheet.txt\" 2>/dev/null; exec bash'"
        ;;
esac

tmuxc select-window -t "${session}:1"
exec tmuxcf attach-session -t "${session}"
