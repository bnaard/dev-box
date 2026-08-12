#!/usr/bin/env bash
# =============================================================================
# build-macos.sh — build aibox CLI binaries for macOS
#
# Run this on a macOS host (not inside the dev-container).
# Produces release binaries for both Apple Silicon and Intel Macs.
#
# Usage:
#   ./scripts/build-macos.sh [version]
#
# Examples:
#   ./scripts/build-macos.sh           # build without version tag
#   ./scripts/build-macos.sh 0.2.0     # build with version in artifact names
#
# Output:
#   dist/aibox-[vVERSION-]aarch64-apple-darwin.tar.gz
#   dist/aibox-[vVERSION-]x86_64-apple-darwin.tar.gz
#
# Prerequisites:
#   - macOS (any version with Xcode command line tools)
#   - Rust toolchain (script will prompt to install if missing)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CLI_DIR="${PROJECT_ROOT}/cli"
DIST_DIR="${PROJECT_ROOT}/dist"

# ── Colours ──────────────────────────────────────────────────────────────────
bold=$'\e[1m'
cyan=$'\e[36m'
red=$'\e[31m'
green=$'\e[32m'
yellow=$'\e[33m'
reset=$'\e[0m'

info()  { echo "${cyan}${bold}==>${reset} $*"; }
ok()    { echo "${green}${bold} ✓${reset} $*"; }
warn()  { echo "${yellow}${bold}  !${reset} $*"; }
die()   { echo "${red}${bold}ERR${reset} $*" >&2; exit 1; }

# ── Preflight checks ────────────────────────────────────────────────────────

# Must be macOS
[[ "$(uname -s)" == "Darwin" ]] || die "This script must be run on macOS."

# Check for Rust toolchain
if ! command -v cargo &>/dev/null; then
  echo ""
  warn "Rust toolchain not found."
  echo "  Install it with:  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo ""
  die "Install Rust and re-run this script."
fi

# ── Parse arguments ──────────────────────────────────────────────────────────
VERSION="${1:-}"
if [[ -n "${VERSION}" ]]; then
  if ! [[ "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
    die "Version must be semver: X.Y.Z or X.Y.Z-prerelease (got: ${VERSION})"
  fi
  VERSION_TAG="v${VERSION}-"
else
  VERSION_TAG=""
fi

# ── Targets ──────────────────────────────────────────────────────────────────
TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")

info "Ensuring Rust targets are installed..."
installed_targets="$(rustup target list --installed)"
for target in "${TARGETS[@]}"; do
  if ! grep -qx "${target}" <<<"${installed_targets}"; then
    if [[ "${AIBOX_RELEASE_HOST_OFFLINE:-0}" == "1" ]]; then
      die "Required Rust target ${target} is not installed; host validation is offline and will not acquire it."
    fi
    rustup target add "${target}"
  fi
done
ok "Targets ready: ${TARGETS[*]}"

# ── Build ────────────────────────────────────────────────────────────────────
mkdir -p "${DIST_DIR}"

build_log_dir="$(mktemp -d "${TMPDIR:-/tmp}/aibox-macos-build.XXXXXX")"
build_pids=()
cleanup_build_logs() {
  local pid
  for pid in "${build_pids[@]}"; do
    if kill -0 "${pid}" >/dev/null 2>&1; then
      kill "${pid}" >/dev/null 2>&1 || true
      wait "${pid}" >/dev/null 2>&1 || true
    fi
  done
  rm -rf "${build_log_dir}"
}
trap cleanup_build_logs EXIT

info "Building macOS targets in parallel..."
for target in "${TARGETS[@]}"; do
  (
    cd "${CLI_DIR}"
    cargo build --release --target "${target}"
  ) >"${build_log_dir}/${target}.log" 2>&1 &
  build_pids+=("$!")
done

build_failed=0
for i in "${!TARGETS[@]}"; do
  target="${TARGETS[${i}]}"
  if wait "${build_pids[${i}]}"; then
    ok "Built ${target}"
  else
    warn "Build failed for ${target}:"
    sed 's/^/  /' "${build_log_dir}/${target}.log" >&2
    build_failed=1
  fi
done
[[ "${build_failed}" -eq 0 ]] || die "One or more macOS target builds failed."

for target in "${TARGETS[@]}"; do

  local_name="aibox-${VERSION_TAG}${target}"
  cp "${CLI_DIR}/target/${target}/release/aibox" "${DIST_DIR}/${local_name}"
  tar -czf "${DIST_DIR}/${local_name}.tar.gz" \
    -C "${DIST_DIR}" "${local_name}" \
    -C "${PROJECT_ROOT}" LICENSE
  rm "${DIST_DIR}/${local_name}"
  shasum -a 256 "${DIST_DIR}/${local_name}.tar.gz" | awk '{print $1}' > "${DIST_DIR}/${local_name}.tar.gz.sha256"
  ok "Packaged ${local_name}.tar.gz"
done

# ── Summary ──────────────────────────────────────────────────────────────────
echo ""
echo "${bold}macOS binaries built:${reset}"
echo ""
for target in "${TARGETS[@]}"; do
  local_name="aibox-${VERSION_TAG}${target}"
  echo "  ${DIST_DIR}/${local_name}.tar.gz"
  echo "  ${DIST_DIR}/${local_name}.tar.gz.sha256"
done
echo ""

if [[ -n "${VERSION}" ]]; then
  echo "To attach to an existing GitHub release:"
  echo ""
  echo "  gh release upload v${VERSION} dist/aibox-v${VERSION}-*-apple-darwin.tar.gz dist/aibox-v${VERSION}-*-apple-darwin.tar.gz.sha256"
  echo ""
fi
