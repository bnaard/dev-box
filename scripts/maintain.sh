#!/usr/bin/env bash
# =============================================================================
# maintain.sh — maintenance script for the aibox project itself
#
# This manages the dev-container we develop IN (not the containers we publish).
# For downstream project container management, use the aibox CLI.
#
# Usage:
#   ./scripts/maintain.sh <command> [options]
#
# Commands:
#   test              Run cargo fmt, clippy, and tests
#   test-e2e          Run SSH companion E2E tests
#   test-e2e-visual   Run all opt-in SSH/asciinema visual E2E tiers
#   build-images      Build published foundation/runtime images locally
#   release-runtime-smoke <version> Run generated runtime smoke against a release
#   docs-serve        Serve Hugo/Docsy locally for preview
#   docs-deploy       Build Hugo/Docsy and push HTML to gh-pages
#   release <version> Tag, build, compile CLI, generate release prompt
#   start             Start this project's dev-container
#   stop              Stop this project's dev-container
#   attach            Attach to running dev-container via tmux
#   status            Show dev-container status
#   help              Show this help
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── Paths ────────────────────────────────────────────────────────────────────
DEVCONTAINER_DIR="${PROJECT_ROOT}/.devcontainer"
COMPOSE_FILE="${DEVCONTAINER_DIR}/docker-compose.yml"
COMPOSE_OVERRIDE_FILE="${DEVCONTAINER_DIR}/docker-compose.override.yml"
HOST_ROOT="${HOST_ROOT:-${PROJECT_ROOT}/.aibox-home}"
WORKSPACE_DIR="${WORKSPACE_DIR:-${PROJECT_ROOT}}"
CLI_DIR="${PROJECT_ROOT}/cli"
DIST_DIR="${PROJECT_ROOT}/dist"
IMAGE_REGISTRY="ghcr.io/projectious-work/aibox"
GITHUB_REPO="${AIBOX_GITHUB_REPO:-projectious-work/aibox}"

# ── Read container name from docker-compose.yml ─────────────────────────────
_init_names() {
  local svc cn
  svc=$(grep -E '^\s{2}[a-zA-Z0-9_-]+:' "${COMPOSE_FILE}" | head -1 | tr -d ' :')
  cn=$(grep 'container_name:' "${COMPOSE_FILE}" | head -1 | awk '{print $2}')
  SERVICE_NAME="${svc}"
  CONTAINER_NAME="${cn:-${svc}}"
}
_init_names

compose_args() {
  printf -- '-f\n%s\n' "${COMPOSE_FILE}"
  if [[ -s "${COMPOSE_OVERRIDE_FILE}" ]]; then
    printf -- '-f\n%s\n' "${COMPOSE_OVERRIDE_FILE}"
  fi
}

compose() {
  local args=()
  mapfile -t args < <(compose_args)
  ${COMPOSE_BIN} "${args[@]}" "$@"
}

# ── Colours ──────────────────────────────────────────────────────────────────
bold=$'\e[1m'
cyan=$'\e[36m'
yellow=$'\e[33m'
red=$'\e[31m'
green=$'\e[32m'
reset=$'\e[0m'

info()  { echo "${cyan}${bold}==>${reset} $*"; }
ok()    { echo "${green}${bold} ✓${reset} $*"; }
warn()  { echo "${yellow}${bold}  !${reset} $*"; }
die()   { echo "${red}${bold}ERR${reset} $*" >&2; exit 1; }

# ── Resolve container runtime ────────────────────────────────────────────────
# Check which runtime is actually functional, not just installed on PATH.
# Podman may exist as a compatibility shim (e.g., OrbStack) but not be running.
if command -v docker &>/dev/null && docker info &>/dev/null 2>&1; then
  COMPOSE_BIN="docker compose"
  RUNTIME_BIN="docker"
elif command -v podman &>/dev/null && podman info &>/dev/null 2>&1; then
  COMPOSE_BIN="podman compose"
  RUNTIME_BIN="podman"
else
  # Not fatal — some commands (test, docs) don't need a runtime
  COMPOSE_BIN=""
  RUNTIME_BIN=""
fi

# ── Help ─────────────────────────────────────────────────────────────────────
usage() {
  cat <<HELP
${bold}maintain.sh${reset} — aibox project maintenance

${bold}Usage:${reset}
  ./scripts/maintain.sh <command> [options]

${bold}Development:${reset}
  test                     Run cargo fmt check, clippy, and tests
  test-e2e                 Run Tier 2 SSH companion E2E tests
  test-e2e-visual-status   Run opt-in visual matrix for layouts/themes/status rows
  test-e2e-visual-tabs     Run opt-in tab traversal for tools and harnesses
  test-e2e-visual-yazi     Run opt-in Yazi previews/git/plugin visual checks
  test-e2e-visual          Run all opt-in visual E2E tiers
  test-e2e-render-starship Run Tier 3 vt100 cell-color tests for Starship (local, ~6s)
  test-e2e-render-tmux     Run Tier 3 vt100 cell-color tests for tmux (companion)
  test-e2e-render-layout-switch
                           Run Tier 3 rendered test: live layout switch (companion)
  test-e2e-render-theme-switch
                           Run Tier 3 rendered test: live theme switch (companion)
  test-e2e-render-yazi     Run Tier 3 vt100 cell-color tests for Yazi (companion)
  test-e2e-render          Run all Tier 3 vt100 rendered-color tests
  test-e2e-doc-captures    Run visual E2E and write docs-ready cast/screen artifacts
                           Set AIBOX_E2E_VISUAL_FULL_MATRIX=1 for exhaustive layout/theme coverage
  build-images [--no-cache] Build published container images locally
  push-images <version>    Push images to GHCR (requires ghcr.io login)
  ghcr-prune-source-tags [--repair-mixed] [--execute]
                           Plan, repair, or delete GHCR source-hash package versions
  ghcr-prune-buildcache-tags [--execute]
                           Plan or delete GHCR BuildKit cache package versions
  release-runtime-smoke <version>
                           Run host-side generated-runtime smoke and write logs
  docs-serve               Serve Hugo/Docsy locally (http://localhost:1316/aibox/)
  docs-deploy --line <v0.x|v1.x> [--version vX.Y.Z] [--dry-run]
                           Build Hugo/Docsy, retain release snapshots, and push gh-pages
  test-visual              Run screencast smoke tests (~40s)
  record-docs              Regenerate all docs screencasts + README GIF

${bold}Release:${reset}
  sync-processkit          Check for new processkit release; patch constants + show diff
                           (runs automatically inside 'release'; also available standalone)
  release-check-state      Write dist/RELEASE-STATE.md with dependency, addon,
                           image, and harness version drift evidence
  release-doctors          Run pk-doctor + aibox doctor; write dist/RELEASE-DOCTORS.md;
                           exit nonzero if either reports ERRORs
  release <version> [--steps list]
                           Run selected release steps. Use comma-separated steps
                           or aliases: all, phase0, checks, build, publish.
                           Add --skip list to exclude long or already-run steps.
  release <version> --list-steps
                           Print release step aliases and concrete step names
  release-host <version>   Build/upload macOS binaries, push GHCR images,
                           run runtime smoke, then refresh + commit generated runtime surfaces
  release-finalize-runtime <version>
                           Refresh and commit repo-owned generated runtime files

${bold}Container (this project's dev-container):${reset}
  start                    Ensure running, then attach via tmux
  stop                     Stop the dev-container
  attach                   Attach to running dev-container
  status                   Show dev-container status
  help                     Show this help
HELP
}

# =============================================================================
# Container helpers (from the original dev.sh)
# =============================================================================

_require_runtime() {
  [[ -n "${RUNTIME_BIN}" ]] || die "Neither podman nor docker found."
}

container_status() {
  _require_runtime
  local state
  state=$(${RUNTIME_BIN} inspect --format '{{.State.Status}}' "${CONTAINER_NAME}" 2>/dev/null || true)
  case "${state}" in
    running)        echo "running" ;;
    exited|stopped) echo "exited"  ;;
    *)              echo "missing" ;;
  esac
}

wait_for_running() {
  local attempts=0 max=15
  while [[ $attempts -lt $max ]]; do
    if [[ "$(container_status)" == "running" ]]; then
      return 0
    fi
    sleep 0.5
    (( attempts++ ))
  done
  die "Container did not reach running state."
}

seed_file() {
  local src="$1" dest="$2"
  if [[ ! -f "${dest}" && -f "${src}" ]]; then
    warn "Seeding $(realpath --relative-to="${PROJECT_ROOT}" "${dest}")"
    mkdir -p "$(dirname "${dest}")"
    cp "${src}" "${dest}"
  fi
}

ensure_host_dirs() {
  info "Checking host directories..."
  mkdir -p "${HOST_ROOT}"/{.ssh,.vim/undo,.config/tmux/layouts,.tmux/plugins,.config/yazi,.config/git,.claude}

  seed_file "${DEVCONTAINER_DIR}/config/vimrc"                        "${HOST_ROOT}/.vim/vimrc"
  seed_file "${DEVCONTAINER_DIR}/config/gitconfig"                     "${HOST_ROOT}/.config/git/config"
  seed_file "${DEVCONTAINER_DIR}/config/tmux/tmux.conf"                "${HOST_ROOT}/.config/tmux/tmux.conf"
  seed_file "${DEVCONTAINER_DIR}/config/yazi/yazi.toml"                "${HOST_ROOT}/.config/yazi/yazi.toml"
  seed_file "${DEVCONTAINER_DIR}/config/yazi/keymap.toml"              "${HOST_ROOT}/.config/yazi/keymap.toml"
  seed_file "${DEVCONTAINER_DIR}/config/yazi/theme.toml"               "${HOST_ROOT}/.config/yazi/theme.toml"

  if [[ -z "$(ls -A "${HOST_ROOT}/.ssh" 2>/dev/null)" ]]; then
    warn "No SSH keys in ${HOST_ROOT}/.ssh"
  fi
}

# =============================================================================
# Commands
# =============================================================================

cmd_test() {
  info "Running cargo fmt check..."
  (cd "${CLI_DIR}" && cargo fmt --check) || die "Format check failed. Run: cd cli && cargo fmt"
  ok "Format OK"

  info "Running clippy..."
  (cd "${CLI_DIR}" && cargo clippy -- -D warnings) || die "Clippy failed"
  ok "Clippy OK"

  info "Running tests..."
  (cd "${CLI_DIR}" && cargo test) || die "Tests failed"
  ok "All tests passed"
}

ensure_e2e_companion() {
  local key="${PROJECT_ROOT}/.aibox-e2e-runner-home/.ssh/id_ed25519"
  local host="${AIBOX_E2E_HOST:-aibox-e2e-testrunner}"
  info "Checking SSH companion E2E container..."
  local ssh_output=""
  if [[ -f "${key}" ]] && ssh_output=$(ssh -i "${key}" \
      -o StrictHostKeyChecking=no \
      -o UserKnownHostsFile=/dev/null \
      -o ConnectTimeout=5 \
      -o LogLevel=ERROR \
      "testuser@${host}" 'echo ok' 2>&1); then
    if grep -qx ok <<<"${ssh_output}"; then
      ok "SSH companion E2E container is reachable"
      return
    fi
  fi

  if grep -Eiq 'could not resolve|temporary failure|name or service not known|no such host' <<<"${ssh_output}"; then
    warn "SSH companion host '${host}' did not resolve. In restricted Codex sandboxes, Docker DNS for the companion is often unavailable."
    warn "For partial release validation, use --skip e2e,visual or select steps that do not need the companion."
  fi
  _require_runtime
  info "SSH companion not reachable; starting aibox-e2e-testrunner via Compose..."
  compose up -d aibox-e2e-testrunner \
    || die "Failed to start aibox-e2e-testrunner"
}

prune_e2e_companion_storage() {
  local key="${PROJECT_ROOT}/.aibox-e2e-runner-home/.ssh/id_ed25519"
  local host="${AIBOX_E2E_HOST:-aibox-e2e-testrunner}"
  ensure_e2e_companion
  info "Pruning SSH companion nested runtime state..."
  ssh -i "${key}" \
      -o StrictHostKeyChecking=no \
      -o UserKnownHostsFile=/dev/null \
      -o ConnectTimeout=5 \
      -o LogLevel=ERROR \
      "testuser@${host}" \
      'runtime=""; if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then runtime=docker; elif command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; then runtime=podman; fi; test -n "$runtime" || exit 0; for workspace in /workspaces/*; do [ -d "$workspace" ] || continue; if [ -f "$workspace/.devcontainer/docker-compose.yml" ]; then (cd "$workspace" && "$runtime" compose -f .devcontainer/docker-compose.yml down -v --remove-orphans >/dev/null 2>&1) || true; fi; done; for id in $("$runtime" ps -aq --filter label=com.docker.compose.project.working_dir 2>/dev/null || true); do working_dir=$("$runtime" inspect --format "{{ index .Config.Labels \"com.docker.compose.project.working_dir\" }}" "$id" 2>/dev/null || true); case "$working_dir" in /workspaces/*) "$runtime" rm -f "$id" >/dev/null 2>&1 || true ;; esac; done; find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null || sudo find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {} +' \
    || return 1
  ok "SSH companion E2E state pruned (images and BuildKit cache preserved)"
}

cmd_test_e2e() {
  local status=0 test_threads="${AIBOX_E2E_TEST_THREADS:-4}"
  [[ "${test_threads}" =~ ^[1-9][0-9]*$ ]] \
    || die "AIBOX_E2E_TEST_THREADS must be a positive integer"
  ensure_e2e_companion
  prune_e2e_companion_storage || die "Failed to prune SSH companion nested runtime state"
  info "Running Tier 2 SSH companion E2E shards..."
  AIBOX_E2E_TEST_THREADS="${test_threads}" "${SCRIPT_DIR}/run-e2e-shards.sh" all \
    || status=$?
  prune_e2e_companion_storage || warn "Post-suite SSH companion prune failed"
  [[ "${status}" -eq 0 ]] || die "Tier 2 SSH companion E2E tests failed"
  ok "Tier 2 SSH companion E2E tests passed"
}

cmd_test_e2e_shard() {
  local shard="$1" status=0 test_threads="${AIBOX_E2E_TEST_THREADS:-4}"
  [[ "${test_threads}" =~ ^[1-9][0-9]*$ ]] \
    || die "AIBOX_E2E_TEST_THREADS must be a positive integer"
  ensure_e2e_companion
  prune_e2e_companion_storage || die "Failed to prune SSH companion nested runtime state"
  info "Running Tier 2 SSH companion E2E shard: ${shard}..."
  AIBOX_E2E_TEST_THREADS="${test_threads}" "${SCRIPT_DIR}/run-e2e-shards.sh" "${shard}" \
    || status=$?
  prune_e2e_companion_storage || warn "Post-shard SSH companion prune failed"
  [[ "${status}" -eq 0 ]] || return "${status}"
  ok "Tier 2 SSH companion E2E shard passed: ${shard}"
}

cmd_test_e2e_visual_status() {
  ensure_e2e_companion
  info "Running opt-in visual E2E: generated tmux layouts, themes, and status line..."
  (cd "${CLI_DIR}" && cargo test --features e2e --test e2e \
    visual_generated_layouts_render_across_all_themes -- --ignored --nocapture --test-threads=1) \
    || die "Visual E2E status/theme matrix failed"
  ok "Visual E2E status/theme matrix passed"
}

cmd_test_e2e_visual_tabs() {
  ensure_e2e_companion
  info "Running opt-in visual E2E: generated tmux windows, tools, and harnesses..."
  (cd "${CLI_DIR}" && cargo test --features e2e --test e2e \
    visual_generated_tools_and_harness_windows_render_when_enabled -- --ignored --nocapture --test-threads=1) \
    || die "Visual E2E generated window traversal failed"
  ok "Visual E2E generated window traversal passed"
}

