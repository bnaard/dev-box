#!/usr/bin/env bash
# Generate a pre-release dependency and harness state report for aibox.
#
# The report is evidence for the maintainer/agent review step. It is intentionally
# non-mutating: it may query registries and package indexes, but it must not edit
# Cargo.lock, Dockerfiles, addon manifests, or generated runtime files.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CLI_DIR="${PROJECT_ROOT}/cli"
DIST_DIR="${PROJECT_ROOT}/dist"
REPORT="${DIST_DIR}/RELEASE-STATE.md"

mkdir -p "${DIST_DIR}"

updates_found=0
warnings_found=0

section() {
  printf '\n## %s\n\n' "$1" >> "${REPORT}"
}

line() {
  printf '%s\n' "$*" >> "${REPORT}"
}

warn() {
  warnings_found=1
  printf 'WARN: %s\n' "$*" >&2
}

run_with_timeout() {
  local duration="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "${duration}" "$@"
  else
    "$@"
  fi
}

mark_update() {
  updates_found=1
}

normalize_version() {
  printf '%s' "$1" | sed -E 's/^v//; s/^V//'
}

version_gt() {
  local latest current
  latest="$(normalize_version "$1")"
  current="$(normalize_version "$2")"
  [[ -n "${latest}" && -n "${current}" ]] || return 1
  [[ "${latest}" != "${current}" ]] || return 1
  [[ "$(printf '%s\n%s\n' "${current}" "${latest}" | sort -V | tail -n 1)" == "${latest}" ]]
}

github_latest_release() {
  local repo="$1"
  if ! command -v gh >/dev/null 2>&1; then
    return 1
  fi
  run_with_timeout 8s gh api "repos/${repo}/releases/latest" --jq '.tag_name' 2>/dev/null || return 1
}

dockerfile_arg() {
  local file="$1" arg="$2"
  grep -E "^ARG ${arg}=" "${file}" | head -n 1 | sed -E "s/^ARG ${arg}=//"
}

check_github_pin() {
  local label="$1" file="$2" arg="$3" repo="$4"
  local current latest status
  current="$(dockerfile_arg "${file}" "${arg}")"
  latest="$(github_latest_release "${repo}" || true)"
  status="unknown"
  if [[ -z "${current}" ]]; then
    status="missing current pin"
    warnings_found=1
  elif [[ -z "${latest}" ]]; then
    status="latest lookup failed"
    warnings_found=1
  elif version_gt "${latest}" "${current}"; then
    status="update available"
    mark_update
  else
    status="current"
  fi
  printf '| %s | `%s` | `%s` | `%s` | %s |\n' \
    "${label}" "${current:-?}" "${latest:-?}" "${repo}" "${status}" >> "${REPORT}"
}

npm_latest() {
  local package="$1"
  if ! command -v npm >/dev/null 2>&1; then
    return 1
  fi
  run_with_timeout 12s npm view "${package}" version 2>/dev/null || return 1
}

pypi_latest() {
  local package="$1"
  if ! command -v python3 >/dev/null 2>&1; then
    return 1
  fi
  python3 - "$package" <<'PY'
import json
import sys
import urllib.request

package = sys.argv[1]
url = f"https://pypi.org/pypi/{package}/json"
with urllib.request.urlopen(url, timeout=12) as response:
    print(json.load(response)["info"]["version"])
PY
}

node_major_latest() {
  local major="$1"
  if ! command -v python3 >/dev/null 2>&1; then
    return 1
  fi
  python3 - "$major" <<'PY'
import json
import sys
import urllib.request

major = sys.argv[1]
with urllib.request.urlopen("https://nodejs.org/dist/index.json", timeout=12) as response:
    releases = json.load(response)
for release in releases:
    version = release.get("version", "")
    if version.startswith(f"v{major}."):
        print(version)
        break
else:
    raise SystemExit(1)
PY
}

claude_apt_latest() {
  local channel="${1:-latest}"
  if ! command -v python3 >/dev/null 2>&1; then
    return 1
  fi
  python3 - "$channel" <<'PY'
import re
import sys
import urllib.request

channel = sys.argv[1]
url = f"https://downloads.claude.ai/claude-code/apt/{channel}/dists/{channel}/main/binary-amd64/Packages"
with urllib.request.urlopen(url, timeout=12) as response:
    body = response.read().decode("utf-8", "replace")
versions = re.findall(r"^Version:\s*([^\s]+)", body, flags=re.MULTILINE)
if not versions:
    raise SystemExit(1)

def key(version: str):
    upstream = version.split("-", 1)[0]
    return tuple(int(part) for part in upstream.split("."))

print(max(versions, key=key))
PY
}

