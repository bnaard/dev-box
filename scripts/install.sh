#!/usr/bin/env bash
# =============================================================================
# install.sh — install aibox CLI from GitHub releases
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash
#
# Options (via environment variables):
#   VERSION=0.2.0    Install a specific version (default: latest)
#   INSTALL_DIR=...  Install to a custom directory (default: ~/.local/bin)
#
# Examples:
#   # Install latest
#   curl -fsSL .../install.sh | bash
#
#   # Install specific version
#   curl -fsSL .../install.sh | VERSION=0.1.0 bash
#
#   # Install to /usr/local/bin (requires sudo)
#   curl -fsSL .../install.sh | INSTALL_DIR=/usr/local/bin sudo -E bash
# =============================================================================
set -euo pipefail

REPO="projectious-work/aibox"
BINARY_NAME="aibox"
DEFAULT_INSTALL_DIR="${HOME}/.local/bin"

# ── Colours (disabled if not a terminal) ─────────────────────────────────────
if [[ -t 1 ]]; then
  bold=$'\e[1m'
  cyan=$'\e[36m'
  red=$'\e[31m'
  green=$'\e[32m'
  yellow=$'\e[33m'
  reset=$'\e[0m'
else
  bold="" cyan="" red="" green="" yellow="" reset=""
fi

info()  { echo "${cyan}${bold}==>${reset} $*"; }
ok()    { echo "${green}${bold} ✓${reset} $*"; }
warn()  { echo "${yellow}${bold}  !${reset} $*"; }
die()   { echo "${red}${bold}ERR${reset} $*" >&2; exit 1; }

# ── Detect platform ─────────────────────────────────────────────────────────
detect_platform() {
  local os arch

  os="$(uname -s)"
  case "${os}" in
    Linux)  os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *)      die "Unsupported operating system: ${os}" ;;
  esac

  arch="$(uname -m)"
  case "${arch}" in
    x86_64)         arch="x86_64" ;;
    aarch64|arm64)  arch="aarch64" ;;
    *)              die "Unsupported architecture: ${arch}" ;;
  esac

  echo "${arch}-${os}"
}

# ── Resolve version ─────────────────────────────────────────────────────────
resolve_version() {
  if [[ -n "${VERSION:-}" ]]; then
    echo "${VERSION}"
    return
  fi

  info "Fetching latest release..." >&2
  local latest
  latest=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | head -1 \
    | sed -E 's/.*"tag_name":[ ]*"v?([^"]+)".*/\1/')

  if [[ -z "${latest}" ]]; then
    die "Could not determine latest version. Set VERSION=x.y.z to install a specific version."
  fi

  echo "${latest}"
}

# ── Check for existing installation ──────────────────────────────────────────
check_existing() {
  local install_dir="$1" version="$2"
  if [[ -x "${install_dir}/${BINARY_NAME}" ]]; then
    local current
    current=$("${install_dir}/${BINARY_NAME}" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
    if [[ "${current}" == "${version}" ]]; then
      ok "aibox v${version} is already installed at ${install_dir}/${BINARY_NAME}"
      exit 0
    fi
    warn "Upgrading aibox from v${current} to v${version}"
  fi
}

# ── Main ─────────────────────────────────────────────────────────────────────
main() {
  local platform version install_dir tarball_name download_url tmpdir

  platform=$(detect_platform)
  version=$(resolve_version)
  install_dir="${INSTALL_DIR:-${DEFAULT_INSTALL_DIR}}"

  info "Installing aibox v${version} for ${platform}"

  check_existing "${install_dir}" "${version}"

  tarball_name="${BINARY_NAME}-v${version}-${platform}.tar.gz"
  download_url="https://github.com/${REPO}/releases/download/v${version}/${tarball_name}"

  # Download
  tmpdir=$(mktemp -d)
  trap 'rm -rf "${tmpdir:-}"' EXIT

  info "Downloading ${tarball_name}..."
  if ! curl -fsSL -o "${tmpdir}/${tarball_name}" "${download_url}"; then
    echo ""
    die "Download failed. Check that release v${version} exists and has a binary for ${platform}.
    URL: ${download_url}
    Releases: https://github.com/${REPO}/releases"
  fi
  ok "Downloaded"

  # Extract
  info "Extracting..."
  tar -xzf "${tmpdir}/${tarball_name}" -C "${tmpdir}"
  ok "Extracted"

  # Install
  mkdir -p "${install_dir}"
  mv "${tmpdir}/${BINARY_NAME}-v${version}-${platform}" "${install_dir}/${BINARY_NAME}"
  chmod +x "${install_dir}/${BINARY_NAME}"
  ok "Installed to ${install_dir}/${BINARY_NAME}"

  # Install addon definitions (YAML files from the repo)
  local addons_dir="${XDG_CONFIG_HOME:-${HOME}/.config}/aibox/addons"
  local addons_base_url="https://raw.githubusercontent.com/${REPO}/v${version}/addons"

  info "Installing addon definitions to ${addons_dir}..."
  local categories="languages tools docs ai"
  for category in ${categories}; do
    mkdir -p "${addons_dir}/${category}"
  done

  # Download each addon YAML file
  local addon_files="
    languages/python.yaml
    languages/rust.yaml
    languages/node.yaml
    languages/go.yaml
    languages/typst.yaml
    languages/latex.yaml
    tools/infrastructure.yaml
    tools/kubernetes.yaml
    tools/cloud-aws.yaml
    tools/cloud-gcp.yaml
    tools/cloud-azure.yaml
    docs/docs-mkdocs.yaml
    docs/docs-zensical.yaml
    docs/docs-docusaurus.yaml
    docs/docs-starlight.yaml
    docs/docs-mdbook.yaml
    docs/docs-hugo.yaml
    ai/ai-claude.yaml
    ai/ai-aider.yaml
    ai/ai-gemini.yaml
    ai/ai-mistral.yaml
  "
  local failed=0
  for file in ${addon_files}; do
    if ! curl -fsSL -o "${addons_dir}/${file}" "${addons_base_url}/${file}" 2>/dev/null; then
      warn "Failed to download addon: ${file}"
      failed=$((failed + 1))
    fi
  done

  if [[ "${failed}" -eq 0 ]]; then
    ok "Installed 21 addon definitions"
  else
    warn "Installed with ${failed} addon download failures — re-run to retry"
  fi

  # Verify
  if "${install_dir}/${BINARY_NAME}" --help &>/dev/null; then
    ok "aibox v${version} is ready"
  else
    warn "Binary installed but failed to execute — check your system compatibility"
  fi

  # PATH check
  if ! echo "${PATH}" | tr ':' '\n' | grep -qx "${install_dir}"; then
    echo ""
    warn "${install_dir} is not in your PATH"
    echo ""
    echo "  Add it to your shell profile:"
    echo ""
    if [[ "${SHELL}" == *"zsh"* ]]; then
      echo "    echo 'export PATH=\"${install_dir}:\$PATH\"' >> ~/.zshrc"
      echo "    source ~/.zshrc"
    else
      echo "    echo 'export PATH=\"${install_dir}:\$PATH\"' >> ~/.bashrc"
      echo "    source ~/.bashrc"
    fi
    echo ""
  fi
}

main