cmd_test_e2e_visual_yazi() {
  ensure_e2e_companion
  info "Running opt-in visual E2E: Yazi previews, git symbols, and plugins..."
  (cd "${CLI_DIR}" && cargo test --features e2e --test e2e \
    visual_yazi_previews_git_symbols_and_optional_plugins_render -- --ignored --nocapture --test-threads=1) \
    || die "Visual E2E Yazi preview matrix failed"
  ok "Visual E2E Yazi preview matrix passed"
}

cmd_test_e2e_visual() {
  info "Running all opt-in visual E2E tiers..."
  info "Visual E2E tier 1/3: generated layouts, themes, and tmux status line"
  cmd_test_e2e_visual_status
  info "Visual E2E tier 2/3: generated windows, tools, and harnesses"
  cmd_test_e2e_visual_tabs
  info "Visual E2E tier 3/3: Yazi previews, git symbols, and plugins"
  cmd_test_e2e_visual_yazi
  ok "Visual E2E matrix passed"
}

# ── Tier 3 (vt100 rendered-color) — closes the gap that allowed a previous
# tmux-bg-not-themed regression to ship undetected. Captures actual painted
# terminal cells via `tmux capture-pane -p -e` / `starship prompt` stdout and
# replays them through vt100 to assert per-cell bg/fg against the theme
# palette. Starship tier is local (no companion). tmux+yazi tiers need the
# companion and are #[ignore]-gated.
cmd_test_e2e_render_starship() {
  info "Running Tier 3 rendered-color tests: Starship prompt (local, no companion)..."
  (cd "${CLI_DIR}" && cargo test --features e2e-render --test e2e \
    visual_rendered_starship -- --nocapture) \
    || die "Tier 3 Starship rendered-color tests failed"
  ok "Tier 3 Starship rendered-color tests passed"
}

cmd_test_e2e_render_tmux() {
  # Removed in v0.26.2: the tests this dispatched (visual_rendered_tmux)
  # used `tmux capture-pane` to assert status-bar colors, but capture-pane
  # only sees pane contents — never the status bar. The asciinema-based
  # visual_themes_produce_tmux_signature_colors +
  # visual_tmux_status_and_panes_render_without_legacy_artifacts in
  # visual.rs cover the snapshot assertions correctly.
  warn "test-e2e-render-tmux: removed in v0.26.2 (capture-pane could not see status bar); use 'test-e2e-visual' (asciinema) instead"
}

cmd_test_e2e_render_layout_switch() {
  # Removed in v0.26.2 alongside cmd_test_e2e_render_tmux. The live
  # layout-switch coverage needs an asciinema-based rewrite — tracked
  # for v0.26.3+.
  warn "test-e2e-render-layout-switch: removed in v0.26.2 pending asciinema-based rewrite"
}

cmd_test_e2e_render_theme_switch() {
  # Removed in v0.26.2 alongside cmd_test_e2e_render_tmux. The live
  # theme-switch coverage needs an asciinema-based rewrite — tracked
  # for v0.26.3+.
  warn "test-e2e-render-theme-switch: removed in v0.26.2 pending asciinema-based rewrite"
}

cmd_test_e2e_render_yazi() {
  # Removed in v0.26.2: visual_rendered_yazi asserted yazi theme cells
  # captured via tmux capture-pane, which was already covered (correctly)
  # by visual_yazi_renders_in_tmux_pane in visual.rs (asciinema-based)
  # and visual_yazi_previews_git_symbols_and_optional_plugins_render in
  # visual_matrix.rs. Both pass — see /workspace/cli/tests/e2e/visual.rs.
  warn "test-e2e-render-yazi: removed in v0.26.2 (redundant with visual.rs/visual_matrix.rs); use 'test-e2e-visual' instead"
}

cmd_test_e2e_render() {
  info "Running Tier 3 rendered-color tests..."
  # Only the local Starship suite remains as a Tier 3 test in v0.26.2.
  # The tmux + yazi cell assertions and the live layout/theme switch
  # tests were structurally broken (capture-pane never sees the tmux
  # status bar) and have been folded into asciinema-based coverage in
  # visual.rs + visual_matrix.rs. A future asciinema-driven rewrite of
  # the live switch tests is tracked separately.
  cmd_test_e2e_render_starship
  ok "Tier 3 rendered-color tests passed"
}

cmd_test_e2e_doc_captures() {
  ensure_e2e_companion
  local artifact_dir="${AIBOX_E2E_VISUAL_ARTIFACT_DIR:-${PROJECT_ROOT}/docs-site/static/img/e2e}"
  mkdir -p "${artifact_dir}"
  info "Running visual E2E with docs-ready artifacts at ${artifact_dir}..."
  (cd "${CLI_DIR}" && AIBOX_E2E_VISUAL_ARTIFACT_DIR="${artifact_dir}" \
    cargo test --features e2e --test e2e visual_matrix -- --ignored --nocapture --test-threads=1) \
    || die "Visual E2E docs capture run failed"
  ok "Visual E2E docs artifacts written to ${artifact_dir}"
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
  local flavor="$1"
  local image_dir="${PROJECT_ROOT}/images/${flavor}"
  [[ -d "${image_dir}" ]] || die "Image source directory not found: ${image_dir}"
  while IFS= read -r file; do
    local rel="${file#${PROJECT_ROOT}/}"
    printf '%s  %s\n' "$(sha256_file "${file}")" "${rel}"
  done < <(find "${image_dir}" -type f | LC_ALL=C sort) | sha256_stdin
}

image_foundation_source_sha() {
  local flavor="$1"
  local image_dir="${PROJECT_ROOT}/images/${flavor}"
  local dockerfile="${image_dir}/Dockerfile"
  [[ -f "${dockerfile}" ]] || die "Image Dockerfile not found: ${dockerfile}"

  {
    awk '
      /^FROM foundation AS runtime$/ { exit }
      { print }
    ' "${dockerfile}"
    local rel file
    for file in \
      "${image_dir}/config/bin/aibox_status_core.rs" \
      "${image_dir}/config/bin/aibox-status.rs" \
      "${image_dir}/config/bin/aibox-diagnostics.rs"
    do
      [[ -f "${file}" ]] || continue
      rel="${file#${PROJECT_ROOT}/}"
      printf '%s  %s\n' "$(sha256_file "${file}")" "${rel}"
    done
  } | sha256_stdin
}

image_foundation_tag() {
  local flavor="$1" version="$2"
  printf '%s:%s-foundation-v%s' "${IMAGE_REGISTRY}" "${flavor}" "${version}"
}

image_runtime_tag() {
  local flavor="$1" version="$2"
  printf '%s:%s-runtime-v%s' "${IMAGE_REGISTRY}" "${flavor}" "${version}"
}

image_runtime_latest_tag() {
  local flavor="$1"
  printf '%s:%s-runtime-latest' "${IMAGE_REGISTRY}" "${flavor}"
}

legacy_image_tag() {
  local flavor="$1" version="$2"
  printf '%s:%s-v%s' "${IMAGE_REGISTRY}" "${flavor}" "${version}"
}

recent_patch_versions() {
  local version="$1" count="${2:-3}"
  local major minor patch
  IFS=. read -r major minor patch <<<"${version}"
  local emitted=0
  while (( patch > 0 && emitted < count )); do
    patch=$((patch - 1))
    printf '%s.%s.%s\n' "${major}" "${minor}" "${patch}"
    emitted=$((emitted + 1))
  done
}

use_runtime_image_tags() {
  local version="$1"
  local major minor patch
  IFS=. read -r major minor patch <<<"${version}"
  [[ "${major}" =~ ^[0-9]+$ && "${minor}" =~ ^[0-9]+$ ]] || return 1
  (( major > 0 || minor >= 27 ))
}

image_manifest_complete() {
  local ref="$1"

  if ! ${RUNTIME_BIN} buildx version >/dev/null 2>&1; then
    return 1
  fi
  if ! command -v jq >/dev/null 2>&1; then
    warn "jq is unavailable; cannot verify ${ref} manifest children before reuse"
    return 1
  fi

  local raw
  if ! raw="$(${RUNTIME_BIN} buildx imagetools inspect --raw "${ref}" 2>/dev/null)"; then
    return 1
  fi

  local digest
  while IFS= read -r digest; do
    [[ -z "${digest}" ]] && continue
    if ! ${RUNTIME_BIN} buildx imagetools inspect "${IMAGE_REGISTRY}@${digest}" >/dev/null 2>&1; then
      return 1
    fi
  done < <(
    printf '%s' "${raw}" \
      | jq -r '
          if (.mediaType == "application/vnd.oci.image.index.v1+json"
              or .mediaType == "application/vnd.docker.distribution.manifest.list.v2+json")
          then .manifests[]?.digest // empty
          else empty
          end
        ' 2>/dev/null
  )

  return 0
}

image_label_value() {
  local ref="$1" label="$2"

  if ! ${RUNTIME_BIN} buildx version >/dev/null 2>&1; then
    return 1
  fi
  if ! command -v jq >/dev/null 2>&1; then
    return 1
  fi

  local raw
  raw="$(${RUNTIME_BIN} buildx imagetools inspect --format '{{json .Image.Config.Labels}}' "${ref}" 2>/dev/null || true)"
  [[ -n "${raw}" && "${raw}" != "null" ]] || return 1
  printf '%s' "${raw}" | jq -er --arg label "${label}" '.[$label] // empty' 2>/dev/null
}

