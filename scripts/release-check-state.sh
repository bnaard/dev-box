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
ports_failure=0
network_failure_marker="$(mktemp)"
export AIBOX_RELEASE_NETWORK_FAILURE_MARKER="${network_failure_marker}"
trap 'rm -f "${network_failure_marker}"' EXIT

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

record_network_failure() {
  local label="$1"
  warnings_found=1
  printf 'NETWORK_LOOKUP_FAILED: %s\n' "${label}" >&2
  printf '%s\n' "${label}" >> "${network_failure_marker}"
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
  local output
  if output="$(run_with_timeout 8s gh api "repos/${repo}/releases/latest" --jq '.tag_name' 2>&1)"; then
    printf '%s\n' "${output}"
    return 0
  fi
  if grep -Eiq 'could not resolve|temporary failure|no such host|network is unreachable|connection timed out|i/o timeout|error connecting' <<<"${output}"; then
    record_network_failure "GitHub release lookup for ${repo}: ${output//$'\n'/ }"
  fi
  return 1
}

dockerfile_arg() {
  local file="$1" arg="$2"
  grep -E "^ARG ${arg}=" "${file}" | head -n 1 | sed -E "s/^ARG ${arg}=//"
}

quoted_assignment() {
  local file="$1" name="$2"
  sed -nE "s/.*${name}=\"([^\"]+)\".*/\\1/p" "${file}" | head -n 1
}

package_pin() {
  local file="$1" package="$2"
  grep -oE "${package}==[0-9]+([.][0-9]+)+" "${file}" \
    | head -n 1 \
    | sed -E "s/^${package}==//"
}

container_image_tag() {
  local file="$1" image="$2"
  grep -oE "${image}:[0-9]+([.][0-9]+)+" "${file}" \
    | head -n 1 \
    | sed -E "s#^${image}:##"
}

sha256_file() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file}" | awk '{print $1}'
  else
    shasum -a 256 "${file}" | awk '{print $1}'
  fi
}

sha256_stdin() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print $1}'
  else
    shasum -a 256 | awk '{print $1}'
  fi
}

image_source_sha() {
  local image_dir="${PROJECT_ROOT}/images/base-debian"
  while IFS= read -r file; do
    local rel="${file#${PROJECT_ROOT}/}"
    printf '%s  %s\n' "$(sha256_file "${file}")" "${rel}"
  done < <(find "${image_dir}" -type f | LC_ALL=C sort) | sha256_stdin
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
  local output
  if output="$(run_with_timeout 12s npm view "${package}" version 2>&1)"; then
    printf '%s\n' "${output}"
    return 0
  fi
  if grep -Eiq 'EAI_AGAIN|ENOTFOUND|ETIMEOUT|network|could not resolve|getaddrinfo' <<<"${output}"; then
    record_network_failure "npm lookup for ${package}: ${output//$'\n'/ }"
  fi
  return 1
}

pypi_latest() {
  local package="$1"
  if ! command -v python3 >/dev/null 2>&1; then
    return 1
  fi
  python3 - "$package" <<'PY'
import json
import os
import socket
import sys
import urllib.error
import urllib.request

package = sys.argv[1]
url = f"https://pypi.org/pypi/{package}/json"
try:
    with urllib.request.urlopen(url, timeout=12) as response:
        print(json.load(response)["info"]["version"])
except (TimeoutError, OSError, urllib.error.URLError, socket.gaierror) as exc:
    message = f"PyPI lookup for {package}: {exc}"
    print(f"NETWORK_LOOKUP_FAILED: {message}", file=sys.stderr)
    marker = os.environ.get("AIBOX_RELEASE_NETWORK_FAILURE_MARKER")
    if marker:
        with open(marker, "a", encoding="utf-8") as handle:
            handle.write(message + "\n")
    raise SystemExit(75)
PY
}

