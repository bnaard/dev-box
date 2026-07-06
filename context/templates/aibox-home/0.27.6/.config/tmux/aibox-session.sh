#!/usr/bin/env bash
set -euo pipefail

layout="${1:-${AIBOX_TMUX_LAYOUT:-ai}}"
session="${2:-${AIBOX_TMUX_SESSION:-aibox}}"
socket="${AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}"

# Resolve the layout script. Search order:
#   1. User drop-in: ~/.config/tmux/layouts/<layout>.sh
#   2. System default: /usr/local/share/aibox/tmux/layouts/<layout>.sh
user_script="${HOME}/.config/tmux/layouts/${layout}.sh"
system_script="/usr/local/share/aibox/tmux/layouts/${layout}.sh"
if [[ -x "${user_script}" ]]; then
  script="${user_script}"
elif [[ -x "${system_script}" ]]; then
  script="${system_script}"
else
  echo "aibox-tmux-session: unknown or unavailable managed layout: ${layout}" >&2
  echo "  searched: ${user_script}" >&2
  echo "  searched: ${system_script}" >&2
  exit 2
fi

exec env AIBOX_TMUX_SESSION="${session}" AIBOX_TMUX_SOCKET="${socket}" "${script}"