find_reusable_image_by_label() {
  local flavor="$1" version="$2" kind="$3" label="$4" expected="$5"
  local prior candidate value
  while IFS= read -r prior; do
    [[ -z "${prior}" ]] && continue
    case "${kind}" in
      foundation) candidate="$(image_foundation_tag "${flavor}" "${prior}")" ;;
      runtime) candidate="$(image_runtime_tag "${flavor}" "${prior}")" ;;
      legacy) candidate="$(legacy_image_tag "${flavor}" "${prior}")" ;;
      *) die "Unknown reusable image kind: ${kind}" ;;
    esac

    image_manifest_complete "${candidate}" || continue
    value="$(image_label_value "${candidate}" "${label}" || true)"
    if [[ "${value}" == "${expected}" ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done < <(recent_patch_versions "${version}" 3)

  return 1
}

ensure_ghcr_login() {
  if ! ${RUNTIME_BIN} login ghcr.io --get-login &>/dev/null 2>&1; then
    if command -v gh &>/dev/null && gh auth status &>/dev/null; then
      info "Logging into ghcr.io via gh auth..."
      gh auth token | ${RUNTIME_BIN} login ghcr.io -u "$(gh api user --jq .login)" --password-stdin \
        || die "Failed to log in to ghcr.io via gh. Ensure your gh token has write:packages scope."
      ok "Logged in to ghcr.io"
    else
      echo ""
      info "Not logged in to ghcr.io. Either:"
      echo ""
      echo "  1. Install and authenticate gh CLI: gh auth login"
      echo "  2. Or log in manually:"
      echo "     echo \$GITHUB_TOKEN | ${RUNTIME_BIN} login ghcr.io -u <username> --password-stdin"
      echo ""
      echo "  Your token needs the write:packages scope."
      echo ""
      die "GHCR authentication required."
    fi
  fi
}

require_docker_buildx_for_images() {
  if [[ "${RUNTIME_BIN}" != "docker" ]]; then
    return 0
  fi
  if docker buildx version >/dev/null 2>&1; then
    return 0
  fi

  die "Docker Buildx is required to build/publish aibox images because the Dockerfile uses BuildKit-only COPY --chmod. Install or enable the Docker Buildx component on the macOS host, then rerun: ./scripts/maintain.sh release-host <version>"
}

cmd_build_images() {
  _require_runtime
  require_docker_buildx_for_images
  local no_cache=""
  [[ "${1:-}" == "--no-cache" ]] && no_cache="--no-cache"
  local release_version="${2:-}"
  local build_env=()
  if [[ "${RUNTIME_BIN}" == "docker" ]]; then
    build_env=(env DOCKER_BUILDKIT="${DOCKER_BUILDKIT:-1}")
  fi

  local flavors=("base-debian")

  for flavor in "${flavors[@]}"; do
    info "Building ${flavor} foundation/runtime images..."
    local runtime_latest foundation_local runtime_local
    runtime_latest="$(image_runtime_latest_tag "${flavor}")"
    foundation_local="${IMAGE_REGISTRY}:${flavor}-foundation-local"
    runtime_local="${IMAGE_REGISTRY}:${flavor}-runtime-local"
    local source_sha foundation_sha runtime_sha
    source_sha="$(image_source_sha "${flavor}")"
    foundation_sha="$(image_foundation_source_sha "${flavor}")"
    runtime_sha="${source_sha}"
    local build_version="${release_version:-dev}"
    if [[ -n "${no_cache}" ]]; then
      "${build_env[@]}" ${RUNTIME_BIN} build --no-cache \
        --target foundation \
        --build-arg BUILDKIT_INLINE_CACHE=1 \
        --build-arg "AIBOX_IMAGE_SOURCE_SHA=${source_sha}" \
        --build-arg "AIBOX_FOUNDATION_SOURCE_SHA=${foundation_sha}" \
        --build-arg "AIBOX_RUNTIME_SOURCE_SHA=${runtime_sha}" \
        --build-arg "AIBOX_IMAGE_BUILD_VERSION=${build_version}" \
        -t "${foundation_local}" \
        -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
        "${PROJECT_ROOT}/images/${flavor}/"
      "${build_env[@]}" ${RUNTIME_BIN} build --no-cache \
        --target runtime \
        --build-arg BUILDKIT_INLINE_CACHE=1 \
        --build-arg "AIBOX_IMAGE_SOURCE_SHA=${source_sha}" \
        --build-arg "AIBOX_FOUNDATION_SOURCE_SHA=${foundation_sha}" \
        --build-arg "AIBOX_RUNTIME_SOURCE_SHA=${runtime_sha}" \
        --build-arg "AIBOX_IMAGE_BUILD_VERSION=${build_version}" \
        -t "${runtime_local}" \
        -t "${runtime_latest}" \
        -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
        "${PROJECT_ROOT}/images/${flavor}/"
    else
      ${RUNTIME_BIN} pull "${runtime_latest}" >/dev/null 2>&1 \
        || warn "Could not pull ${runtime_latest} as a remote build cache seed"
      if ${RUNTIME_BIN} buildx version >/dev/null 2>&1; then
        local foundation_cache_from_args=()
        local runtime_cache_from_args=()
        if image_manifest_complete "${runtime_latest}"; then
          foundation_cache_from_args+=(--cache-from "type=registry,ref=${runtime_latest}")
          runtime_cache_from_args+=(--cache-from "type=registry,ref=${runtime_latest}")
        else
          warn "Skipping unusable remote image cache ${runtime_latest}"
        fi
        ${RUNTIME_BIN} buildx build --load \
          --target foundation \
          --provenance=false \
          --sbom=false \
          --build-arg BUILDKIT_INLINE_CACHE=1 \
          --build-arg "AIBOX_IMAGE_SOURCE_SHA=${source_sha}" \
          --build-arg "AIBOX_FOUNDATION_SOURCE_SHA=${foundation_sha}" \
          --build-arg "AIBOX_RUNTIME_SOURCE_SHA=${runtime_sha}" \
          --build-arg "AIBOX_IMAGE_BUILD_VERSION=${build_version}" \
          ${foundation_cache_from_args[@]+"${foundation_cache_from_args[@]}"} \
          -t "${foundation_local}" \
          -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
          "${PROJECT_ROOT}/images/${flavor}/"
        ${RUNTIME_BIN} buildx build --load \
          --target runtime \
          --provenance=false \
          --sbom=false \
          --build-arg BUILDKIT_INLINE_CACHE=1 \
          --build-arg "AIBOX_IMAGE_SOURCE_SHA=${source_sha}" \
          --build-arg "AIBOX_FOUNDATION_SOURCE_SHA=${foundation_sha}" \
          --build-arg "AIBOX_RUNTIME_SOURCE_SHA=${runtime_sha}" \
          --build-arg "AIBOX_IMAGE_BUILD_VERSION=${build_version}" \
          ${runtime_cache_from_args[@]+"${runtime_cache_from_args[@]}"} \
          -t "${runtime_local}" \
          -t "${runtime_latest}" \
          -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
          "${PROJECT_ROOT}/images/${flavor}/"
      else
        "${build_env[@]}" ${RUNTIME_BIN} build \
          --target runtime \
          --build-arg BUILDKIT_INLINE_CACHE=1 \
          --build-arg "AIBOX_IMAGE_SOURCE_SHA=${source_sha}" \
          --build-arg "AIBOX_FOUNDATION_SOURCE_SHA=${foundation_sha}" \
          --build-arg "AIBOX_RUNTIME_SOURCE_SHA=${runtime_sha}" \
          --build-arg "AIBOX_IMAGE_BUILD_VERSION=${build_version}" \
          --cache-from "${runtime_latest}" \
          -t "${runtime_local}" \
          -t "${runtime_latest}" \
          -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
          "${PROJECT_ROOT}/images/${flavor}/"
      fi
    fi
    ok "Built ${foundation_local} and ${runtime_local}"
  done

  ok "All images built"
}

cmd_push_images() {
  _require_runtime
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh push-images <version>  (e.g. 0.2.0)"

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
    die "Version must be semver: X.Y.Z or X.Y.Z-prerelease (got: ${version})"
  fi

  ensure_ghcr_login

  local flavors=("base-debian")

  # Verify local runtime/foundation images exist and create versioned tags.
  for flavor in "${flavors[@]}"; do
    local foundation_local runtime_local foundation_versioned runtime_versioned runtime_latest legacy_versioned legacy_latest
    foundation_local="${IMAGE_REGISTRY}:${flavor}-foundation-local"
    runtime_local="${IMAGE_REGISTRY}:${flavor}-runtime-local"
    foundation_versioned="$(image_foundation_tag "${flavor}" "${version}")"
    runtime_versioned="$(image_runtime_tag "${flavor}" "${version}")"
    runtime_latest="$(image_runtime_latest_tag "${flavor}")"
    legacy_versioned="$(legacy_image_tag "${flavor}" "${version}")"
    legacy_latest="${IMAGE_REGISTRY}:${flavor}-latest"
    if ! ${RUNTIME_BIN} image exists "${foundation_local}" 2>/dev/null && \
       ! ${RUNTIME_BIN} inspect "${foundation_local}" &>/dev/null; then
      die "Image ${foundation_local} not found locally. Run 'build-images' first."
    fi
    if ! ${RUNTIME_BIN} image exists "${runtime_local}" 2>/dev/null && \
       ! ${RUNTIME_BIN} inspect "${runtime_local}" &>/dev/null; then
      die "Image ${runtime_local} not found locally. Run 'build-images' first."
    fi
    if use_runtime_image_tags "${version}"; then
      ${RUNTIME_BIN} tag "${foundation_local}" "${foundation_versioned}"
      ${RUNTIME_BIN} tag "${runtime_local}" "${runtime_versioned}"
      ${RUNTIME_BIN} tag "${runtime_local}" "${runtime_latest}"
    else
      ${RUNTIME_BIN} tag "${runtime_local}" "${legacy_versioned}"
      ${RUNTIME_BIN} tag "${runtime_local}" "${legacy_latest}"
    fi
  done

  ok "All images found and tagged for v${version}"

  # Push versioned foundation and versioned/latest runtime tags. Do not publish
  # source-hash marker tags; source hashes live in image labels only.
  for flavor in "${flavors[@]}"; do
    local foundation_versioned runtime_versioned runtime_latest legacy_versioned legacy_latest
    foundation_versioned="$(image_foundation_tag "${flavor}" "${version}")"
    runtime_versioned="$(image_runtime_tag "${flavor}" "${version}")"
    runtime_latest="$(image_runtime_latest_tag "${flavor}")"
    legacy_versioned="$(legacy_image_tag "${flavor}" "${version}")"
    legacy_latest="${IMAGE_REGISTRY}:${flavor}-latest"

    info "Pushing ${flavor}..."
    if use_runtime_image_tags "${version}"; then
      ${RUNTIME_BIN} push "${foundation_versioned}" || die "Failed to push ${foundation_versioned}"
      ${RUNTIME_BIN} push "${runtime_versioned}" || die "Failed to push ${runtime_versioned}"
      ${RUNTIME_BIN} push "${runtime_latest}" || die "Failed to push ${runtime_latest}"
      ok "Pushed ${flavor}-foundation-v${version} + ${flavor}-runtime-v${version} + ${flavor}-runtime-latest"
    else
      ${RUNTIME_BIN} push "${legacy_versioned}" || die "Failed to push ${legacy_versioned}"
      ${RUNTIME_BIN} push "${legacy_latest}" || die "Failed to push ${legacy_latest}"
      ok "Pushed ${flavor}-v${version} + ${flavor}-latest"
    fi
  done

  echo ""
  ok "All ${#flavors[@]} image(s) pushed to ${IMAGE_REGISTRY}"
  info "Verify at: https://github.com/orgs/projectious-work/packages"
}

cmd_publish_images_for_release() {
  _require_runtime
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh publish-images-for-release <version>"

  ensure_ghcr_login
  require_docker_buildx_for_images

  local flavors=("base-debian")
  local all_retagged=true
  for flavor in "${flavors[@]}"; do
    local foundation_versioned runtime_versioned runtime_latest legacy_versioned legacy_latest
    local current_sha foundation_sha runtime_sha reusable_foundation reusable_runtime reusable_legacy
    foundation_versioned="$(image_foundation_tag "${flavor}" "${version}")"
    runtime_versioned="$(image_runtime_tag "${flavor}" "${version}")"
    runtime_latest="$(image_runtime_latest_tag "${flavor}")"
    legacy_versioned="$(legacy_image_tag "${flavor}" "${version}")"
    legacy_latest="${IMAGE_REGISTRY}:${flavor}-latest"
    current_sha="$(image_source_sha "${flavor}")"
    foundation_sha="$(image_foundation_source_sha "${flavor}")"
    runtime_sha="${current_sha}"

    if ! use_runtime_image_tags "${version}"; then
      reusable_legacy="$(find_reusable_image_by_label "${flavor}" "${version}" legacy "org.projectious-work.aibox.image-source-sha" "${current_sha}" || true)"
      if [[ -n "${reusable_legacy}" ]]; then
        info "${flavor} source unchanged; retagging recent legacy manifest"
        ${RUNTIME_BIN} buildx imagetools create \
          -t "${legacy_versioned}" \
          -t "${legacy_latest}" \
          "${reusable_legacy}" \
          || die "Failed to retag ${reusable_legacy} as ${legacy_versioned} and ${legacy_latest}"
        ok "Retagged ${reusable_legacy} without rebuilding layers"
        continue
      fi

      all_retagged=false
      info "${flavor} has no recent reusable legacy manifest for current source hash; rebuilding"
      continue
    fi

    reusable_foundation="$(find_reusable_image_by_label "${flavor}" "${version}" foundation "org.projectious-work.aibox.image-foundation-source-sha" "${foundation_sha}" || true)"
    reusable_runtime="$(find_reusable_image_by_label "${flavor}" "${version}" runtime "org.projectious-work.aibox.image-runtime-source-sha" "${runtime_sha}" || true)"

    if [[ -n "${reusable_foundation}" && -n "${reusable_runtime}" ]]; then
      info "${flavor} foundation/runtime sources unchanged; retagging recent manifests"
      ${RUNTIME_BIN} buildx imagetools create \
        -t "${foundation_versioned}" \
        "${reusable_foundation}" \
        || die "Failed to retag ${reusable_foundation} as ${foundation_versioned}"
      ${RUNTIME_BIN} buildx imagetools create \
        -t "${runtime_versioned}" \
        -t "${runtime_latest}" \
        "${reusable_runtime}" \
        || die "Failed to retag ${reusable_runtime} as ${runtime_versioned} and ${runtime_latest}"
      ok "Retagged ${reusable_foundation} and ${reusable_runtime} without rebuilding layers"
    else
      all_retagged=false
      if ! ${RUNTIME_BIN} buildx version >/dev/null 2>&1; then
        warn "buildx is unavailable; rebuilding ${flavor} instead of retagging by source hash"
      elif [[ -z "${reusable_foundation}" && -z "${reusable_runtime}" ]]; then
        info "${flavor} has no recent reusable foundation/runtime manifests for current source hashes; rebuilding"
      elif [[ -z "${reusable_foundation}" ]]; then
        info "${flavor} runtime is reusable but foundation is not; rebuilding to keep release tags consistent"
      else
        info "${flavor} foundation is reusable but runtime is not; rebuilding runtime/foundation pair"
      fi
    fi
  done

  if [[ "${all_retagged}" == "true" ]]; then
    ok "All release images reused existing GHCR layers"
    verify_release_images_in_ghcr "${version}" "${flavors[@]}"
    return 0
  fi

  cmd_build_images "" "${version}"
  cmd_push_images "${version}"
  verify_release_images_in_ghcr "${version}" "${flavors[@]}"
}

# Post-publish guard: confirm every release-image tag is actually live in GHCR
# before declaring success. This validates the public foundation/runtime tags
# and rejects tags whose manifest index points at pruned child manifests.
verify_release_images_in_ghcr() {
  local version="${1:-}"
  shift || true
  local flavors=("$@")
  [[ ${#flavors[@]} -eq 0 ]] && flavors=("base-debian")

  local probe="${RUNTIME_BIN}"
  if ! ${probe} buildx version >/dev/null 2>&1; then
    warn "buildx unavailable; skipping post-publish GHCR verification"
    warn "(install Docker Buildx on the macOS host so release-host can confirm the published tags)"
    return 0
  fi

  local missing=()
  for flavor in "${flavors[@]}"; do
    local foundation_versioned runtime_versioned runtime_latest legacy_versioned legacy_latest
    local foundation_digest runtime_digest latest_digest legacy_digest
    foundation_versioned="$(image_foundation_tag "${flavor}" "${version}")"
    runtime_versioned="$(image_runtime_tag "${flavor}" "${version}")"
    runtime_latest="$(image_runtime_latest_tag "${flavor}")"
    legacy_versioned="$(legacy_image_tag "${flavor}" "${version}")"
    legacy_latest="${IMAGE_REGISTRY}:${flavor}-latest"

    if ! use_runtime_image_tags "${version}"; then
      if image_manifest_complete "${legacy_versioned}"; then
        legacy_digest="$(${probe} buildx imagetools inspect --raw "${legacy_versioned}" 2>/dev/null | sha256sum 2>/dev/null | awk '{print $1}')"
      else
        legacy_digest=""
        missing+=("${legacy_versioned}")
      fi
      if image_manifest_complete "${legacy_latest}"; then
        latest_digest="$(${probe} buildx imagetools inspect --raw "${legacy_latest}" 2>/dev/null | sha256sum 2>/dev/null | awk '{print $1}')"
      else
        latest_digest=""
        missing+=("${legacy_latest}")
      fi
      if [[ -n "${legacy_digest}" && -n "${latest_digest}" \
         && "${legacy_digest}" != "${latest_digest}" ]]; then
        warn "${legacy_versioned} and ${legacy_latest} resolve to different manifests in GHCR — latest likely stale"
        missing+=("${legacy_latest} (digest mismatch with ${legacy_versioned})")
      fi
      if [[ -n "${legacy_digest}" ]]; then
        ok "Verified ${legacy_versioned} is live in GHCR"
      fi
      continue
    fi

    if image_manifest_complete "${foundation_versioned}"; then
      foundation_digest="$(${probe} buildx imagetools inspect --raw "${foundation_versioned}" 2>/dev/null | sha256sum 2>/dev/null | awk '{print $1}')"
    else
      foundation_digest=""
      missing+=("${foundation_versioned}")
    fi
    if image_manifest_complete "${runtime_versioned}"; then
      runtime_digest="$(${probe} buildx imagetools inspect --raw "${runtime_versioned}" 2>/dev/null | sha256sum 2>/dev/null | awk '{print $1}')"
    else
      runtime_digest=""
      missing+=("${runtime_versioned}")
    fi
    if image_manifest_complete "${runtime_latest}"; then
      latest_digest="$(${probe} buildx imagetools inspect --raw "${runtime_latest}" 2>/dev/null | sha256sum 2>/dev/null | awk '{print $1}')"
    else
      latest_digest=""
      missing+=("${runtime_latest}")
    fi
    if [[ -n "${runtime_digest}" && -n "${latest_digest}" \
       && "${runtime_digest}" != "${latest_digest}" ]]; then
      warn "${runtime_versioned} and ${runtime_latest} resolve to different manifests in GHCR — latest likely stale"
      missing+=("${runtime_latest} (digest mismatch with ${runtime_versioned})")
    fi
    if [[ -n "${foundation_digest}" ]]; then
      ok "Verified ${foundation_versioned} is live in GHCR"
    fi
    if [[ -n "${runtime_digest}" ]]; then
      ok "Verified ${runtime_versioned} is live in GHCR"
    fi
  done

  if [[ ${#missing[@]} -gt 0 ]]; then
    echo ""
    for m in "${missing[@]}"; do
      warn "Missing or stale in GHCR after publish: ${m}"
    done
    die "Release image publish reported success but GHCR is missing one or more tags. Investigate buildx / ghcr.io auth before declaring the release complete."
  fi
}

cmd_release_runtime_smoke() {
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh release-runtime-smoke <version>  (e.g. 0.10.2)"

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
    die "Version must be semver: X.Y.Z or X.Y.Z-prerelease (got: ${version})"
  fi

  "${SCRIPT_DIR}/release-runtime-smoke.sh" "${version}" \
    || die "Release runtime smoke failed. See dist/release-smoke/v${version}/ for logs."
}

ghcr_package_versions_json() {
  local versions_json
  if ! versions_json="$(gh api /orgs/projectious-work/packages/container/aibox/versions --paginate 2>&1)"; then
    if [[ "${versions_json}" == *"read:packages"* || "${versions_json}" == *"HTTP 403"* ]]; then
      die "GHCR package cleanup requires a GitHub token with read:packages. Add delete:packages too when running with --execute."
    fi
    die "Failed to list GHCR package versions: ${versions_json}"
  fi
  printf '%s' "${versions_json}"
}

ghcr_source_tag_sets() {
  local versions_json="$1"
  local source_only_var="$2"
  local mixed_var="$3"
  local source_only_json mixed_json

  source_only_json="$(
    printf '%s' "${versions_json}" | jq -sc '
      add
      |
      [
        .[]
        | {id, name, tags: (.metadata.container.tags // [])}
        | select([.tags[] | startswith("base-") and contains("-source-")] | any)
        | select((.tags | length) > 0)
        | select(all(.tags[]; contains("-source-")))
      ]
    '
  )"
  mixed_json="$(
    printf '%s' "${versions_json}" | jq -sc '
      add
      |
      [
        .[]
        | {
            id,
            name,
            tags: (.metadata.container.tags // []),
            source_tags: [(.metadata.container.tags // [])[] | select(startswith("base-") and contains("-source-"))],
            keep_tags: [(.metadata.container.tags // [])[] | select((startswith("base-") and contains("-source-")) | not)]
          }
        | select((.source_tags | length) > 0 and (.keep_tags | length) > 0)
      ]
    '
  )"

  printf -v "${source_only_var}" '%s' "${source_only_json}"
  printf -v "${mixed_var}" '%s' "${mixed_json}"
}

ghcr_repair_mixed_source_tags() {
  local mixed_json="$1"
  local execute="$2"
  local mixed_count
  mixed_count="$(printf '%s' "${mixed_json}" | jq 'length')"
  (( mixed_count > 0 )) || return 0

  if [[ "${execute}" == "true" ]]; then
    require_docker_buildx_for_images
  fi

  warn "Repairing mixed source-hash package versions works by moving non-source tags to a fresh manifest copy."
  warn "The old package version should then become source-only and deletable by this command."

  local row id digest annotation source_ref
  local -a keep_tags tag_args
  while IFS= read -r row; do
    [[ -z "${row}" ]] && continue
    id="$(printf '%s' "${row}" | jq -r '.id')"
    digest="$(printf '%s' "${row}" | jq -r '.name')"
    source_ref="${IMAGE_REGISTRY}@${digest}"
    mapfile -t keep_tags < <(printf '%s' "${row}" | jq -r '.keep_tags[]')
    (( ${#keep_tags[@]} > 0 )) || continue

    tag_args=()
    local keep_tag
    for keep_tag in "${keep_tags[@]}"; do
      tag_args+=(-t "${IMAGE_REGISTRY}:${keep_tag}")
    done
    annotation="index:org.projectious-work.aibox.source-tag-detached=$(date -u +%Y%m%dT%H%M%SZ)"

    if [[ "${execute}" != "true" ]]; then
      printf '  repair/mixed id=%s source=%s keep-tags=%s\n' \
        "${id}" "${source_ref}" "$(IFS=,; printf '%s' "${keep_tags[*]}")"
      continue
    fi

    info "Moving non-source tag(s) off mixed GHCR package version ${id}: $(IFS=,; printf '%s' "${keep_tags[*]}")"
    ${RUNTIME_BIN} buildx imagetools create \
      --prefer-index=true \
      --annotation "${annotation}" \
      "${tag_args[@]}" \
      "${source_ref}" \
      || die "Failed to move non-source tags off mixed GHCR package version ${id}"
    ok "Moved non-source tag(s) for mixed GHCR package version ${id}"
  done < <(printf '%s' "${mixed_json}" | jq -c '.[]')
}

cmd_ghcr_prune_source_tags() {
  local execute=false
  local repair_mixed=false
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --execute) execute=true ;;
      --repair-mixed) repair_mixed=true ;;
      --dry-run) ;;
      *) die "Usage: ./scripts/maintain.sh ghcr-prune-source-tags [--repair-mixed] [--execute]" ;;
    esac
    shift
  done

  if ! command -v gh >/dev/null 2>&1; then
    die "gh CLI is required for GHCR package cleanup"
  fi
  if ! command -v jq >/dev/null 2>&1; then
    die "jq is required for GHCR package cleanup"
  fi

  info "Scanning GHCR aibox package versions for public source-hash tags..."
  local versions_json
  versions_json="$(ghcr_package_versions_json)"
  local source_only_json mixed_json source_only_count mixed_count
  ghcr_source_tag_sets "${versions_json}" source_only_json mixed_json
  source_only_count="$(printf '%s' "${source_only_json}" | jq 'length')"
  mixed_count="$(printf '%s' "${mixed_json}" | jq 'length')"

  if (( mixed_count > 0 )); then
    warn "Found ${mixed_count} package version(s) where source-hash tags share a manifest with non-source tags."
    warn "GitHub's package-version delete API would delete every tag on those versions, so these are reported but not deleted directly."
    printf '%s' "${mixed_json}" | jq -r '.[] | "  keep/mixed id=\(.id) tags=\(.tags | join(","))"'
    if [[ "${repair_mixed}" == "true" ]]; then
      ghcr_repair_mixed_source_tags "${mixed_json}" "${execute}"
      if [[ "${execute}" == "true" ]]; then
        info "Re-scanning GHCR after mixed-version repair..."
        versions_json="$(ghcr_package_versions_json)"
        ghcr_source_tag_sets "${versions_json}" source_only_json mixed_json
        source_only_count="$(printf '%s' "${source_only_json}" | jq 'length')"
        mixed_count="$(printf '%s' "${mixed_json}" | jq 'length')"
        if (( mixed_count > 0 )); then
          warn "${mixed_count} mixed source-hash package version(s) remain after repair attempt."
          printf '%s' "${mixed_json}" | jq -r '.[] | "  keep/mixed id=\(.id) tags=\(.tags | join(","))"'
        fi
      else
        warn "Dry run only. Re-run with --repair-mixed --execute to move non-source tags off mixed versions."
      fi
    else
      warn "Re-run with --repair-mixed to plan moving non-source tags off mixed versions."
      warn "Use --repair-mixed --execute only after reviewing the plan."
    fi
  fi

  if (( source_only_count == 0 )); then
    ok "No source-hash-only GHCR package versions found."
    return 0
  fi

  printf '%s' "${source_only_json}" | jq -r '.[] | "  source-only id=\(.id) tags=\(.tags | join(","))"'
  if [[ "${execute}" != "true" ]]; then
    warn "Dry run only. Re-run with --execute to delete the ${source_only_count} source-hash-only package version(s)."
    return 0
  fi

  warn "Deleting ${source_only_count} source-hash-only GHCR package version(s)."
  local id delete_output
  while IFS= read -r id; do
    [[ -z "${id}" ]] && continue
    if ! delete_output="$(gh api -X DELETE "/orgs/projectious-work/packages/container/aibox/versions/${id}" 2>&1)"; then
      if [[ "${delete_output}" == *"delete:packages"* || "${delete_output}" == *"HTTP 403"* ]]; then
        die "Deleting GHCR package versions requires a GitHub token with delete:packages as well as read:packages."
      fi
      die "Failed to delete GHCR package version ${id}: ${delete_output}"
    fi
    ok "Deleted GHCR package version ${id}"
  done < <(printf '%s' "${source_only_json}" | jq -r '.[].id')
}

cmd_ghcr_prune_buildcache_tags() {
  local execute=false
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --execute) execute=true ;;
      --dry-run) ;;
      *) die "Usage: ./scripts/maintain.sh ghcr-prune-buildcache-tags [--execute]" ;;
    esac
    shift
  done

  if ! command -v gh >/dev/null 2>&1; then
    die "gh CLI is required for GHCR package cleanup"
  fi
  if ! command -v jq >/dev/null 2>&1; then
    die "jq is required for GHCR package cleanup"
  fi

  info "Scanning GHCR aibox package versions for BuildKit cache tags..."
  local versions_json cache_only_json mixed_json cache_only_count mixed_count
  versions_json="$(ghcr_package_versions_json)"
  cache_only_json="$(
    printf '%s' "${versions_json}" | jq -sc '
      add
      |
      [
        .[]
        | {id, name, tags: (.metadata.container.tags // [])}
        | select([.tags[] | endswith("-buildcache")] | any)
        | select((.tags | length) > 0)
        | select(all(.tags[]; endswith("-buildcache")))
      ]
    '
  )"
  mixed_json="$(
    printf '%s' "${versions_json}" | jq -sc '
      add
      |
      [
        .[]
        | {
            id,
            name,
            tags: (.metadata.container.tags // []),
            cache_tags: [(.metadata.container.tags // [])[] | select(endswith("-buildcache"))],
            keep_tags: [(.metadata.container.tags // [])[] | select((endswith("-buildcache")) | not)]
          }
        | select((.cache_tags | length) > 0 and (.keep_tags | length) > 0)
      ]
    '
  )"
  cache_only_count="$(printf '%s' "${cache_only_json}" | jq 'length')"
  mixed_count="$(printf '%s' "${mixed_json}" | jq 'length')"

  if (( mixed_count > 0 )); then
    warn "Found ${mixed_count} package version(s) where buildcache tags share a manifest with non-cache tags."
    warn "GitHub's package-version delete API would delete every tag on those versions, so these are reported but not deleted."
    printf '%s' "${mixed_json}" | jq -r '.[] | "  keep/mixed id=\(.id) tags=\(.tags | join(","))"'
  fi

  if (( cache_only_count == 0 )); then
    ok "No buildcache-only GHCR package versions found."
    return 0
  fi

  printf '%s' "${cache_only_json}" | jq -r '.[] | "  buildcache-only id=\(.id) tags=\(.tags | join(","))"'
  if [[ "${execute}" != "true" ]]; then
    warn "Dry run only. Re-run with --execute to delete the ${cache_only_count} buildcache-only package version(s)."
    return 0
  fi

  warn "Deleting ${cache_only_count} buildcache-only GHCR package version(s)."
  local id delete_output
  while IFS= read -r id; do
    [[ -z "${id}" ]] && continue
    if ! delete_output="$(gh api -X DELETE "/orgs/projectious-work/packages/container/aibox/versions/${id}" 2>&1)"; then
      if [[ "${delete_output}" == *"delete:packages"* || "${delete_output}" == *"HTTP 403"* ]]; then
        die "Deleting GHCR package versions requires a GitHub token with delete:packages as well as read:packages."
      fi
      die "Failed to delete GHCR package version ${id}: ${delete_output}"
    fi
    ok "Deleted GHCR package version ${id}"
  done < <(printf '%s' "${cache_only_json}" | jq -r '.[].id')
}

cmd_docs_serve() {
  command -v hugo &>/dev/null || die "Hugo extended not found. Install Hugo >= 0.157.0."
  command -v npm &>/dev/null  || die "npm not found. Install Node.js."
  if [[ ! -f "${PROJECT_ROOT}/docs-site/themes/docsy/theme.toml" ]]; then
    git -C "${PROJECT_ROOT}" submodule update --init --recursive docs-site/themes/docsy
  fi
  if [[ ! -d "${PROJECT_ROOT}/docs-site/node_modules" ]]; then
    npm --prefix "${PROJECT_ROOT}/docs-site" ci
  fi
  info "Serving docs with Hugo and Docsy at http://localhost:1316/aibox/ ..."
  hugo server --source "${PROJECT_ROOT}/docs-site" \
    --bind 0.0.0.0 --port 1316 --baseURL "http://localhost:1316/aibox/"
}

cmd_docs_deploy() {
  "${PROJECT_ROOT}/scripts/deploy-docs.sh" "$@"
}

# Retained temporarily for line-history comparison while the versioned
# deployment path rolls out. It is not called by the command dispatcher.
cmd_docs_deploy_legacy() {
  local dry_run=false
  # Bug (b): declare tmpdir early so the EXIT trap below never sees an unbound
  # variable under set -u if the function exits before reaching mktemp -d.
  local tmpdir=""
  [[ "${1:-}" == "--dry-run" ]] && dry_run=true

  command -v hugo &>/dev/null   || die "Hugo extended not found. Install Hugo >= 0.157.0."
  command -v npm &>/dev/null    || die "npm not found. Install Node.js."
  command -v git &>/dev/null    || die "git not found"
  git rev-parse --is-inside-work-tree &>/dev/null || die "Not inside a git repository"

  local remote_url current_branch commit_sha commit_msg repo_slug
  remote_url=$(git remote get-url origin 2>/dev/null) || die "No 'origin' remote"
  current_branch=$(git rev-parse --abbrev-ref HEAD)
  commit_sha=$(git rev-parse --short HEAD)
  commit_msg="docs: deploy from ${current_branch}@${commit_sha} ($(date -u +%Y-%m-%dT%H:%M:%SZ))"
  repo_slug=$(echo "${remote_url}" | sed -E 's|.*[:/]([^/]+/[^/]+)(\.git)?$|\1|' | sed 's/\.git$//')

  info "Remote: ${remote_url}"
  info "Source: ${current_branch}@${commit_sha}"

  cd "${PROJECT_ROOT}"
  info "Building docs with Hugo and Docsy..."
  "${PROJECT_ROOT}/scripts/build-docs.sh"
  ok "Site built in docs-site/public/"

  if [[ "${dry_run}" == "true" ]]; then
    warn "Dry run — site is in docs-site/public/"
    return 0
  fi

  tmpdir=$(mktemp -d)
  # Bug (b): use ${tmpdir:-} so trap is safe even if mktemp never ran (set -u).
  trap '[[ -n "${tmpdir:-}" ]] && rm -rf "${tmpdir}"' EXIT

  cp -r "${PROJECT_ROOT}/docs-site/public/." "${tmpdir}/"
  touch "${tmpdir}/.nojekyll"

  info "Pushing to gh-pages branch..."
  cd "${tmpdir}"
  git init -q
  git checkout -q -b gh-pages
  git add -A
  # Bug (a): fresh worktree has no user identity; devcontainer git has none by
  # default. Read from the project git config; fall back to a release-bot identity.
  local git_user git_email
  git_user=$(git -C "${PROJECT_ROOT}" config user.name  2>/dev/null || echo "aibox-release-bot")
  git_email=$(git -C "${PROJECT_ROOT}" config user.email 2>/dev/null || echo "release@aibox.local")
  git -c "user.name=${git_user}" -c "user.email=${git_email}" \
    commit -q -m "${commit_msg}"
  # The fresh tmpdir worktree has no credential helper inherited from the
  # project. When `remote_url` is HTTPS, plain git push prompts for a
  # username and aborts non-interactively. Inject the gh CLI's OAuth token as
  # GitHub's documented Basic x-access-token credential when (a) the URL is
  # HTTPS-on-github.com and (b) `gh auth token` succeeds. SSH and other hosts
  # pass through unchanged.
  local push_url="${remote_url}"
  local push_auth_extraheader=()
  if [[ "${remote_url}" == https://github.com/* ]] && command -v gh &>/dev/null; then
    local gh_token
    if gh_token=$(gh auth token 2>/dev/null) && [[ -n "${gh_token}" ]]; then
      local gh_basic
      gh_basic=$(printf 'x-access-token:%s' "${gh_token}" | base64 | tr -d '\n')
      push_auth_extraheader=(-c "http.https://github.com/.extraheader=AUTHORIZATION: basic ${gh_basic}")
    fi
  fi
  git "${push_auth_extraheader[@]}" push --force "${push_url}" gh-pages:gh-pages
  cd "${PROJECT_ROOT}"
  ok "Deployed to gh-pages branch"

  # Configure GitHub Pages if gh is available. Probe-first: only POST when
  # Pages doesn't exist yet. Avoids the spurious "Could not configure"
  # warning we used to print on every release because PUT 422s when Pages
  # is already configured by another Pages source or already pinned to
  # gh-pages with identical settings.
  if command -v gh &>/dev/null && [[ -n "${repo_slug}" ]]; then
    if gh api "repos/${repo_slug}/pages" --silent 2>/dev/null; then
      ok "GitHub Pages already configured (skipping)"
    elif gh api --method POST "repos/${repo_slug}/pages" \
           -f "source[branch]=gh-pages" -f "source[path]=/" \
           --silent 2>/dev/null; then
      ok "GitHub Pages configured"
    else
      warn "Pages auto-config skipped (already configured or unavailable — non-fatal)"
    fi
  fi

  echo ""
  ok "Documentation deployed."
  [[ -n "${repo_slug}" ]] && info "URL: https://${repo_slug/\//.github.io\/}/"
  trap - EXIT
  rm -rf "${tmpdir}"
}

# =============================================================================
# cmd_sync_processkit — check for a new processkit release and pull it in
#
# Queries GitHub for the latest processkit tag, compares it with the
# PROCESSKIT_DEFAULT_VERSION constant in cli/src/processkit_vocab.rs, and if
# a newer version exists:
#
#   1. Patches PROCESSKIT_DEFAULT_VERSION in processkit_vocab.rs
#   2. Fetches the new FORMAT.md from the processkit repo and displays the
#      diff against the previous version so the maintainer can spot vocabulary
#      changes (new categories, renamed filenames, new directory segments, etc.)
#   3. Re-runs the vocabulary unit tests (they enforce count/no-duplicates on
#      CATEGORY_ORDER and will catch obvious drift immediately)
#
# Called automatically by cmd_release. Can also be run standalone:
#   ./scripts/maintain.sh sync-processkit
#
# After this runs, review the FORMAT.md diff and update processkit_vocab.rs
# manually if any vocabulary changed (CATEGORY_ORDER, src:: segments, filename
# constants). Then commit before running `release`.
# =============================================================================
cmd_sync_processkit() {
  command -v gh &>/dev/null || die "gh CLI required for processkit version check"

  info "Checking for processkit updates..."

  # ── Resolve latest upstream tag ───────────────────────────────────────────
  local latest_tag
  latest_tag=$(gh api repos/projectious-work/processkit/releases/latest --jq '.tag_name' 2>/dev/null) \
    || { warn "Could not reach GitHub API — skipping processkit update check"; return 0; }

  if [[ -z "${latest_tag}" ]]; then
    warn "No processkit releases found — skipping update check"
    return 0
  fi

  # ── Read the currently pinned version from processkit_vocab.rs ────────────
  local vocab_file="${CLI_DIR}/src/processkit_vocab.rs"
  local current_tag
  current_tag=$(grep 'pub const PROCESSKIT_DEFAULT_VERSION' "${vocab_file}" \
    | grep -oP '"v[^"]+"' | tr -d '"')

  info "processkit: current=${current_tag}  latest=${latest_tag}"

  if [[ "${current_tag}" == "${latest_tag}" ]]; then
    ok "processkit is already up to date (${current_tag})"
    return 0
  fi

  warn "New processkit version available: ${current_tag} → ${latest_tag}"

  # ── Fetch FORMAT.md for both versions for a vocabulary diff ───────────────
  local fmt_path="src/.processkit/FORMAT.md"
  local tmp_old tmp_new
  tmp_old=$(mktemp)
  tmp_new=$(mktemp)
  trap 'rm -f "${tmp_old}" "${tmp_new}"' RETURN

  _fetch_processkit_file() {
    local ref="$1" dest="$2"
    gh api "repos/projectious-work/processkit/contents/${fmt_path}?ref=${ref}" \
      --jq '.content' 2>/dev/null \
      | base64 -d > "${dest}" 2>/dev/null \
      || { warn "Could not fetch FORMAT.md for ${ref}"; touch "${dest}"; }
  }

  info "Fetching FORMAT.md for ${current_tag} and ${latest_tag}..."
  _fetch_processkit_file "${current_tag}" "${tmp_old}"
  _fetch_processkit_file "${latest_tag}"  "${tmp_new}"

  local diff_output
  diff_output=$(diff --unified=3 "${tmp_old}" "${tmp_new}" || true)

  if [[ -z "${diff_output}" ]]; then
    info "FORMAT.md is unchanged between ${current_tag} and ${latest_tag}"
    info "(Vocabulary constants in processkit_vocab.rs need no update)"
  else
    echo ""
    echo "${bold}FORMAT.md diff (${current_tag} → ${latest_tag}):${reset}"
    echo "${diff_output}"
    echo ""
    warn "Review the diff above. If any of these changed, update processkit_vocab.rs manually:"
    echo "  · CATEGORY_ORDER        (new/removed/reordered categories)"
    echo "  · processkit_vocab::src (new/renamed source-tree directory segments)"
    echo "  · *_FILENAME constants  (SKILL_FILENAME, PROVENANCE_FILENAME, INDEX_FILENAME, …)"
    echo ""
    warn "Press Enter to continue after reviewing, or Ctrl-C to abort and update first."
    read -r
  fi

  # ── Patch PROCESSKIT_DEFAULT_VERSION in processkit_vocab.rs ───────────────
  info "Patching PROCESSKIT_DEFAULT_VERSION: ${current_tag} → ${latest_tag}"
  sed -i "s|pub const PROCESSKIT_DEFAULT_VERSION: &str = \"${current_tag}\";|pub const PROCESSKIT_DEFAULT_VERSION: \&str = \"${latest_tag}\";|" \
    "${vocab_file}"
  ok "Patched ${vocab_file}"

  # ── Re-run vocabulary tests to catch obvious drift ────────────────────────
  info "Running processkit vocabulary tests..."
  (cd "${CLI_DIR}" && cargo test processkit_vocab 2>&1) \
    || die "Vocabulary tests failed after update — fix processkit_vocab.rs before releasing"
  ok "Vocabulary tests pass for ${latest_tag}"

  # ── Remind maintainer to commit ────────────────────────────────────────────
  echo ""
  warn "processkit_vocab.rs patched but not yet committed."
  warn "Review the diff above, make any additional vocabulary changes, then commit:"
  echo ""
  echo "  git add cli/src/processkit_vocab.rs"
  echo "  git commit -m \"chore: bump processkit default version to ${latest_tag}\""
  echo ""
  echo "Then re-run: ./scripts/maintain.sh release <version>"
}

cmd_release_check_state() {
  case "${1:-}" in
    --require-network)
      AIBOX_RELEASE_REQUIRE_NETWORK=1 "${SCRIPT_DIR}/release-check-state.sh"
      ;;
    "")
      "${SCRIPT_DIR}/release-check-state.sh"
      ;;
    *)
      die "Usage: ./scripts/maintain.sh release-check-state [--require-network]"
      ;;
  esac
}

run_aibox_doctor_for_release() {
  if command -v cargo >/dev/null 2>&1; then
    (cd "${PROJECT_ROOT}" && cargo run --manifest-path "${CLI_DIR}/Cargo.toml" --quiet -- doctor)
    return $?
  fi

  local candidate
  for candidate in \
    "${CLI_DIR}/target/aarch64-unknown-linux-gnu/release/aibox" \
    "${CLI_DIR}/target/release/aibox" \
    "${CLI_DIR}/target/x86_64-unknown-linux-gnu/release/aibox"
  do
    if [[ -x "${candidate}" ]]; then
      "${candidate}" doctor
      return $?
    fi
  done

  printf 'cargo is unavailable and no runnable Linux aibox binary was found for release doctor.\n' >&2
  return 127
}

# =============================================================================
# cmd_release_doctors — run pk-doctor + aibox doctor as a Phase 0 gate
#
# Both doctors are invoked sequentially.  Output (stdout + stderr) is
# captured and written to dist/RELEASE-DOCTORS.md.  Gate semantics:
#
# release-doctors is an explicit aibox CLI development exception: it may run
# aibox doctor from the devcontainer as a host-context simulation. Normal
# dogfood/self-management inside the workspace container must use pk-doctor
# instead.
#
#   pk-doctor  exits 0 → no ERRORs; exits 1 → ERRORs found.
#   aibox doctor always exits 0, but prints a summary line:
#     "Diagnostics complete: N warning(s), M error(s)"
#   We parse that line; M > 0 is treated as ERROR.
#
# Gate outcome:
#   Both pass (0 ERRORs) → continue, exit 0.
#   Either has ERRORs    → write RELEASE-DOCTORS.md, halt with message.
#   WARNs only           → write RELEASE-DOCTORS.md, continue (non-blocking).
# =============================================================================
cmd_release_doctors() {
  mkdir -p "${DIST_DIR}"
  local report="${DIST_DIR}/RELEASE-DOCTORS.md"
  local blocked=0

  info "Running pk-doctor (processkit health check)..."
  local pk_out pk_exit
  set +e
  pk_out=$( \
    cd "${PROJECT_ROOT}" && \
    uv run --script \
      "${PROJECT_ROOT}/context/skills/processkit/pk-doctor/scripts/doctor.py" \
      --no-log \
      2>&1 \
  )
  pk_exit=$?
  set -e

  info "Running aibox doctor (runtime hygiene check)..."
  local aibox_out aibox_exit aibox_err_count
  set +e
  aibox_out=$(run_aibox_doctor_for_release 2>&1)
  aibox_exit=$?
  set -e

  # aibox doctor always exits 0; detect errors by parsing the summary line.
  # Format: "Diagnostics complete: N warning(s), M error(s)"
  aibox_err_count=$(echo "${aibox_out}" | \
    grep -oP '(\d+) error\(s\)' | grep -oP '^\d+' || echo "0")
  [[ -z "${aibox_err_count}" ]] && aibox_err_count=0

  # Determine gate outcome.
  local pk_status aibox_status
  if [[ "${pk_exit}" -ne 0 ]]; then
    pk_status="ERROR (exit ${pk_exit})"
    blocked=1
  else
    pk_status="OK"
  fi

  if [[ "${aibox_exit}" -ne 0 ]]; then
    aibox_status="ERROR (exit ${aibox_exit})"
    blocked=1
  elif [[ "${aibox_err_count}" -gt 0 ]]; then
    aibox_status="ERROR (${aibox_err_count} error(s))"
    blocked=1
  else
    aibox_status="OK"
  fi

  # Write the combined report regardless of outcome.
  {
    echo "# Release Doctors Report"
    echo ""
    echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo ""
    echo "| Doctor | Status |"
    echo "|--------|--------|"
    echo "| pk-doctor | ${pk_status} |"
    echo "| aibox doctor | ${aibox_status} |"
    echo ""
    echo "## pk-doctor output"
    echo ""
    echo "\`\`\`"
    echo "${pk_out}"
    echo "\`\`\`"
    echo ""
    echo "## aibox doctor output"
    echo ""
    echo "\`\`\`"
    echo "${aibox_out}"
    echo "\`\`\`"
  } > "${report}"

  if [[ "${blocked}" -eq 1 ]]; then
    echo ""
    if [[ "${pk_exit}" -ne 0 ]]; then
      warn "pk-doctor reported ERRORs (exit ${pk_exit})."
    fi
    if [[ "${aibox_err_count}" -gt 0 ]]; then
      warn "aibox doctor reported ${aibox_err_count} error(s)."
    elif [[ "${aibox_exit}" -ne 0 ]]; then
      warn "aibox doctor invocation failed (exit ${aibox_exit})."
    fi
    die "Release blocked: doctor checks failed. See ${report} for details."
  fi

  ok "Doctor checks passed. Report written to ${report}"
}

release_usage() {
  die "Usage: ./scripts/maintain.sh release <version> [--steps list] [--skip list] [--list-steps]"
}

release_list_steps() {
  cat <<'STEPS'
Release step aliases:
  all       state,doctors,sync,version,test,e2e,visual,audit,build-linux,version-smoke,push-main,notes,tag,github-release,docs,prompt
  phase0    state,doctors
  checks    sync,version,test,e2e,visual,audit
  build     build-linux,version-smoke
  publish   push-main,notes,tag,github-release

Concrete release steps:
  state           Generate dist/RELEASE-STATE.md with dependency and harness network lookups
  doctors         Run pk-doctor and aibox doctor into dist/RELEASE-DOCTORS.md
  sync            Check/sync processkit default version
  version         Bump cli/Cargo.toml and Cargo.lock if needed, then commit
  test            Run fmt, clippy, and unit/integration tests
  e2e             Run Tier 2 SSH companion E2E
  visual          Run the selected visual E2E tier from AIBOX_RELEASE_VISUAL_E2E
  audit           Run cargo audit
  build-linux     Build Linux release archives and checksums
  version-smoke   Verify the native release binary reports the requested version
  push-main       Promote the candidate to its protected release branch
  notes           Prepare or reuse dist/RELEASE-NOTES.md
  tag             Create and push the annotated release tag
  github-release  Create the GitHub release with Linux archives
  docs            Deploy documentation
  prompt          Write dist/RELEASE-PROMPT.md for host-side phase 2

Examples:
  ./scripts/maintain.sh release 0.25.15 --steps audit,build
  ./scripts/maintain.sh release 0.25.15 --skip e2e,visual
  ./scripts/maintain.sh release 0.25.15 --steps phase0
  ./scripts/maintain.sh release 0.25.15 --steps publish,prompt
STEPS
}

release_add_step() {
  local candidate="$1" existing
  for existing in "${release_steps[@]}"; do
    [[ "${existing}" == "${candidate}" ]] && return 0
  done
  release_steps+=("${candidate}")
}

release_expand_step_token() {
  local token="$1"
  case "${token}" in
    all)
      release_expand_step_token phase0
      release_expand_step_token checks
      release_expand_step_token build
      release_expand_step_token publish
      release_add_step docs
      release_add_step prompt
      ;;
    phase0)
      release_add_step state
      release_add_step doctors
      ;;
    checks)
      release_add_step sync
      release_add_step version
      release_add_step test
      release_add_step e2e
      release_add_step visual
      release_add_step audit
      ;;
    build)
      release_add_step build-linux
      release_add_step version-smoke
      ;;
    publish)
      release_add_step push-main
      release_add_step notes
      release_add_step tag
      release_add_step github-release
      ;;
    state|doctors|sync|version|test|e2e|visual|audit|build-linux|version-smoke|push-main|notes|tag|github-release|docs|prompt)
      release_add_step "${token}"
      ;;
    "")
      ;;
    *)
      die "Unknown release step '${token}'. Run './scripts/maintain.sh release --list-steps' for valid steps."
      ;;
  esac
}

release_parse_steps() {
  local spec="${1:-all}" token
  release_steps=()
  IFS=',' read -ra release_step_tokens <<< "${spec}"
  for token in "${release_step_tokens[@]}"; do
    token="${token//[[:space:]]/}"
    release_expand_step_token "${token}"
  done
  [[ "${#release_steps[@]}" -gt 0 ]] || release_usage
}

release_remove_step() {
  local skipped="$1" step kept=()
  for step in "${release_steps[@]}"; do
    [[ "${step}" == "${skipped}" ]] && continue
    kept+=("${step}")
  done
  release_steps=("${kept[@]}")
}

release_apply_skip_steps() {
  local spec="$1" token skip_steps=()
  [[ -n "${spec}" ]] || return 0

  local original_steps=("${release_steps[@]}")
  release_steps=()
  IFS=',' read -ra release_step_tokens <<< "${spec}"
  for token in "${release_step_tokens[@]}"; do
    token="${token//[[:space:]]/}"
    release_expand_step_token "${token}"
  done
  skip_steps=("${release_steps[@]}")
  release_steps=("${original_steps[@]}")

  for token in "${skip_steps[@]}"; do
    release_remove_step "${token}"
  done
  [[ "${#release_steps[@]}" -gt 0 ]] || die "No release steps remain after --skip."
}

release_step_requested() {
  local step="$1" existing
  for existing in "${release_steps[@]}"; do
    [[ "${existing}" == "${step}" ]] && return 0
  done
  return 1
}

release_steps_joined() {
  local IFS=','
  printf '%s' "${release_steps[*]}"
}

release_requires_clean_tree() {
  local step
  for step in "${release_steps[@]}"; do
    case "${step}" in
      sync|version|push-main|tag|github-release|docs)
        return 0
        ;;
    esac
  done
  return 1
}

release_collect_linux_archives() {
  local version="$1" target archive checksum
  [[ -f "${PROJECT_ROOT}/LICENSE" ]] || die "LICENSE file is required for release archives."
  built_archives=()
  for target in aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu; do
    archive="${DIST_DIR}/aibox-v${version}-${target}.tar.gz"
    checksum="${archive}.sha256"
    [[ -f "${archive}" ]] || die "Missing release archive ${archive}; run --steps build first."
    [[ -f "${checksum}" ]] || die "Missing release checksum ${checksum}; run --steps build first."
    built_archives+=("${archive}" "${checksum}")
  done
}

release_license_name() {
  local first_line
  first_line="$(sed -n '1p' "${PROJECT_ROOT}/LICENSE" | tr -d '\r')"
  [[ -n "${first_line}" ]] || die "LICENSE first line is empty; cannot derive README license notice."
  printf '%s' "${first_line}"
}

release_validate_license_guardrails() {
  [[ -f "${PROJECT_ROOT}/LICENSE" ]] || die "LICENSE file is required before release."
  [[ -f "${PROJECT_ROOT}/README.md" ]] || die "README.md is required before release."

  local license_name expected_notice readme_text
  license_name="$(release_license_name)"
  expected_notice="Unless otherwise noted, the copyright holder grants the ${license_name} for all versions of this repository, including historical commits and tags."
  readme_text="$(tr '\n' ' ' < "${PROJECT_ROOT}/README.md" | tr -s ' ')"
  if [[ "${readme_text}" != *"${expected_notice}"* ]]; then
    die "README.md must contain the retroactive license notice: ${expected_notice}"
  fi
}

# Release validation evidence is local by design. Each marker is bound to the
# exact candidate commit, requested version, and Rust toolchain. This lets an
# interrupted release resume without weakening the gate or trusting evidence
# from a different source tree.
release_evidence_init() {
  local version="$1"
  RELEASE_PHASE="${2:-container}"
  RELEASE_CANDIDATE_SHA="$(git rev-parse HEAD)"
  RELEASE_TOOLCHAIN_FINGERPRINT="$(rustc -Vv | sha256_stdin)"
  if [[ -n "$(git status --porcelain)" ]]; then
    RELEASE_TREE_STATE="dirty"
  else
    RELEASE_TREE_STATE="clean"
  fi
  RELEASE_EVIDENCE_DIR="${DIST_DIR}/release-evidence/v${version}/${RELEASE_CANDIDATE_SHA}"
  RELEASE_LOG_DIR="${RELEASE_EVIDENCE_DIR}/logs"
  RELEASE_TIMING_LOG="${RELEASE_EVIDENCE_DIR}/timing-events.tsv"
  mkdir -p "${RELEASE_LOG_DIR}"
  export RELEASE_PHASE RELEASE_CANDIDATE_SHA RELEASE_TOOLCHAIN_FINGERPRINT RELEASE_TREE_STATE RELEASE_EVIDENCE_DIR RELEASE_LOG_DIR RELEASE_TIMING_LOG
}

release_evidence_key_path() {
  local key="$1"
  printf '%s/%s.env' "${RELEASE_EVIDENCE_DIR}" "${key//[^a-zA-Z0-9._-]/_}"
}

# Timing events deliberately use an append-only log instead of the evidence
# markers above. Markers describe the latest reusable successful result; this
# event stream preserves failed attempts and every resumed invocation, which is
# the information needed to account for real release wall time.
release_timing_record_event() {
  local event="$1" status="$2" step="$3" started_at="$4" completed_at="$5"
  local duration="$6" exit_code="$7" details="${8:-}"
  [[ -n "${RELEASE_TIMING_LOG:-}" ]] || return 0
  if [[ ! -e "${RELEASE_TIMING_LOG}" ]]; then
    printf 'event\trun_id\tphase\tstatus\tstep\tstarted_at\tcompleted_at\tduration_seconds\texit_code\tdetails\n' \
      > "${RELEASE_TIMING_LOG}"
  fi
  # All fields are internal identifiers or ISO timestamps. Keep the event
  # format one-record-per-line even if a future caller supplies prose details.
  details="${details//$'\t'/ }"
  details="${details//$'\n'/ }"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${event}" "${RELEASE_TIMING_RUN_ID:-unknown}" "${RELEASE_PHASE}" \
    "${status}" "${step}" "${started_at}" "${completed_at}" "${duration}" \
    "${exit_code}" "${details}" >> "${RELEASE_TIMING_LOG}"
}

release_timing_begin() {
  local selected_steps="${1:-}"
  RELEASE_TIMING_RUN_STARTED_EPOCH="$(date +%s)"
  RELEASE_TIMING_RUN_STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  RELEASE_TIMING_RUN_ID="${RELEASE_PHASE}-$(date -u +%Y%m%dT%H%M%SZ)-$$"
  export RELEASE_TIMING_RUN_STARTED_EPOCH RELEASE_TIMING_RUN_STARTED_AT RELEASE_TIMING_RUN_ID
  release_timing_record_event run started "release-${RELEASE_PHASE}" \
    "${RELEASE_TIMING_RUN_STARTED_AT}" "" 0 0 \
    "steps=${selected_steps};parallelism=${AIBOX_RELEASE_PARALLELISM:-2}"
}

release_timing_finish() {
  local duration="${1:-$(( $(date +%s) - RELEASE_TIMING_RUN_STARTED_EPOCH ))}"
  local status="${2:-completed}"
  local exit_code=0
  [[ "${status}" == "completed" ]] || exit_code=1
  release_timing_record_event run "${status}" "release-${RELEASE_PHASE}" \
    "${RELEASE_TIMING_RUN_STARTED_AT}" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    "${duration}" "${exit_code}" ""
}

release_companion_fingerprint() {
  local key="${PROJECT_ROOT}/.aibox-e2e-runner-home/.ssh/id_ed25519"
  local host="${AIBOX_E2E_HOST:-aibox-e2e-testrunner}"
  [[ -f "${key}" ]] || return 1
  {
    printf 'host=%s\n' "${host}"
    ssh -i "${key}" \
      -o StrictHostKeyChecking=no \
      -o UserKnownHostsFile=/dev/null \
      -o ConnectTimeout=5 \
      -o LogLevel=ERROR \
      "testuser@${host}" \
      'printf "container="; cat /etc/hostname 2>/dev/null || true; uname -a; cat /etc/os-release 2>/dev/null || true; tmux -V 2>/dev/null || true; yazi --version 2>/dev/null || true; asciinema --version 2>/dev/null || true; if command -v docker >/dev/null 2>&1; then docker version --format "{{.Server.Version}}" 2>/dev/null || true; elif command -v podman >/dev/null 2>&1; then podman version --format "{{.Server.Version}}" 2>/dev/null || true; fi'
  } | sha256_stdin
}

release_evidence_scope() {
  local key="$1"
  case "${key}" in
    audit)
      # Advisory data changes independently of the repository. A UTC-day scope
      # gives audit evidence a maximum useful lifetime of 24 hours.
      date -u +%Y-%m-%d
      ;;
    e2e|e2e-*|visual-*)
      release_companion_fingerprint
      ;;
    test)
      printf 'render-local=%s' "${AIBOX_RELEASE_SKIP_RENDER_LOCAL:-0}"
      ;;
    *)
      printf 'source-bound'
      ;;
  esac
}