node_major_latest() {
  local major="$1"
  if ! command -v python3 >/dev/null 2>&1; then
    return 1
  fi
  python3 - "$major" <<'PY'
import json
import os
import socket
import sys
import urllib.error
import urllib.request

major = sys.argv[1]
url = "https://nodejs.org/dist/index.json"
try:
    with urllib.request.urlopen(url, timeout=12) as response:
        releases = json.load(response)
except (TimeoutError, OSError, urllib.error.URLError, socket.gaierror) as exc:
    message = f"Node.js release stream lookup for {major}: {exc}"
    print(f"NETWORK_LOOKUP_FAILED: {message}", file=sys.stderr)
    marker = os.environ.get("AIBOX_RELEASE_NETWORK_FAILURE_MARKER")
    if marker:
        with open(marker, "a", encoding="utf-8") as handle:
            handle.write(message + "\n")
    raise SystemExit(75)
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
import os
import re
import socket
import sys
import urllib.error
import urllib.request

channel = sys.argv[1]
url = f"https://downloads.claude.ai/claude-code/apt/{channel}/dists/{channel}/main/binary-amd64/Packages"
try:
    with urllib.request.urlopen(url, timeout=12) as response:
        body = response.read().decode("utf-8", "replace")
except (TimeoutError, OSError, urllib.error.URLError, socket.gaierror) as exc:
    message = f"Claude apt lookup for channel {channel}: {exc}"
    print(f"NETWORK_LOOKUP_FAILED: {message}", file=sys.stderr)
    marker = os.environ.get("AIBOX_RELEASE_NETWORK_FAILURE_MARKER")
    if marker:
        with open(marker, "a", encoding="utf-8") as handle:
            handle.write(message + "\n")
    raise SystemExit(75)
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

addon_tool_default() {
  local file="$1" tool="$2"
  python3 - "$file" "$tool" <<'PY'
import re
import sys

path, tool = sys.argv[1], sys.argv[2]
in_tool = False
with open(path, encoding="utf-8") as handle:
    for raw in handle:
        line = raw.rstrip("\n")
        if re.match(r"\s*-\s+name:\s*['\"]?" + re.escape(tool) + r"['\"]?\s*$", line):
            in_tool = True
            continue
        if in_tool and re.match(r"\s*-\s+name:", line):
            break
        if in_tool:
            match = re.match(r'\s*default_version:\s*"([^"]*)"', line)
            if match:
                print(match.group(1))
                raise SystemExit(0)
raise SystemExit(1)
PY
}

apt_candidate_version() {
  local package="$1"
  if ! command -v apt-cache >/dev/null 2>&1; then
    return 1
  fi
  apt-cache policy "${package}" 2>/dev/null \
    | awk '/Candidate:/ {print $2; exit}' \
    | sed 's/(none)//'
}

check_addon_version() {
  local label="$1" current="$2" latest="$3" source="$4" status="${5:-}"
  if [[ -n "${status}" ]]; then
    printf '| %s | `%s` | `%s` | %s | %s |\n' \
      "${label}" "${current:-?}" "${latest:-?}" "${source}" "${status}" >> "${REPORT}"
    return 0
  fi
  status="current"
  if [[ -z "${current}" ]]; then
    status="missing current pin"
    warnings_found=1
  elif [[ -z "${latest}" ]]; then
    status="latest lookup failed"
    warnings_found=1
  elif version_gt "${latest}" "${current}"; then
    status="update available"
    mark_update
  fi
  printf '| %s | `%s` | `%s` | %s | %s |\n' \
    "${label}" "${current:-?}" "${latest:-?}" "${source}" "${status}" >> "${REPORT}"
}

check_addon_tool_pypi() {
  local label="$1" file="$2" tool="$3" package="$4"
  check_addon_version "${label}" \
    "$(addon_tool_default "${file}" "${tool}" || true)" \
    "$(pypi_latest "${package}" || true)" \
    "PyPI ${package}"
}

check_addon_tool_npm() {
  local label="$1" file="$2" tool="$3" package="$4"
  check_addon_version "${label}" \
    "$(addon_tool_default "${file}" "${tool}" || true)" \
    "$(npm_latest "${package}" || true)" \
    "npm ${package}"
}

check_addon_tool_github() {
  local label="$1" file="$2" tool="$3" repo="$4" latest
  latest="$(github_latest_release "${repo}" || true)"
  latest="$(normalize_version "${latest}")"
  check_addon_version "${label}" \
    "$(addon_tool_default "${file}" "${tool}" || true)" \
    "${latest}" \
    "GitHub ${repo}"
}

kubectl_latest() {
  local output
  if output="$(run_with_timeout 8s curl -fsSL https://dl.k8s.io/release/stable.txt 2>&1)"; then
    normalize_version "${output}"
    return 0
  fi
  if grep -Eiq 'could not resolve|temporary failure|no such host|network is unreachable|connection timed out|i/o timeout|error connecting' <<<"${output}"; then
    record_network_failure "kubectl stable lookup: ${output//$'\n'/ }"
  fi
  return 1
}

go_latest() {
  local output
  if output="$(run_with_timeout 8s curl -fsSL https://go.dev/VERSION?m=text 2>&1 | head -n 1)"; then
    printf '%s\n' "${output#go}"
    return 0
  fi
  if grep -Eiq 'could not resolve|temporary failure|no such host|network is unreachable|connection timed out|i/o timeout|error connecting' <<<"${output}"; then
    record_network_failure "Go latest lookup: ${output//$'\n'/ }"
  fi
  return 1
}

rust_latest() {
  local output
  if output="$(run_with_timeout 8s curl -fsSL https://static.rust-lang.org/dist/channel-rust-stable.toml 2>&1)"; then
    awk '
      /^\[pkg\.rust\]$/ { in_rust = 1; next }
      in_rust && /^version = / {
        value = $0
        sub(/^version = "/, "", value)
        sub(/ .*/, "", value)
        print value
        exit
      }
    ' <<<"${output}"
    return 0
  fi
  if grep -Eiq 'could not resolve|temporary failure|no such host|network is unreachable|connection timed out|i/o timeout|error connecting' <<<"${output}"; then
    record_network_failure "Rust stable lookup: ${output//$'\n'/ }"
  fi
  return 1
}

check_apt_managed() {
  local label="$1" package="$2" candidate
  candidate="$(apt_candidate_version "${package}" || true)"
  printf '| %s | `%s` | `%s` | %s |\n' \
    "${label}" "${package}" "${candidate:-?}" "Debian apt managed; review distro/security updates during base-image refresh" >> "${REPORT}"
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
base_image_source_sha="$(image_source_sha)"
line ""
line "Source hash for release cache decisions: \`${base_image_source_sha}\`."
line ""
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
uv_pin="$(container_image_tag "${BASE_DOCKERFILE}" "ghcr.io/astral-sh/uv" || true)"
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
node_default="$(addon_tool_default "${PROJECT_ROOT}/addons/languages/node.yaml" node || true)"
node_latest="$(node_major_latest "${node_default:-26}" || true)"
line "| Node.js runtime | \`node_${node_default:-?}.x\` / \`node:${node_default:-?}-slim\` | \`${node_latest:-check Node.js ${node_default:-current} release stream}\` | floating major; review Node ${node_default:-current} release/security status |"
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
check_unpinned_package "Hermes Agent" "pypi" "hermes-agent"
check_unpinned_package "Mistral SDK" "pypi" "mistralai"
line ""
line "For latest-by-default harnesses, check upstream release notes and install-layout changes. If a harness changed project command, skill, config, auth, or binary paths, update aibox projection code and docs before release."

section "Addon Tool Version Pins"
line "| Tool | Current | Latest | Source | Status |"
line "|---|---:|---:|---|---|"
check_addon_version "Hugo" "$(quoted_assignment "${PROJECT_ROOT}/addons/docs/docs-hugo.yaml" HUGO_VERSION || true)" "$(normalize_version "$(github_latest_release gohugoio/hugo || true)")" "GitHub gohugoio/hugo"
check_addon_version "mdBook" "$(quoted_assignment "${PROJECT_ROOT}/addons/docs/docs-mdbook.yaml" MDBOOK_VERSION || true)" "$(normalize_version "$(github_latest_release rust-lang/mdBook || true)")" "GitHub rust-lang/mdBook"
check_addon_tool_pypi "MkDocs" "${PROJECT_ROOT}/addons/docs/docs-mkdocs.yaml" "mkdocs" "mkdocs"
check_addon_version "MkDocs Material" "$(package_pin "${PROJECT_ROOT}/addons/docs/docs-mkdocs.yaml" mkdocs-material || true)" "$(pypi_latest mkdocs-material || true)" "PyPI mkdocs-material"
check_addon_tool_pypi "Zensical" "${PROJECT_ROOT}/addons/docs/docs-zensical.yaml" "zensical" "zensical"
check_addon_version "Starlight scaffold" "floating" "$(npm_latest create-starlight || true)" "npm create-starlight" "latest by default"
check_addon_version "Go" "$(addon_tool_default "${PROJECT_ROOT}/addons/languages/go.yaml" go || true)" "$(go_latest || true)" "go.dev"
check_addon_version "Rust" "$(addon_tool_default "${PROJECT_ROOT}/addons/languages/rust.yaml" rustc || true)" "$(rust_latest || true)" "rustup stable"
check_addon_tool_npm "pnpm" "${PROJECT_ROOT}/addons/languages/node.yaml" "pnpm" "pnpm"
check_addon_tool_npm "Bun" "${PROJECT_ROOT}/addons/languages/node.yaml" "bun" "bun"
check_addon_version "Yarn" "$(addon_tool_default "${PROJECT_ROOT}/addons/languages/node.yaml" yarn || true)" "Berry/current via corepack" "corepack" "manual review"
check_addon_version "Python interpreter" "$(addon_tool_default "${PROJECT_ROOT}/addons/languages/python.yaml" python || true)" "curated via uv python install" "addon default" "curated default"
check_addon_tool_pypi "Poetry" "${PROJECT_ROOT}/addons/languages/python.yaml" "poetry" "poetry"
check_addon_tool_pypi "PDM" "${PROJECT_ROOT}/addons/languages/python.yaml" "pdm" "pdm"
check_addon_tool_github "OpenTofu" "${PROJECT_ROOT}/addons/tools/infrastructure.yaml" "opentofu" "opentofu/opentofu"
check_addon_tool_pypi "Ansible" "${PROJECT_ROOT}/addons/tools/infrastructure.yaml" "ansible" "ansible"
check_addon_tool_github "Packer" "${PROJECT_ROOT}/addons/tools/infrastructure.yaml" "packer" "hashicorp/packer"
check_addon_version "kubectl" "$(addon_tool_default "${PROJECT_ROOT}/addons/tools/kubernetes.yaml" kubectl || true)" "$(kubectl_latest || true)" "dl.k8s.io stable"
check_addon_tool_github "Helm" "${PROJECT_ROOT}/addons/tools/kubernetes.yaml" "helm" "helm/helm"
kustomize_latest="$(github_latest_release kubernetes-sigs/kustomize || true)"
kustomize_latest="${kustomize_latest#kustomize/}"
kustomize_latest="$(normalize_version "${kustomize_latest}")"
check_addon_version "Kustomize" "$(addon_tool_default "${PROJECT_ROOT}/addons/tools/kubernetes.yaml" kustomize || true)" "${kustomize_latest}" "GitHub kubernetes-sigs/kustomize"
check_addon_tool_github "k9s" "${PROJECT_ROOT}/addons/tools/kubernetes.yaml" "k9s" "derailed/k9s"
check_addon_tool_github "OpenCode" "${PROJECT_ROOT}/addons/ai/ai-opencode.yaml" "opencode" "opencode-ai/opencode"
check_addon_tool_pypi "Hermes Agent" "${PROJECT_ROOT}/addons/ai/ai-hermes.yaml" "hermes" "hermes-agent"
check_addon_tool_pypi "Tau" "${PROJECT_ROOT}/addons/ai/ai-tau.yaml" "tau" "tau-ai"
line ""
line "Python defaults are curated in the addon catalog and installed through uv when they differ from the Debian base interpreter."

section "LaTeX And Apt-Managed Addon Inputs"
line "| Tool/Input | Package | Candidate | Status |"
line "|---|---|---:|---|"
line "| TeX Live mirror | \`texlive.info/historic/systems/texlive/2025/tlnet-final\` | \`2025 final\` | pinned immutable TeX Live archive; update requires a deliberate TeX Live year bump and rebuild |"
check_apt_managed "LaTeX runtime Perl" "perl"
check_apt_managed "LaTeX fontconfig" "fontconfig"
check_apt_managed "Emoji fonts" "fonts-noto-color-emoji"
check_apt_managed "SVG inclusion" "inkscape"
check_apt_managed "PDF utilities" "poppler-utils"
check_apt_managed "LilyPond" "lilypond"
line ""
line "LaTeX packages installed through \`tlmgr\` are governed by the pinned TeX Live 2025 final repository. Do not treat them as floating apt packages."

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
if ! command -v cargo >/dev/null 2>&1; then
  line "cargo is not available on PATH in this environment. Run this check in the release container before publishing."
  warnings_found=1
else
  cargo_update_output=""
  if cargo_update_output="$(cd "${CLI_DIR}" && run_with_timeout 120s cargo update --dry-run 2>&1)"; then
    printf '%s\n' "${cargo_update_output}" >> "${REPORT}"
    line ""
    if grep -Eq 'Locking 0 packages' <<<"${cargo_update_output}"; then
      line "Status: Cargo.lock is current for the active Rust toolchain."
    else
      line "Status: lockfile-resolvable crate updates are available."
      line ""
      line "Disposition required: apply them with a real \`cargo update\` and rerun validation, or create a processkit WorkItem for the deferred crate-update pass before continuing the release."
      mark_update
    fi
  else
    printf '%s\n' "${cargo_update_output}" >> "${REPORT}"
    line ""
    line "Status: cargo update --dry-run failed or could not reach the index; rerun before release if network was unavailable."
    warnings_found=1
  fi
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

section "Version-Line Port Gate"
release_major="$(sed -nE 's/^version = "([0-9]+)\..*/\1/p' "${CLI_DIR}/Cargo.toml" | head -n 1)"
if "${SCRIPT_DIR}/check-version-line-ports.sh" check "v${release_major}" >> "${REPORT}" 2>&1; then
  line ""
  line "Status: no open cross-line port obligations target this release line."
else
  ports_failure=1
  line ""
  line "Status: release blocked by an open cross-line port obligation."
fi

section "Summary"
if [[ "${updates_found}" -eq 1 ]]; then
  line "- Dependency or security drift was detected. Agent review is required before release."
else
  line "- No definite newer pinned dependency was detected by this script."
fi
if [[ "${warnings_found}" -eq 1 ]]; then
  line "- Some lookups failed or require manual review."
fi

if [[ -s "${network_failure_marker}" ]]; then
  line "- Network lookups failed during this report. Re-run from a shell with DNS/network access before releasing."
  section "Network Lookup Failures"
  while IFS= read -r failure; do
    line "- ${failure}"
  done < "${network_failure_marker}"
fi

printf 'Release state report written to %s\n' "${REPORT}"
if [[ "${updates_found}" -eq 1 || "${warnings_found}" -eq 1 ]]; then
  printf 'Review required before release continues.\n'
fi
if [[ "${ports_failure}" -eq 1 ]]; then
  printf 'ERR: open version-line port obligations block this release. See %s.\n' "${REPORT}" >&2
  exit 1
fi
if [[ -s "${network_failure_marker}" && "${AIBOX_RELEASE_REQUIRE_NETWORK:-0}" == "1" ]]; then
  printf 'ERR: release-check-state requires network access for release gating; DNS/network lookup failed. See %s.\n' "${REPORT}" >&2
  exit 75
fi
