#!/bin/bash
# open-in-editor.sh — Open a file in the Vim editor pane/window from Yazi.
#
# Behavior depends on layout:
# - dev/ai layouts: Vim is in the same tmux window, usually to the right
# - focus layout: Vim owns the editor window
# - browse layout: no editor pane is started until the first file open
#
# Set AIBOX_EDITOR_DIR to: right, down, or tab.
# When unset, the helper prefers an existing editor pane/window and falls
# back to creating a one-shot Vim pane in the current tmux window.

file="${1:-}"
[ -z "$file" ] && exit 1

file="$(realpath "$file" 2>/dev/null || printf '%s' "$file")"

if [ -z "${TMUX:-}" ] || ! command -v tmux >/dev/null 2>&1; then
    exec "${EDITOR:-vim}" "$file"
fi

dir="${AIBOX_EDITOR_DIR:-}"
session="$(tmux display-message -p '#{session_name}')"
session_window="$(tmux display-message -p '#{session_name}:#{window_index}')"
current_window_id="$(tmux display-message -p '#{window_id}')"
source_pane="${TMUX_PANE:-$(tmux display-message -p '#{pane_id}')}"

vim_escape_path() {
    printf '%s' "$1" \
        | sed -e 's/\\/\\\\/g' \
              -e 's/ /\\ /g' \
              -e 's/	/\\	/g' \
              -e 's/|/\\|/g' \
              -e 's/%/\\%/g' \
              -e 's/#/\\#/g' \
              -e 's/!/\\!/g'
}

shell_quote() {
    printf "'%s'" "$(printf '%s' "$1" | sed "s/'/'\\\\''/g")"
}

find_editor_pane() {
    current_target="$(tmux list-panes -t "$session_window" -F '#{pane_id} #{pane_title} #{pane_current_command}' 2>/dev/null \
        | awk '{ line=tolower($0); if (line ~ /(^| )editor( |$)|vim-loop|(^| )vim( |$)/) { print $1; exit } }'
    )"
    if [ -n "$current_target" ]; then
        printf '%s\n' "$current_target"
        return 0
    fi

    tmux list-windows -t "$session" -F '#{window_id} #{window_name}' 2>/dev/null \
        | while read -r window_id window_name; do
            [ "$window_id" = "$current_window_id" ] && continue
            window_name="$(printf '%s' "$window_name" | tr '[:upper:]' '[:lower:]')"
            case "$window_name" in
                editor|vim)
                    tmux list-panes -t "$window_id" -F '#{pane_id} #{pane_title} #{pane_current_command}' 2>/dev/null \
                        | awk '{ line=tolower($0); if (line ~ /(^| )editor( |$)|vim-loop|(^| )vim( |$)/) { print $1; exit } }'
                    ;;
            esac
        done \
        | head -n 1
}

pane_is_editor() {
    tmux display-message -p -t "$1" '#{pane_current_command}' 2>/dev/null \
        | awk 'BEGIN { found=0 } { cmd=tolower($0); if (cmd ~ /^(vim|nvim|vim-loop)$/) found=1 } END { exit found ? 0 : 1 }'
}

vim_return_cmd() {
    escaped="$(vim_escape_path "$source_pane")"
    printf 'silent! autocmd VimLeavePre <buffer> ++once call system("tmux select-pane -t %s >/dev/null 2>&1")' "$escaped"
}

target="$(find_editor_pane)"
if [ -z "$target" ]; then
    source_dir="$(dirname "$file")"
    open_cmd="vim --cmd \"set t_u7=\" --cmd \"set t_RV=\" --cmd \"$(vim_return_cmd)\" $(shell_quote "$file")"
    case "${dir:-right}" in
        down) split_flag="-v" ;;
        tab)
            tmux new-window -n editor -c "$source_dir" "$open_cmd"
            exit 0
            ;;
        *) split_flag="-h" ;;
    esac
    target="$(tmux split-window "$split_flag" -P -F '#{pane_id}' -c "$source_dir" "$open_cmd")"
    tmux select-pane -t "$target" -T editor
    exit 0
fi

vim_file="$(vim_escape_path "$file")"
if ! pane_is_editor "$target"; then
    tmux send-keys -t "$target" C-c
    tmux send-keys -t "$target" "vim --cmd \"set t_u7=\" --cmd \"set t_RV=\" --cmd \"$(vim_return_cmd)\" $(shell_quote "$file")" Enter
else
    tmux send-keys -t "$target" Escape
    tmux send-keys -t "$target" ":silent! autocmd VimLeavePre <buffer> ++once call system(\"tmux select-pane -t ${source_pane} >/dev/null 2>&1\")" Enter
    tmux send-keys -t "$target" ":edit ${vim_file}" Enter
fi
target_window="$(tmux display-message -p -t "$target" '#{session_name}:#{window_index}' 2>/dev/null || true)"
[ -n "$target_window" ] && tmux select-window -t "$target_window"
tmux select-pane -t "$target"