release_evidence_valid() {
  local key="$1" version="$2" marker scope
  marker="$(release_evidence_key_path "${key}")"
  [[ "${AIBOX_RELEASE_REUSE_EVIDENCE:-1}" != "0" ]] || return 1
  [[ "${RELEASE_TREE_STATE}" == "clean" ]] || return 1
  [[ -f "${marker}" ]] || return 1
  grep -Fqx "version=${version}" "${marker}" || return 1
  grep -Fqx "commit=${RELEASE_CANDIDATE_SHA}" "${marker}" || return 1
  grep -Fqx "toolchain=${RELEASE_TOOLCHAIN_FINGERPRINT}" "${marker}" || return 1
  grep -Fqx "tree_state=clean" "${marker}" || return 1
  grep -Fqx "phase=${RELEASE_PHASE}" "${marker}" || return 1
  scope="$(release_evidence_scope "${key}")" || return 1
  grep -Fqx "scope=${scope}" "${marker}" || return 1

  if [[ "${key}" == "build-linux" ]]; then
    local target archive checksum expected actual
    for target in aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu; do
      archive="${DIST_DIR}/aibox-v${version}-${target}.tar.gz"
      checksum="${archive}.sha256"
      [[ -f "${archive}" && -f "${checksum}" ]] || return 1
      expected="$(awk 'NR == 1 { print $1 }' "${checksum}")"
      actual="$(sha256_file "${archive}")"
      [[ -n "${expected}" && "${expected}" == "${actual}" ]] || return 1
    done
  fi

  if [[ "${key}" == "build-macos" ]]; then
    local target archive checksum expected actual
    for target in aarch64-apple-darwin x86_64-apple-darwin; do
      archive="${DIST_DIR}/aibox-v${version}-${target}.tar.gz"
      checksum="${archive}.sha256"
      [[ -f "${archive}" && -f "${checksum}" ]] || return 1
      expected="$(awk 'NR == 1 { print $1 }' "${checksum}")"
      actual="$(sha256_file "${archive}")"
      [[ -n "${expected}" && "${expected}" == "${actual}" ]] || return 1
    done
  fi

  # Published image tags are external mutable state. Always invoke the image
  # publisher; BuildKit can reuse its cache and the publisher verifies GHCR.
  [[ "${key}" != "publish-images" ]] || return 1
}