check_unpinned_package() {
  local label="$1" ecosystem="$2" package="$3"
  local latest status
  latest=""
  status="latest by default; version pin supported"
  case "${ecosystem}" in
    npm) latest="$(npm_latest "${package}" || true)" ;;
    pypi) latest="$(pypi_latest "${package}" || true)" ;;
    manual) latest="manual review" ;;
  esac
  if [[ -z "${latest}" ]]; then
    latest="lookup failed"
    warnings_found=1
  fi
  printf '| %s | %s | `%s` | `%s` | %s |\n' \
    "${label}" "${ecosystem}" "${package}" "${latest}" "${status}" >> "${REPORT}"
}

check_claude_package() {
  local latest status
  latest="$(claude_apt_latest latest || true)"
  status="latest channel via signed apt repo; pin optional"
  if [[ -z "${latest}" ]]; then
    latest="lookup failed"
    warnings_found=1
  fi
  printf '| %s | %s | `%s` | `%s` | %s |\n' \
    "Claude Code" "apt" "claude-code" "${latest}" "${status}" >> "${REPORT}"
}

{
  echo "# aibox release state"
  echo ""
  echo "- Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  echo "- Workspace: ${PROJECT_ROOT}"
  echo "- Purpose: pre-release evidence for dependency, addon, image, and harness drift."
} > "${REPORT}"

