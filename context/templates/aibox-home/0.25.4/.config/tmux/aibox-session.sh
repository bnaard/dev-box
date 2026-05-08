#!/usr/bin/env bash
set -euo pipefail

layout="${1:-${AIBOX_TMUX_LAYOUT:-ai}}"
session="${2:-${AIBOX_TMUX_SESSION:-aibox}}"
socket="${AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}"
script="${HOME}/.config/tmux/layouts/${layout}.sh"

if [[ ! -x "${script}" ]]; then
  echo "aibox-tmux-session: unknown or unavailable managed layout: ${layout}" >&2
  exit 2
fi

exec env AIBOX_TMUX_SESSION="${session}" AIBOX_TMUX_SOCKET="${socket}" "${script}"