release_record_evidence() {
  local key="$1" version="$2" duration="$3" marker tmp scope
  marker="$(release_evidence_key_path "${key}")"
  tmp="${marker}.tmp.$$"
  scope="$(release_evidence_scope "${key}")" \
    || die "Could not fingerprint release evidence scope for ${key}"
  {
    printf 'version=%s\n' "${version}"
    printf 'commit=%s\n' "${RELEASE_CANDIDATE_SHA}"
    printf 'toolchain=%s\n' "${RELEASE_TOOLCHAIN_FINGERPRINT}"
    printf 'tree_state=%s\n' "${RELEASE_TREE_STATE}"
    printf 'phase=%s\n' "${RELEASE_PHASE}"
    printf 'scope=%s\n' "${scope}"
    printf 'step=%s\n' "${key}"
    printf 'duration_seconds=%s\n' "${duration}"
    printf 'completed_at=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  } > "${tmp}"
  mv "${tmp}" "${marker}"
}

release_run_evidenced_step() {
  local key="$1" version="$2" label="$3"
  shift 3
  if release_evidence_valid "${key}" "${version}"; then
    local reused_at
    reused_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    release_timing_record_event step reused "${key}" "${reused_at}" "${reused_at}" 0 0 ""
    ok "${label}: reusing evidence for ${RELEASE_CANDIDATE_SHA:0:12}"
    return 0
  fi

  local started started_at completed_at duration status
  started="$(date +%s)"
  started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if "$@"; then
    status=0
  else
    status="$?"
  fi
  duration=$(( $(date +%s) - started ))
  completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [[ "${status}" -ne 0 ]]; then
    release_timing_record_event step failed "${key}" "${started_at}" "${completed_at}" \
      "${duration}" "${status}" ""
    return "${status}"
  fi
  release_record_evidence "${key}" "${version}" "${duration}"
  release_timing_record_event step passed "${key}" "${started_at}" "${completed_at}" \
    "${duration}" 0 ""
  ok "${label}: completed in ${duration}s"
}

release_local_test_gate() {
  cmd_test
  case "${AIBOX_RELEASE_SKIP_RENDER_LOCAL:-}" in
    1|true|yes)
      warn "Skipping Tier 3 local Starship rendered-color tests because AIBOX_RELEASE_SKIP_RENDER_LOCAL=${AIBOX_RELEASE_SKIP_RENDER_LOCAL}."
      ;;
    *)
      cmd_test_e2e_render_starship
      ;;
  esac
}

