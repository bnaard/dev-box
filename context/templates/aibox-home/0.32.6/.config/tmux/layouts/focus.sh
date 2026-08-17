#!/usr/bin/env bash
set -euo pipefail

session="${AIBOX_TMUX_SESSION:-aibox}"
workspace="${AIBOX_WORKSPACE:-/workspace}"
config="${AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}"
socket="${AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}"
mkdir -p "$(dirname "$socket")"

# tool_or_shell <tool>: print the bash -lc invocation that the new pane
# will run. In rebuild mode the yazi wait-for-client trick is omitted —
# the session is already attached and tmux is alive.
is_agent_tool() {
  case "$1" in
    claude|codex|gemini|aider|cn|copilot|opencode|hermes|tau|cursor) return 0 ;;
    *) return 1 ;;
  esac
}

agent_harness_name() {
  [[ "$1" == cn ]] && printf 'continue' || printf '%s' "$1"
}

agent_tool_shell() {
  local tool="$1" harness
  harness="$(agent_harness_name "$tool")"
  # Start/finish/error are lifecycle fallbacks for harnesses without native
  # hooks. Native integrations may refine the state (for example, a question)
  # while the process is running. Always return to a usable bash pane.
  printf "bash -lc 'aibox-agent-signal working --harness %q >/dev/null 2>&1 || true; if command -v %q >/dev/null 2>&1; then %q; rc=\\$?; if [ \\$rc -eq 0 ]; then aibox-agent-signal done --harness %q >/dev/null 2>&1 || true; else aibox-agent-signal error --harness %q >/dev/null 2>&1 || true; fi; else aibox-agent-signal error --harness %q >/dev/null 2>&1 || true; fi; exec bash'" "$harness" "$tool" "$tool" "$harness" "$harness" "$harness"
}

if [[ "${AIBOX_LAYOUT_MODE:-}" == "rebuild" ]]; then
  tool_or_shell() {
    local tool="$1"
    if is_agent_tool "$tool"; then
      agent_tool_shell "$tool"
      return
    fi
    if [[ "$tool" == "yazi" ]]; then
      printf "bash -lc 'if command -v yazi >/dev/null 2>&1; then exec yazi; fi; exec bash'"
      return
    fi
    printf "bash -lc 'if command -v %q >/dev/null 2>&1; then %q; fi; exec bash'" "$tool" "$tool"
  }
else
  tool_or_shell() {
    local tool="$1"
    if is_agent_tool "$tool"; then
      agent_tool_shell "$tool"
      return
    fi
    if [[ "$tool" == "yazi" ]]; then
      printf "bash -lc 'for _ in {1..50}; do tmux -S %q list-clients -t %q >/dev/null 2>&1 && break; sleep 0.1; done; if command -v yazi >/dev/null 2>&1; then exec yazi; fi; exec bash'" "$socket" "$session"
      return
    fi
    printf "bash -lc 'if command -v %q >/dev/null 2>&1; then %q; fi; exec bash'" "$tool" "$tool"
  }
fi

# _create_first_window <name> <cmd>: bootstrap the first window of the
# layout. Fresh mode → `new-session -d -s`. Rebuild mode → the helper
# `aibox-tmux-switch-layout` already left exactly one placeholder window
# (named `_swap_`); we rename + respawn it in place.
if [[ "${AIBOX_LAYOUT_MODE:-}" == "rebuild" ]]; then
  _create_first_window() {
    local name="$1" cmd="$2"
    # Find the placeholder window (the only one left by the switcher).
    tmux -S "$socket" rename-window -t "$session:_swap_" "$name"
    tmux -S "$socket" respawn-window -k -t "$session:$name" -c "$workspace" "$cmd"
  }
else
  _create_first_window() {
    local name="$1" cmd="$2"
    tmux -S "$socket" -f "$config" new-session -d -s "$session" -n "$name" -c "$workspace" "$cmd"
  }
fi

# Rebuild mode: run the layout body and stop — caller stays attached.
if [[ "${AIBOX_LAYOUT_MODE:-}" == "rebuild" ]]; then
  _create_first_window "files" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:files" '#{pane_id}')"
tmux -S "$socket" new-window -t "$session:" -n "codex" -c "$workspace" "$(tool_or_shell codex)"
tmux -S "$socket" new-window -t "$session:" -n "claude" -c "$workspace" "$(tool_or_shell claude)"
tmux -S "$socket" new-window -t "$session:" -n "lazygit" -c "$workspace" "$(tool_or_shell lazygit)"
tmux -S "$socket" new-window -t "$session:" -n "shell" -c "$workspace" "bash"
tmux -S "$socket" select-window -t "$session:files"

  exit 0
fi

# Fresh mode: short-circuit if the session is already up; otherwise build
# from scratch and attach.
if tmux -S "$socket" -f "$config" has-session -t "$session" 2>/dev/null; then
  exec tmux -S "$socket" -f "$config" attach-session -t "$session"
fi

_create_first_window "files" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:files" '#{pane_id}')"
tmux -S "$socket" new-window -t "$session:" -n "codex" -c "$workspace" "$(tool_or_shell codex)"
tmux -S "$socket" new-window -t "$session:" -n "claude" -c "$workspace" "$(tool_or_shell claude)"
tmux -S "$socket" new-window -t "$session:" -n "lazygit" -c "$workspace" "$(tool_or_shell lazygit)"
tmux -S "$socket" new-window -t "$session:" -n "shell" -c "$workspace" "bash"
tmux -S "$socket" select-window -t "$session:files"
exec tmux -S "$socket" -f "$config" attach-session -t "$session"
