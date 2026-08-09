#!/usr/bin/env bash
# Owner-reviewed entry point. Its sole argument is a prepared release run dir.
set -euo pipefail

if [[ "$#" -ne 1 ]]; then
  echo "Usage: ./scripts/release-host-gate.sh tmp/host-gates/aibox-release/<run-id>" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OWNER_NAME="$(/usr/bin/id -un)"
OWNER_HOME_RECORD="$(/usr/bin/dscl . -read "/Users/${OWNER_NAME}" NFSHomeDirectory)"
OWNER_HOME="${OWNER_HOME_RECORD#NFSHomeDirectory: }"
case "$(/usr/bin/uname -m)" in
  arm64) UV_BIN="/opt/homebrew/bin/uv" ;;
  x86_64) UV_BIN="/usr/local/bin/uv" ;;
  *) echo "Unsupported macOS architecture" >&2; exit 1 ;;
esac
[[ -x "${UV_BIN}" ]] || { echo "Owner-approved uv is required at ${UV_BIN}" >&2; exit 1; }

exec /usr/bin/env -i \
  HOME="${OWNER_HOME}" \
  PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin" \
  UV_CACHE_DIR="${OWNER_HOME}/Library/Caches/aibox-host-gates/uv" \
  UV_PYTHON_INSTALL_DIR="${OWNER_HOME}/Library/Application Support/aibox-host-gates/python" \
  UV_NO_CONFIG=1 UV_OFFLINE=1 AIBOX_HOST_GATE_UV_BIN="${UV_BIN}" \
  "${UV_BIN}" run --offline --no-project --python 3.12.11 \
  "${SCRIPT_DIR}/release_host_gate.py" "$1"