release_companion_e2e_gate() {
  case "${AIBOX_RELEASE_SKIP_COMPANION_E2E:-}" in
    1|true|yes)
      warn "Skipping Tier 2 SSH companion E2E during release because AIBOX_RELEASE_SKIP_COMPANION_E2E=${AIBOX_RELEASE_SKIP_COMPANION_E2E}. Re-run ./scripts/maintain.sh test-e2e after rebuilding the companion."
      ;;
    *)
      cmd_test_e2e
      ;;
  esac
}

release_companion_e2e_core_gate() {
  cmd_test_e2e_shard core
}

release_companion_e2e_addon_gate() {
  cmd_test_e2e_shard addon
}

release_companion_e2e_latex_gate() {
  cmd_test_e2e_shard latex
}

release_visual_gate() {
  case "${AIBOX_RELEASE_VISUAL_E2E:-skip}" in
    status) cmd_test_e2e_visual_status ;;
    tabs|tools) cmd_test_e2e_visual_tabs ;;
    yazi) cmd_test_e2e_visual_yazi ;;
    render)
      cmd_test_e2e_render_tmux
      cmd_test_e2e_render_yazi
      ;;
    full)
      cmd_test_e2e_visual
      cmd_test_e2e_render_tmux
      cmd_test_e2e_render_yazi
      ;;
    docs|captures) cmd_test_e2e_doc_captures ;;
    *)
      die "Unknown AIBOX_RELEASE_VISUAL_E2E=${AIBOX_RELEASE_VISUAL_E2E:-}; expected skip, status, tabs, yazi, render, full, or docs"
      ;;
  esac
}

release_audit_gate() {
  info "Running cargo audit..."
  command -v cargo-audit &>/dev/null \
    || (cd "${CLI_DIR}" && cargo install cargo-audit --quiet)
  local audit_db="${TMPDIR:-/tmp}/aibox-cargo-advisory-db"
  mkdir -p "${audit_db}"
  (cd "${CLI_DIR}" && cargo audit --db "${audit_db}") \
    || die "cargo audit found advisories — resolve before releasing"
  ok "Audit clean"
}

release_build_linux_target() {
  local version="$1" target="$2"
  info "  → ${target}"
  (cd "${CLI_DIR}" && \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    cargo build --release --target "${target}") \
    || return 1

  local binary_name="aibox-v${version}-${target}"
  cp "${CLI_DIR}/target/${target}/release/aibox" "${DIST_DIR}/${binary_name}"
  tar -czf "${DIST_DIR}/${binary_name}.tar.gz" \
    -C "${DIST_DIR}" "${binary_name}" \
    -C "${PROJECT_ROOT}" LICENSE
  rm "${DIST_DIR}/${binary_name}"
  sha256_file "${DIST_DIR}/${binary_name}.tar.gz" > "${DIST_DIR}/${binary_name}.tar.gz.sha256"
  ok "Built ${binary_name}.tar.gz"
}

release_build_linux_gate() {
  local version="$1" target pid status=0
  local targets=(aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu)
  local pids=()
  mkdir -p "${DIST_DIR}"
  info "Building CLI release targets in parallel..."
  for target in "${targets[@]}"; do
    release_build_linux_target "${version}" "${target}" &
    pids+=("$!")
  done
  for pid in "${pids[@]}"; do
    if ! wait "${pid}"; then
      status=1
    fi
  done
  [[ "${status}" -eq 0 ]] || die "cargo build failed for one or more Linux targets"
}

release_version_smoke_gate() {
  local version="$1" machine target candidate reported
  machine="$(uname -m)"
  case "${machine}" in
    x86_64|amd64) target="x86_64-unknown-linux-gnu" ;;
    aarch64|arm64) target="aarch64-unknown-linux-gnu" ;;
    *) target="" ;;
  esac
  candidate="${CLI_DIR}/target/${target}/release/aibox"
  if [[ -z "${target}" || ! -x "${candidate}" ]]; then
    info "No runnable cross-target binary found; building the native release binary..."
    (cd "${CLI_DIR}" && cargo build --release --quiet) \
      || die "native cargo build failed (needed for --version smoke test)"
    candidate="${CLI_DIR}/target/release/aibox"
  fi
  reported="$("${candidate}" --version | awk '{print $NF}')"
  [[ "${reported}" == "${version}" ]] \
    || die "aibox --version reports '${reported}' but the tag being cut is '${version}'. Fix Cargo.toml before retrying."
  ok "aibox --version = ${reported} (matches tag)"
}

release_build_macos_gate() {
  local version="$1"
  info "Building macOS release artifacts..."
  "${SCRIPT_DIR}/build-macos.sh" "${version}"
}

release_publish_images_gate() {
  local version="$1"
  info "Publishing container images..."
  cmd_publish_images_for_release "${version}"
}

release_cleanup_parallel_children() {
  local index pid
  for index in "${!pids[@]}"; do
    pid="${pids[$index]}"
    kill "${pid}" 2>/dev/null || true
  done
  for index in "${!pids[@]}"; do
    wait "${pids[$index]}" 2>/dev/null || true
  done
}