section "Processkit"
current_processkit="$(grep 'pub const PROCESSKIT_DEFAULT_VERSION' "${CLI_DIR}/src/processkit_vocab.rs" \
  | grep -oE '"v[^"]+"' | tr -d '"' || true)"
latest_processkit="$(github_latest_release projectious-work/processkit || true)"
if [[ -z "${latest_processkit}" ]]; then
  status="latest lookup failed"
  warnings_found=1
elif [[ -z "${current_processkit}" ]]; then
  status="missing current pin"
  warnings_found=1
elif [[ "${latest_processkit}" != "${current_processkit}" ]]; then
  mark_update
  status="update available"
else
  status="current"
fi
line "| Component | Current | Latest | Status |"
line "|---|---:|---:|---|"
line "| processkit default | \`${current_processkit:-?}\` | \`${latest_processkit:-?}\` | ${status} |"
line ""
line "If processkit changed, run \`./scripts/maintain.sh sync-processkit\`, review the FORMAT.md diff, then update CLI vocabulary and tests before release."

section "Base Image Tool Pins"
line "| Tool | Current | Latest | Source | Status |"
line "|---|---:|---:|---|---|"
BASE_DOCKERFILE="${PROJECT_ROOT}/images/base-debian/Dockerfile"
line "| tmux | \`Debian package\` | \`Debian security tracker\` | \`apt\` | managed through base image rebuilds |"
check_github_pin "Yazi" "${BASE_DOCKERFILE}" "YAZI_VERSION" "sxyazi/yazi"
check_github_pin "ripgrep" "${BASE_DOCKERFILE}" "RIPGREP_VERSION" "BurntSushi/ripgrep"
check_github_pin "fd" "${BASE_DOCKERFILE}" "FD_VERSION" "sharkdp/fd"
check_github_pin "bat" "${BASE_DOCKERFILE}" "BAT_VERSION" "sharkdp/bat"
check_github_pin "eza" "${BASE_DOCKERFILE}" "EZA_VERSION" "eza-community/eza"
check_github_pin "zoxide" "${BASE_DOCKERFILE}" "ZOXIDE_VERSION" "ajeetdsouza/zoxide"
check_github_pin "fzf" "${BASE_DOCKERFILE}" "FZF_VERSION" "junegunn/fzf"
check_github_pin "delta" "${BASE_DOCKERFILE}" "DELTA_VERSION" "dandavison/delta"
check_github_pin "starship" "${BASE_DOCKERFILE}" "STARSHIP_VERSION" "starship/starship"

section "Unpinned Image Inputs"
line "| Input | Current selector | Latest/Review target | Status |"
line "|---|---|---|---|"
uv_pin="0.11.10"
uv_latest="$(github_latest_release astral-sh/uv || true)"
uv_status="current"
if [[ -z "${uv_latest}" ]]; then
  uv_status="latest lookup failed"
  warnings_found=1
elif version_gt "${uv_latest}" "${uv_pin}"; then
  uv_status="update available"
  mark_update
fi
line "| uv image | \`ghcr.io/astral-sh/uv:${uv_pin}\` | \`${uv_latest:-lookup failed}\` | pinned image tag; ${uv_status} |"
node_latest="$(node_major_latest 22 || true)"
line "| Node.js runtime | \`node_22.x\` / \`node:22-slim\` | \`${node_latest:-check Node.js 22 release stream}\` | floating major; review Node 22 LTS/security status |"
line "| Debian base | \`debian:trixie-slim\` | Debian security tracker | floating distro tag; review base-image rebuild risk |"

section "AI Harness Addons"
line "| Harness | Ecosystem | Package/Installer | Latest | Status |"
line "|---|---|---|---:|---|"
check_claude_package
check_unpinned_package "Codex CLI" "npm" "@openai/codex"
check_unpinned_package "Gemini CLI" "npm" "@google/gemini-cli"
check_unpinned_package "Continue CLI" "npm" "@continuedev/cli"
check_unpinned_package "GitHub Copilot CLI" "npm" "@github/copilot"
check_unpinned_package "Aider" "pypi" "aider-chat"
check_unpinned_package "OpenCode" "manual" "https://opencode.ai/install"
check_unpinned_package "Mistral SDK" "pypi" "mistralai"
line ""
line "For latest-by-default harnesses, check upstream release notes and install-layout changes. If a harness changed project command, skill, config, auth, or binary paths, update aibox projection code and docs before release."

section "Rust Dependencies"
line "### cargo audit"
if command -v cargo-audit >/dev/null 2>&1; then
  if (cd "${CLI_DIR}" && run_with_timeout 120s cargo audit) >> "${REPORT}" 2>&1; then
    line ""
    line "Status: audit clean."
  else
    line ""
    line "Status: cargo audit reported advisories. Release must not proceed until resolved."
    updates_found=1
  fi
else
  line "cargo-audit is not installed in this environment. The release command installs/runs it and aborts on advisories."
  warnings_found=1
fi

line ""
line "### cargo update --dry-run"
if (cd "${CLI_DIR}" && run_with_timeout 120s cargo update --dry-run) >> "${REPORT}" 2>&1; then
  line ""
  line "Status: dry-run completed. Review the output above for lockfile-resolvable crate updates."
  line ""
  line "Disposition required: if updates are listed, either apply them with a real \`cargo update\` and rerun validation, or create a processkit WorkItem for the deferred crate-update pass before continuing the release."
else
  line ""
  line "Status: cargo update --dry-run failed or could not reach the index; rerun before release if network was unavailable."
  warnings_found=1
fi

section "Addon Inventory"
line "Addon manifests present at release time:"
line ""
while IFS= read -r addon; do
  rel="${addon#${PROJECT_ROOT}/}"
  name="$(grep -E '^name:' "${addon}" | head -n 1 | sed -E 's/^name:[[:space:]]*//; s/\"//g')"
  version="$(grep -E '^version:' "${addon}" | head -n 1 | sed -E 's/^version:[[:space:]]*//; s/\"//g')"
  printf -- '- `%s` version `%s` (%s)\n' "${name:-unknown}" "${version:-?}" "${rel}" >> "${REPORT}"
done < <(find "${PROJECT_ROOT}/addons" -name '*.yaml' -type f | sort)

section "Agent Review Checklist"
line "- Read this report before running or continuing \`./scripts/maintain.sh release <version>\`."
line "- For every update available, inspect upstream release notes and decide: bump now, defer explicitly, or file a follow-up issue."
line "- For \`cargo update --dry-run\`, apply available crate updates now or create a processkit WorkItem for the deferred crate-update pass."
line "- For every deferred finding, create a processkit WorkItem in the same turn and mention its ID in release notes or handover."
line "- For every harness, verify install location, binary path, config path, command/skill projection path, auth persistence, and generated devcontainer expectations."
line "- For every Dockerfile pin bump, rebuild the base image and run at least the layout/status/Yazi smoke path if the tool affects runtime UX."
line "- Confirm \`cargo audit\` is clean. The release script also enforces this before building binaries."

section "Summary"
if [[ "${updates_found}" -eq 1 ]]; then
  line "- Dependency or security drift was detected. Agent review is required before release."
else
  line "- No definite newer pinned dependency was detected by this script."
fi
if [[ "${warnings_found}" -eq 1 ]]; then
  line "- Some lookups failed or require manual review."
fi

printf 'Release state report written to %s\n' "${REPORT}"
if [[ "${updates_found}" -eq 1 || "${warnings_found}" -eq 1 ]]; then
  printf 'Review required before release continues.\n'
fi
