#!/usr/bin/env bash
# Owner-reviewed entry point. Dry-run performs validation without publication.
set -euo pipefail

DRY_RUN=0
UI_MODE=auto
UI_SEEN=0
RUN_DIR=""
for ARGUMENT in "$@"; do
  case "${ARGUMENT}" in
    --dry-run) [[ "${DRY_RUN}" == 0 ]] || { echo "Duplicate --dry-run" >&2; exit 2; }; DRY_RUN=1 ;;
    --ui=auto|--ui=textual|--ui=plain) [[ "${UI_SEEN}" == 0 ]] || { echo "Duplicate --ui" >&2; exit 2; }; UI_MODE="${ARGUMENT#--ui=}"; UI_SEEN=1 ;;
    --*) echo "Unknown release-host option: ${ARGUMENT}" >&2; exit 2 ;;
    *) [[ -z "${RUN_DIR}" ]] || { echo "Only one release-host run directory is accepted" >&2; exit 2; }; RUN_DIR="${ARGUMENT}" ;;
  esac
done

if [[ -z "${RUN_DIR:-}" ]]; then
  echo "Usage: ./scripts/release-host-gate.sh [--dry-run] [--ui=auto|textual|plain] tmp/host-gates/aibox-release/<run-id>" >&2
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

# Accept uv's official standalone-installer location as well as the native
# Homebrew prefix. Do not search inherited PATH: the executable must come from
# one of these explicit owner-approved paths and must not be writable by group
# or other users.
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
  UV_NO_CONFIG=1 UV_PYTHON_DOWNLOADS=automatic AIBOX_HOST_GATE_UV_BIN="${UV_BIN}" \
  TERM="${TERM:-dumb}" COLORTERM="${COLORTERM:-}" \
  AIBOX_RELEASE_HOST_DRY_RUN="${DRY_RUN}" AIBOX_RELEASE_HOST_UI="${UI_MODE}" \
  "${UV_BIN}" run --no-project --python 3.14.6 \
  --with-requirements "${SCRIPT_DIR}/release-host-ui.lock" -- \
  python "${SCRIPT_DIR}/release_host_gate.py" "${RUN_DIR}"