release_run_parallel_validation() {
  local version="$1"
  shift
  local max_jobs="${AIBOX_RELEASE_PARALLELISM:-2}"
  [[ "${max_jobs}" =~ ^[1-9][0-9]*$ ]] \
    || die "AIBOX_RELEASE_PARALLELISM must be a positive integer"

  local specs=("$@") active=0 status=0 next=0 total="${#specs[@]}"
  local spec key label command log pid index completed job_status status_file
  local pids=() labels=() logs=() status_files=()

  trap 'release_cleanup_parallel_children' EXIT
  trap 'release_cleanup_parallel_children; exit 130' INT TERM

  while [[ "${next}" -lt "${total}" || "${active}" -gt 0 ]]; do
    while [[ "${next}" -lt "${total}" && "${active}" -lt "${max_jobs}" ]]; do
      spec="${specs[$next]}"
      IFS='|' read -r key label command <<< "${spec}"
      log="${RELEASE_LOG_DIR}/${key}.log"
      status_file="${log}.status"
      rm -f "${status_file}"
      (
        trap - EXIT INT TERM
        set +e
        (release_run_evidenced_step "${key}" "${version}" "${label}" "${command}" "${version}") \
          > "${log}" 2>&1
        job_status="$?"
        printf '%s\n' "${job_status}" > "${status_file}.tmp.$$"
        mv "${status_file}.tmp.$$" "${status_file}"
        exit "${job_status}"
      ) &
      pid="$!"
      pids+=("${pid}")
      labels+=("${label}")
      logs+=("${log}")
      status_files+=("${status_file}")
      active=$((active + 1))
      next=$((next + 1))
    done

    completed=0
    while [[ "${completed}" -eq 0 ]]; do
      for index in "${!pids[@]}"; do
        status_file="${status_files[$index]}"
        if [[ -f "${status_file}" ]]; then
          job_status="$(sed -n '1p' "${status_file}")"
        elif kill -0 "${pids[$index]}" 2>/dev/null; then
          continue
        else
          job_status=1
          warn "${labels[$index]} terminated without reporting status"
        fi
        wait "${pids[$index]}" 2>/dev/null || true
        if [[ "${job_status}" -eq 0 ]]; then
          tail -n 200 "${logs[$index]}"
        else
          warn "${labels[$index]} failed; log follows: ${logs[$index]}"
          tail -n 240 "${logs[$index]}" >&2
          status=1
        fi
        rm -f "${status_file}"
        unset 'pids[index]' 'labels[index]' 'logs[index]' 'status_files[index]'
        active=$((active - 1))
        completed=1
        break
      done
      [[ "${completed}" -eq 1 ]] || sleep 0.2
    done
  done

  trap - EXIT INT TERM
  if [[ "${status}" -ne 0 ]]; then
    # Failed validation attempts are already in the append-only timing log.
    # Refresh the human-readable view before stopping so an interrupted release
    # has an immediately useful cumulative report.
    release_timing_finish "$(( $(date +%s) - RELEASE_TIMING_RUN_STARTED_EPOCH ))" failed
    release_write_timing_report "${version}"
    die "One or more parallel release validation jobs failed"
  fi
}

