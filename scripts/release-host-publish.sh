#!/usr/bin/env bash
# Narrow publication stage. It never builds or executes candidate code.
set -euo pipefail

if [[ "$#" -ne 1 ]]; then
  echo "Usage: ./scripts/release-host-publish.sh tmp/host-gates/aibox-release/<run-id>" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OWNER_NAME="$(/usr/bin/id -un)"
OWNER_HOME_RECORD="$(/usr/bin/dscl . -read "/Users/${OWNER_NAME}" NFSHomeDirectory)"
OWNER_HOME="${OWNER_HOME_RECORD#NFSHomeDirectory: }"
case "$(/usr/bin/uname -m)" in
  arm64) UV_SYSTEM_BIN="/opt/homebrew/bin/uv" ;;
  x86_64) UV_SYSTEM_BIN="/usr/local/bin/uv" ;;
  *) echo "Unsupported macOS architecture" >&2; exit 1 ;;
esac
UV_BIN=""
for UV_CANDIDATE in "${OWNER_HOME}/.local/bin/uv" "${UV_SYSTEM_BIN}"; do
  [[ -f "${UV_CANDIDATE}" && -x "${UV_CANDIDATE}" ]] || continue
  UV_OWNER="$(/usr/bin/stat -f '%Su' "${UV_CANDIDATE}")"
  UV_MODE="$(/usr/bin/stat -f '%Lp' "${UV_CANDIDATE}")"
  [[ "${UV_OWNER}" == "${OWNER_NAME}" || "${UV_OWNER}" == "root" ]] || continue
  UV_MODE_DEC=$((8#${UV_MODE}))
  (( (UV_MODE_DEC & 0022) == 0 )) || continue
  UV_BIN="${UV_CANDIDATE}"
  break
done
[[ -n "${UV_BIN}" ]] || {
  echo "Owner-approved uv is required at ${OWNER_HOME}/.local/bin/uv or ${UV_SYSTEM_BIN}; it must be executable, owned by ${OWNER_NAME} or root, and not group/world-writable." >&2
  exit 1
}

exec /usr/bin/env -i \
  HOME="${OWNER_HOME}" \
  PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin" \
  UV_CACHE_DIR="${OWNER_HOME}/Library/Caches/aibox-host-gates/uv" \
  UV_PYTHON_INSTALL_DIR="${OWNER_HOME}/Library/Application Support/aibox-host-gates/python" \
  UV_NO_CONFIG=1 UV_OFFLINE=1 UV_PYTHON_DOWNLOADS=never AIBOX_HOST_GATE_UV_BIN="${UV_BIN}" \
  "${UV_BIN}" run --offline --no-project --python 3.12.11 -- \
  python "${SCRIPT_DIR}/release_host_publish.py" "$1"