release_write_timing_report() {
  local version="$1" total_duration="${2:-}" report phase_label marker
  local timing_log total_steps passed_steps failed_steps reused_steps cumulative_duration
  local completed_runs failed_runs cumulative_run_duration
  case "${RELEASE_PHASE}" in
    host)
      report="${DIST_DIR}/RELEASE-HOST-TIMINGS.md"
      phase_label="Host"
      ;;
    *)
      report="${DIST_DIR}/RELEASE-TIMINGS.md"
      phase_label="Container"
      ;;
  esac
  timing_log="${RELEASE_EVIDENCE_DIR}/timing-events.tsv"
  {
    printf '# %s release timings for v%s\n\n' "${phase_label}" "${version}"
    printf -- '- Candidate: `%s`\n' "${RELEASE_CANDIDATE_SHA}"
    printf -- '- Parallelism: `%s`\n\n' "${AIBOX_RELEASE_PARALLELISM:-2}"
    if [[ -n "${total_duration}" ]]; then
      printf -- '- Most recent command duration: `%ss`\n\n' "${total_duration}"
    fi
    if [[ -f "${timing_log}" ]]; then
      total_steps="$(awk -F '\t' 'NR > 1 && $1 == "step" { count++ } END { print count + 0 }' "${timing_log}")"
      passed_steps="$(awk -F '\t' 'NR > 1 && $1 == "step" && $4 == "passed" { count++ } END { print count + 0 }' "${timing_log}")"
      failed_steps="$(awk -F '\t' 'NR > 1 && $1 == "step" && $4 == "failed" { count++ } END { print count + 0 }' "${timing_log}")"
      reused_steps="$(awk -F '\t' 'NR > 1 && $1 == "step" && $4 == "reused" { count++ } END { print count + 0 }' "${timing_log}")"
      cumulative_duration="$(awk -F '\t' 'NR > 1 && $1 == "step" { total += $8 } END { print total + 0 }' "${timing_log}")"
      completed_runs="$(awk -F '\t' 'NR > 1 && $1 == "run" && $4 == "completed" { count++ } END { print count + 0 }' "${timing_log}")"
      failed_runs="$(awk -F '\t' 'NR > 1 && $1 == "run" && $4 == "failed" { count++ } END { print count + 0 }' "${timing_log}")"
      cumulative_run_duration="$(awk -F '\t' 'NR > 1 && $1 == "run" && ($4 == "completed" || $4 == "failed") { total += $8 } END { print total + 0 }' "${timing_log}")"
      printf '## Cumulative attempts\n\n'
      printf -- '- Recorded step attempts: `%s` (`%s` passed, `%s` failed, `%s` reused)\n' \
        "${total_steps}" "${passed_steps}" "${failed_steps}" "${reused_steps}"
      printf -- '- Cumulative executed-step duration: `%ss`\n' "${cumulative_duration}"
      printf -- '- Completed command runs: `%s`; failed command runs: `%s`; cumulative command duration: `%ss`\n' \
        "${completed_runs}" "${failed_runs}" "${cumulative_run_duration}"
      printf -- '- Append-only event log: `%s`\n\n' "${timing_log#${PROJECT_ROOT}/}"
      printf '| Run | Step | Result | Duration | Started | Completed |\n'
      printf '|---|---|---|---:|---|---|\n'
      awk -F '\t' 'NR > 1 && $1 == "step" {
        printf "| %s | %s | %s | %ss | %s | %s |\\n", $2, $5, $4, $8, $6, $7
      }' "${timing_log}"
      printf '\n'
    fi
    printf '## Latest reusable evidence\n\n'
    printf '| Step | Duration | Completed |\n'
    printf '|---|---:|---|\n'
    for marker in "${RELEASE_EVIDENCE_DIR}"/*.env; do
      [[ -f "${marker}" ]] || continue
      grep -Fqx "phase=${RELEASE_PHASE}" "${marker}" || continue
      printf '| %s | %ss | %s |\n' \
        "$(awk -F= '$1 == "step" { print $2 }' "${marker}")" \
        "$(awk -F= '$1 == "duration_seconds" { print $2 }' "${marker}")" \
        "$(awk -F= '$1 == "completed_at" { print $2 }' "${marker}")"
    done
  } > "${report}"
  ok "Release timing evidence written to ${report}"
}

cmd_release() {
  local version="${1:-}"
  local release_started_epoch="$(date +%s)"
  if [[ "${version}" == "--list-steps" ]]; then
    release_list_steps
    return 0
  fi
  [[ -z "${version}" ]] && release_usage
  shift || true

  local steps_spec="all"
  local skip_spec=""
  while [[ "$#" -gt 0 ]]; do
    case "$1" in
      --list-steps)
        release_list_steps
        return 0
        ;;
      --steps)
        shift
        [[ "$#" -gt 0 ]] || release_usage
        steps_spec="$1"
        ;;
      --steps=*)
        steps_spec="${1#--steps=}"
        ;;
      --skip)
        shift
        [[ "$#" -gt 0 ]] || release_usage
        skip_spec="$1"
        ;;
      --skip=*)
        skip_spec="${1#--skip=}"
        ;;
      *)
        release_usage
        ;;
    esac
    shift || true
  done

  # Validate semver (simple check)
  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
    die "Version must be semver: X.Y.Z or X.Y.Z-prerelease (got: ${version})"
  fi

  local tag="v${version}"
  local release_branch
  release_branch="$(release_branch_for_version "${version}")"
  local release_steps=()
  local release_step_tokens=()
  local built_archives=()
  release_parse_steps "${steps_spec}"
  release_apply_skip_steps "${skip_spec}"
  release_validate_license_guardrails

  info "Preparing release ${tag} with steps: $(release_steps_joined)"

  if release_requires_clean_tree && [[ -n "$(git status --porcelain)" ]]; then
    die "Selected release steps require a clean tree. Commit or stash changes first, or choose non-publishing steps such as --steps audit,build."
  fi

  if release_step_requested tag && git rev-parse "${tag}" &>/dev/null; then
    die "Tag ${tag} already exists."
  fi
  if release_step_requested github-release && ! release_step_requested tag && ! git rev-parse "${tag}" &>/dev/null; then
    die "Tag ${tag} does not exist. Include the 'tag' step or run it before github-release."
  fi

  if release_step_requested state; then
    info "Checking dependency, addon, image, and harness state..."
    cmd_release_check_state --require-network
    ok "Release state report written"
    if [[ -t 0 ]]; then
      warn "Review dist/RELEASE-STATE.md. Press Enter to continue, or Ctrl-C to abort and update dependencies first."
      read -r
    fi
  fi

  if release_step_requested doctors; then
    if [[ "${AIBOX_RELEASE_SKIP_DOCTORS:-}" == "1" ]]; then
      warn "AIBOX_RELEASE_SKIP_DOCTORS=1 set — skipping Phase 0 doctor checks. Owner accepts the risk."
    else
      info "Running release doctor checks..."
      cmd_release_doctors
      if [[ -t 0 ]]; then
        warn "Review dist/RELEASE-DOCTORS.md for any warnings. Press Enter to continue."
        read -r
      fi
    fi
  fi

  if release_step_requested sync; then
    cmd_sync_processkit
    if [[ -n "$(git status --porcelain)" ]]; then
      echo ""
      die "processkit_vocab.rs was updated. Review the diff, make any required CLI changes, commit, then re-run release."
    fi
  fi

  if release_step_requested version; then
    local current_cargo_version
    current_cargo_version=$(grep -m1 '^version = ' "${CLI_DIR}/Cargo.toml" | sed -E 's/version = "(.+)"/\1/')
    if [[ "${current_cargo_version}" != "${version}" ]]; then
      info "Bumping cli/Cargo.toml ${current_cargo_version} → ${version}..."
      # macOS/BSD sed and GNU sed differ on -i; write atomically via a tmp file.
      local tmp_cargo
      tmp_cargo=$(mktemp)
      sed -E "s/^version = \"${current_cargo_version}\"$/version = \"${version}\"/" \
        "${CLI_DIR}/Cargo.toml" > "${tmp_cargo}"
      mv "${tmp_cargo}" "${CLI_DIR}/Cargo.toml"
      # Refresh Cargo.lock so the new version is locked.
      (cd "${CLI_DIR}" && cargo metadata --format-version 1 --quiet >/dev/null) \
        || die "cargo metadata failed after version bump — review Cargo.toml"
      git add "${CLI_DIR}/Cargo.toml" "${CLI_DIR}/Cargo.lock"
      git commit -m "chore: bump CLI version to ${version}" \
        -m "Version-Line-Port: not-applicable" \
        || die "failed to commit Cargo.toml/Cargo.lock bump"
      ok "Cargo.toml bumped and committed"
    else
      ok "Cargo.toml already at ${version}"
    fi
  fi

  # Everything below this point is validated against the immutable candidate
  # commit. Evidence from an interrupted run is reusable only for this SHA.
  release_evidence_init "${version}"
  release_timing_begin "$(release_steps_joined)"
  info "Release candidate: ${RELEASE_CANDIDATE_SHA}"

  # These gates own independent resources. Run a bounded number concurrently,
  # retain separate logs, wait for every job, then fail as a group. The default
  # of two avoids oversubscribing developer laptops; override locally with
  # AIBOX_RELEASE_PARALLELISM when more CPU and memory are available.
  local validation_specs=()
  if release_step_requested test; then
    validation_specs+=("test|Local fmt, Clippy, tests, and Starship render|release_local_test_gate")
  fi
  if release_step_requested audit; then
    validation_specs+=("audit|Cargo dependency audit|release_audit_gate")
  fi
  if release_step_requested e2e; then
    case "${AIBOX_RELEASE_SKIP_COMPANION_E2E:-}" in
      1|true|yes)
        warn "Skipping Tier 2 SSH companion E2E during release because AIBOX_RELEASE_SKIP_COMPANION_E2E=${AIBOX_RELEASE_SKIP_COMPANION_E2E}. Re-run ./scripts/maintain.sh test-e2e after rebuilding the companion."
        ;;
      *)
        validation_specs+=("e2e-core|Tier 2 companion E2E core shard|release_companion_e2e_core_gate")
        ;;
    esac
  fi
  if release_step_requested build-linux; then
    validation_specs+=("build-linux|Linux release artifacts|release_build_linux_gate")
  fi
  if [[ "${#validation_specs[@]}" -gt 0 ]]; then
    release_run_parallel_validation "${version}" "${validation_specs[@]}"
  fi

  # The addon and LaTeX shards each build a large image on the single companion.
  # Keep them outside the parallel validation pool and give each its own
  # candidate-bound evidence marker so a resumed release reruns only the shard
  # that failed.
  if release_step_requested e2e; then
    case "${AIBOX_RELEASE_SKIP_COMPANION_E2E:-}" in
      1|true|yes) ;;
      *)
        release_run_parallel_validation "${version}" \
          "e2e-addon|Tier 2 companion E2E addon shard|release_companion_e2e_addon_gate"
        release_run_parallel_validation "${version}" \
          "e2e-latex|Tier 2 companion E2E LaTeX shard|release_companion_e2e_latex_gate"
        ;;
    esac
  fi

  if release_step_requested visual; then
    case "${AIBOX_RELEASE_VISUAL_E2E:-skip}" in
      skip|"")
        warn "Skipping opt-in visual E2E during release. The release agent must justify this in notes or handover, or run AIBOX_RELEASE_VISUAL_E2E=<status|tabs|yazi|render|full|docs>."
        ;;
      *)
        release_run_evidenced_step "visual-${AIBOX_RELEASE_VISUAL_E2E}" "${version}" \
          "Visual E2E (${AIBOX_RELEASE_VISUAL_E2E})" release_visual_gate
        ;;
    esac
  fi

  if release_step_requested version-smoke; then
    info "Verifying 'aibox --version' matches ${version}..."
    release_run_evidenced_step "version-smoke" "${version}" "CLI version smoke" \
      release_version_smoke_gate "${version}"
  fi

  if release_step_requested push-main; then
    publish_release_candidate "${version}" "${release_branch}"
  fi

  if release_step_requested notes; then
    local notes_file="${DIST_DIR}/RELEASE-NOTES.md"
    if [[ ! -f "${notes_file}" ]] || ! grep -q "${tag}" "${notes_file}" 2>/dev/null; then
      info "Generating release-notes scaffold at ${notes_file}..."
      local prev_tag
      prev_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
      {
        echo "# aibox ${tag}"
        echo ""
        if [[ -n "${prev_tag}" ]]; then
          echo "## Changes since ${prev_tag}"
          echo ""
          git log --oneline "${prev_tag}..HEAD" | sed 's/^/- /'
        else
          git log --oneline HEAD | head -20 | sed 's/^/- /'
        fi
      } > "${notes_file}"
      if [[ -t 0 ]]; then
        warn "Auto-generated release notes written to dist/RELEASE-NOTES.md."
        warn "Edit them now, then press Enter to continue (Ctrl-C to abort)."
        read -r
      else
        warn "Running non-interactively — using auto-generated notes. Edit dist/RELEASE-NOTES.md and re-run if needed."
      fi
    else
      ok "Using hand-written release notes from ${notes_file}"
    fi
  fi

  if release_step_requested tag; then
    info "Tagging and pushing ${tag}..."
    git tag -a "${tag}" -m "Release ${tag}"
    git push origin "${tag}"
    ok "Tag ${tag} pushed"
  fi

  if release_step_requested github-release; then
    if [[ "${#built_archives[@]}" -eq 0 ]]; then
      release_collect_linux_archives "${version}"
    fi
    local notes_file="${DIST_DIR}/RELEASE-NOTES.md"
    [[ -f "${notes_file}" ]] || die "Missing ${notes_file}; run the 'notes' step first or include the publish alias."
    info "Creating GitHub release ${tag}..."
    gh release create "${tag}" \
      --repo "${GITHUB_REPO}" \
      --title "aibox ${tag}" \
      --notes-file "${notes_file}" \
      "${PROJECT_ROOT}/LICENSE" \
      "${built_archives[@]}"
    ok "GitHub release ${tag} created with Linux binaries and LICENSE"
  fi

  if release_step_requested docs; then
    info "Deploying documentation..."
    local docs_line="v${version%%.*}.x"
    cmd_docs_deploy --line "${docs_line}" --version "v${version}"
    ok "Documentation deployed"
  fi

  if release_step_requested prompt; then
    local prompt_file="${DIST_DIR}/RELEASE-PROMPT.md"
    {
      echo "# Host-side steps for aibox ${tag}"
      echo ""
      echo "Linux binaries are already uploaded to the GitHub release."
      echo "Run the following on the macOS host to sync the checkout and complete the release:"
      echo ""
      echo "\`\`\`bash"
      echo "git fetch origin ${release_branch}"
      echo "git switch ${release_branch}"
      echo "git reset --keep origin/${release_branch}"
      echo "./scripts/maintain.sh release-host ${version}"
      echo "\`\`\`"
      echo ""
      echo "This will:"
      echo "- Verify the host checkout is current with the version-line release branch and contains ${tag}"
      echo "- Build macOS binaries (aarch64-apple-darwin, x86_64-apple-darwin)"
      echo "- Upload them to the existing GitHub release ${tag}"
      echo "- Build and push container images to GHCR"
      echo "- Refresh repo-owned generated runtime surfaces after the image tags exist"
      echo "- Commit and push generated runtime changes if they drift"
    } > "${prompt_file}"

    ok "Host-side prompt written to dist/RELEASE-PROMPT.md"
  fi

  # ── Summary ──────────────────────────────────────────────────────────────
  echo ""
  local release_duration="$(( $(date +%s) - release_started_epoch ))"
  release_timing_finish "${release_duration}"
  release_write_timing_report "${version}" "${release_duration}"
  echo "${bold}Release ${tag} selected steps complete: $(release_steps_joined).${reset}"
  echo ""
  echo "  GitHub release: https://github.com/projectious-work/aibox/releases/tag/${tag}"
  if release_step_requested github-release; then
    echo "  Linux binaries uploaded:"
    for a in "${built_archives[@]}"; do
      echo "    $(basename "${a}")"
    done
  fi
  if release_step_requested docs; then
    echo "  Documentation: deployed to gh-pages"
  fi
  if release_step_requested prompt; then
    echo ""
    echo "  ${bold}Remaining (macOS host):${reset} ./scripts/maintain.sh release-host ${version}"
  fi
}

release_branch_for_version() {
  local version="$1"
  case "${version}" in
    0.*) printf '%s\n' 'v0.x-release' ;;
    1.*-*) printf '%s\n' 'v1.x-pre-release' ;;
    1.*) printf '%s\n' 'v1.x-release' ;;
    *) die "Unsupported release line for ${version}; add a branch mapping first." ;;
  esac
}

publish_release_candidate() {
  local version="$1" release_branch="$2"
  local candidate_branch="chore/release-v${version}" pr_url

  [[ "$(git branch --show-current)" == "${release_branch}" ]] \
    || die "Release ${version} must be published from ${release_branch}."

  git fetch origin "${release_branch}"
  git merge-base --is-ancestor "origin/${release_branch}" HEAD \
    || die "Local ${release_branch} does not descend from origin/${release_branch}; reconcile it before publishing."

  if [[ "$(git rev-parse HEAD)" == "$(git rev-parse "origin/${release_branch}")" ]]; then
    ok "${release_branch} is already current on origin"
    return
  fi

  if git ls-remote --exit-code --heads origin "${candidate_branch}" >/dev/null 2>&1; then
    die "Remote branch ${candidate_branch} already exists; inspect or merge it before publishing."
  fi

  info "Promoting the release candidate to protected branch ${release_branch}..."
  git switch -c "${candidate_branch}"
  git push -u origin "${candidate_branch}"
  pr_url="$(gh pr create \
    --base "${release_branch}" \
    --head "${candidate_branch}" \
    --title "chore: release v${version}" \
    --body "Promotes the validated v${version} release candidate to ${release_branch}.")" \
    || die "Could not create release-candidate PR."
  gh pr merge "${pr_url}" --merge --delete-branch \
    || die "Could not merge release-candidate PR ${pr_url}."
  git switch "${release_branch}"
  git pull --ff-only origin "${release_branch}"
  ok "Release candidate merged into ${release_branch}"
}

cmd_release_finalize_runtime() {
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh release-finalize-runtime <version>  (e.g. 0.10.2)"

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
    die "Version must be semver: X.Y.Z or X.Y.Z-prerelease (got: ${version})"
  fi

  local release_branch finalization_branch pr_url
  release_branch="$(release_branch_for_version "${version}")"
  finalization_branch="chore/refresh-generated-runtime-v${version}"

  info "Refreshing repo-owned generated runtime surfaces..."
  (
    cd "${PROJECT_ROOT}"
    if ! git diff --cached --quiet; then
      die "Staged changes are already present; commit or unstage them before release runtime finalization."
    fi

    AIBOX_ADDONS_DIR="${PROJECT_ROOT}/addons" \
      cargo run --manifest-path "${CLI_DIR}/Cargo.toml" -- apply generated-runtime

    git add -- .devcontainer aibox.lock context/migrations context/templates/aibox-home
    if git diff --cached --quiet -- .devcontainer aibox.lock context/migrations context/templates/aibox-home; then
      ok "Generated runtime surfaces already match v${version}; no commit needed."
    else
      [[ "$(git branch --show-current)" == "${release_branch}" ]] \
        || die "Generated runtime finalization for v${version} must start on ${release_branch}."
      if git ls-remote --exit-code --heads origin "${finalization_branch}" >/dev/null 2>&1; then
        die "Remote branch ${finalization_branch} already exists; inspect or merge it before rerunning release finalization."
      fi

      git switch -c "${finalization_branch}"
      git commit -m "chore: refresh generated runtime for v${version}" \
        -m "Version-Line-Port: not-applicable"
      git push -u origin "${finalization_branch}"
      pr_url="$(gh pr create \
        --base "${release_branch}" \
        --head "${finalization_branch}" \
        --title "chore: refresh generated runtime for v${version}" \
        --body "Post-image generated runtime refresh for v${version}.")" \
        || die "Could not create generated-runtime finalization PR."
      gh pr merge "${pr_url}" --merge --delete-branch \
        || die "Could not merge generated-runtime finalization PR ${pr_url}."
      git switch "${release_branch}"
      git pull --ff-only origin "${release_branch}"
      ok "Generated runtime surfaces merged into ${release_branch} for v${version}."
    fi
  )
}

# ── Host-side release (run on macOS after container-side `release`) ──────────

ensure_release_host_checkout_current() {
  local version="$1" tag release_branch
  tag="v${version}"
  release_branch="$(release_branch_for_version "${version}")"

  info "Verifying host checkout is current with origin/${release_branch} and ${tag}..."
  (
    cd "${PROJECT_ROOT}"
    git fetch origin \
      "refs/heads/${release_branch}:refs/remotes/origin/${release_branch}" \
      "refs/tags/${tag}:refs/tags/${tag}" >/dev/null

    local head remote_head tag_commit
    head=$(git rev-parse HEAD)
    remote_head=$(git rev-parse "origin/${release_branch}")
    tag_commit=$(git rev-parse "${tag}^{commit}" 2>/dev/null) \
      || die "Release tag ${tag} is missing locally. Fetch it from origin before running release-host."

    if ! git merge-base --is-ancestor "${tag_commit}" "${remote_head}"; then
      die "Release tag ${tag} is not reachable from origin/${release_branch}; refusing to build host artifacts from the wrong version line."
    fi

    if [[ "${head}" != "${remote_head}" ]]; then
      die "release-host must run from current origin/${release_branch} containing ${tag}. Run: git fetch origin ${release_branch} && git switch ${release_branch} && git reset --keep origin/${release_branch}"
    fi
  )
  ok "Host checkout matches the ${release_branch} release line and contains ${tag}"
}

cmd_release_host() {
  local version="${1:-}"
  local release_started_epoch="$(date +%s)"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh release-host <version>  (e.g. 0.10.2)"

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
    die "Version must be semver: X.Y.Z or X.Y.Z-prerelease (got: ${version})"
  fi

  local tag="v${version}"
  ensure_release_host_checkout_current "${version}"
  release_evidence_init "${version}" host
  release_timing_begin "release-host"
  info "Host release source: ${RELEASE_CANDIDATE_SHA}"

  # ── Step 1: Build macOS binaries ──────────────────────────────────────────
  # macOS compilation and image publication are independent. Run both with
  # separate logs and aggregate their failures before continuing.
  release_run_parallel_validation "${version}" \
    "build-macos|macOS release artifacts|release_build_macos_gate" \
    "publish-images|GHCR image publication|release_publish_images_gate"

  # ── Step 2: Upload macOS binaries to existing GitHub release ──────────────
  info "Uploading macOS binaries to GitHub release ${tag}..."
  local release_view_output
  if ! release_view_output=$(gh release view "${tag}" --repo "${GITHUB_REPO}" 2>&1); then
    if grep -qiE 'not[[:space:]]+found|HTTP 404' <<<"${release_view_output}"; then
      die "GitHub release ${tag} not found in ${GITHUB_REPO}. Run 'release' in the container first."
    fi
    die "Could not verify GitHub release ${tag} in ${GITHUB_REPO}: ${release_view_output}"
  fi
  gh release upload "${tag}" "${PROJECT_ROOT}/LICENSE" \
    --repo "${GITHUB_REPO}" \
    --clobber \
    || warn "LICENSE upload failed — verify the GitHub release includes LICENSE"
  gh release upload "${tag}" "${DIST_DIR}"/aibox-v${version}-*-apple-darwin.tar.gz "${DIST_DIR}"/aibox-v${version}-*-apple-darwin.tar.gz.sha256 \
    --repo "${GITHUB_REPO}" \
    || warn "Upload failed — binaries may already be attached"
  ok "macOS binaries, checksums, and LICENSE uploaded to ${tag}"

  # ── Step 3: Publish container images ─────────────────────────────────────
  # ── Step 4: Run generated-runtime smoke against the pushed image ──────────
  info "Running generated runtime smoke..."
  cmd_release_runtime_smoke "${version}"

  # ── Step 5: Commit generated runtime surfaces now that images exist ───────
  cmd_release_finalize_runtime "${version}"

  # ── Step 6: Final GHCR sanity check ───────────────────────────────────────
  # cmd_publish_images_for_release also calls this internally; re-running at
  # the end of release-host guards against any later step (smoke, finalize)
  # accidentally clobbering the tags or against a retag that succeeded
  # locally but didn't propagate to GHCR.
  info "Re-verifying GHCR tags after smoke + runtime finalize..."
  verify_release_images_in_ghcr "${version}" "base-debian"
  local release_duration="$(( $(date +%s) - release_started_epoch ))"
  release_timing_finish "${release_duration}"
  release_write_timing_report "${version}" "${release_duration}"

  # ── Done ──────────────────────────────────────────────────────────────────
  echo ""
  ok "Release ${tag} host-side steps complete."
  echo ""
  echo "  macOS binaries: uploaded to GitHub release"
  echo "  Container images: pushed to GHCR and verified live"
  echo "  Runtime smoke: passed (logs in dist/release-smoke/v${version}/)"
  echo "  Generated runtime: refreshed and committed if needed"
}

# ── Container commands ───────────────────────────────────────────────────────

cmd_start() {
  _require_runtime
  export HOST_ROOT WORKSPACE_DIR
  ensure_host_dirs

  local status
  status=$(container_status)
  case "${status}" in
    running)
      info "Container already running — attaching."
      ;;
    exited)
      info "Starting stopped container..."
      if ! compose start "${SERVICE_NAME}" 2>/dev/null; then
        ${RUNTIME_BIN} start "${CONTAINER_NAME}"
      fi
      wait_for_running
      ;;
    missing)
      local image_exists
      image_exists=$(compose images -q "${SERVICE_NAME}" 2>/dev/null || true)
      if [[ -z "${image_exists}" ]]; then
        warn "Image not found — building first..."
        compose build
      fi
      info "Starting container..."
      compose up -d "${SERVICE_NAME}"
      wait_for_running
      ;;
  esac
  cmd_attach
}

cmd_stop() {
  _require_runtime
  export HOST_ROOT WORKSPACE_DIR
  local status
  status=$(container_status)
  if [[ "${status}" == "missing" ]]; then
    warn "Container is not running."
    exit 0
  fi
  info "Stopping container..."
  if ! compose stop "${SERVICE_NAME}" 2>/dev/null; then
    ${RUNTIME_BIN} stop "${CONTAINER_NAME}"
  fi
  ok "Container stopped."
}

cmd_attach() {
  _require_runtime
  export HOST_ROOT WORKSPACE_DIR
  local status
  status=$(container_status)
  if [[ "${status}" != "running" ]]; then
    die "Container is not running. Run './scripts/maintain.sh start' first."
  fi
  info "Attaching — launching tmux..."
  echo ""
  ${RUNTIME_BIN} exec -it \
    --user aibox \
    --env HOME=/home/aibox \
    --env SHELL=/bin/bash \
    --env XDG_CACHE_HOME=/home/aibox/.cache \
    "${CONTAINER_NAME}" \
    tmux new-session -A -s aibox
}

cmd_status() {
  _require_runtime
  export HOST_ROOT WORKSPACE_DIR
  local status
  status=$(container_status)
  case "${status}" in
    running) ok  "Container is ${bold}running${reset}." ;;
    exited)  warn "Container is ${bold}stopped${reset} (run 'start' to resume)." ;;
    missing) warn "Container does not exist (run 'start' to create it)." ;;
  esac
}

cmd_test_visual() {
  info "Running visual smoke tests..."
  "${SCRIPT_DIR}/test-screencasts.sh" all
}

cmd_record_docs() {
  info "Recording all docs screencasts..."
  "${SCRIPT_DIR}/record-asciinema.sh" all
}

# =============================================================================
# Entrypoint
# =============================================================================
if [[ "${AIBOX_MAINTAIN_SOURCE_ONLY:-0}" == "1" ]]; then
  return 0 2>/dev/null || exit 0
fi

COMMAND="${1:-help}"
shift || true

case "${COMMAND}" in
  test)         cmd_test ;;
  test-e2e)     cmd_test_e2e ;;
  test-e2e-visual-status) cmd_test_e2e_visual_status ;;
  test-e2e-visual-tabs) cmd_test_e2e_visual_tabs ;;
  test-e2e-visual-yazi) cmd_test_e2e_visual_yazi ;;
  test-e2e-visual) cmd_test_e2e_visual ;;
  test-e2e-render-starship) cmd_test_e2e_render_starship ;;
  test-e2e-render-tmux) cmd_test_e2e_render_tmux ;;
  test-e2e-render-layout-switch) cmd_test_e2e_render_layout_switch ;;
  test-e2e-render-theme-switch) cmd_test_e2e_render_theme_switch ;;
  test-e2e-render-yazi) cmd_test_e2e_render_yazi ;;
  test-e2e-render) cmd_test_e2e_render ;;
  test-e2e-doc-captures) cmd_test_e2e_doc_captures ;;
  build-images) cmd_build_images "$@" ;;
  push-images)  cmd_push_images "$@" ;;
  ghcr-prune-source-tags) cmd_ghcr_prune_source_tags "$@" ;;
  ghcr-prune-buildcache-tags) cmd_ghcr_prune_buildcache_tags "$@" ;;
  release-runtime-smoke) cmd_release_runtime_smoke "$@" ;;
  docs-serve)   cmd_docs_serve ;;
  docs-deploy)  cmd_docs_deploy "$@" ;;
  test-visual)  cmd_test_visual ;;
  record-docs)  cmd_record_docs ;;
  sync-processkit) cmd_sync_processkit ;;
  release)      cmd_release "$@" ;;
  release-check-state) cmd_release_check_state "$@" ;;
  release-doctors) cmd_release_doctors ;;
  release-host) cmd_release_host "$@" ;;
  release-finalize-runtime) cmd_release_finalize_runtime "$@" ;;
  start)        cmd_start ;;
  stop)         cmd_stop ;;
  attach)       cmd_attach ;;
  status)       cmd_status ;;
  help|--help|-h) usage ;;
  *) die "Unknown command: '${COMMAND}'. Run './scripts/maintain.sh help' for usage." ;;
esac
