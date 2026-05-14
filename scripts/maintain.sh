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
#   build-images      Build all 10 published images locally
#   release-runtime-smoke <version> Run generated runtime smoke against a release
#   docs-serve        Serve MkDocs locally for preview
#   docs-deploy       Build MkDocs and push HTML to gh-pages
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
  release-runtime-smoke <version>
                           Run host-side generated-runtime smoke and write logs
  docs-serve               Serve MkDocs locally (http://localhost:8000)
  docs-deploy [--dry-run]  Build MkDocs and push to gh-pages branch
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
  ensure_e2e_companion
  info "Pruning SSH companion nested runtime state..."
  ssh -i "${key}" \
      -o StrictHostKeyChecking=no \
      -o UserKnownHostsFile=/dev/null \
      -o ConnectTimeout=5 \
      -o LogLevel=ERROR \
      testuser@aibox-e2e-testrunner \
      'runtime=""; if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then runtime=docker; elif command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; then runtime=podman; fi; test -n "$runtime" || exit 0; ids=$("$runtime" ps -aq 2>/dev/null || true); if [ -n "$ids" ]; then "$runtime" rm -f $ids >/dev/null 2>&1 || true; fi; "$runtime" system prune -af --volumes >/dev/null 2>&1 || true; find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null || sudo find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {} +' \
    || return 1
  ok "SSH companion nested runtime state pruned"
}

cmd_test_e2e() {
  ensure_e2e_companion
  prune_e2e_companion_storage || die "Failed to prune SSH companion nested runtime state"
  info "Running Tier 2 SSH companion E2E tests..."
  local status=0
  (cd "${CLI_DIR}" && cargo test --features e2e --test e2e -- --test-threads=1) \
    || status=$?
  prune_e2e_companion_storage || warn "Post-suite SSH companion prune failed"
  [[ "${status}" -eq 0 ]] || die "Tier 2 SSH companion E2E tests failed"
  ok "Tier 2 SSH companion E2E tests passed"
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

image_source_tag() {
  local flavor="$1" source_sha="$2"
  printf '%s:%s-source-%s' "${IMAGE_REGISTRY}" "${flavor}" "${source_sha}"
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
    info "Building ${flavor} image..."
    local latest="${IMAGE_REGISTRY}:${flavor}-latest"
    local build_cache_ref="${IMAGE_REGISTRY}:${flavor}-buildcache"
    local source_sha
    source_sha="$(image_source_sha "${flavor}")"
    local build_version="${release_version:-dev}"
    if [[ -n "${no_cache}" ]]; then
      "${build_env[@]}" ${RUNTIME_BIN} build --no-cache \
        --build-arg BUILDKIT_INLINE_CACHE=1 \
        --build-arg "AIBOX_IMAGE_SOURCE_SHA=${source_sha}" \
        --build-arg "AIBOX_IMAGE_BUILD_VERSION=${build_version}" \
        -t "${latest}" \
        -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
        "${PROJECT_ROOT}/images/${flavor}/"
    else
      ${RUNTIME_BIN} pull "${latest}" >/dev/null 2>&1 \
        || warn "Could not pull ${latest} as a remote build cache seed"
      if ${RUNTIME_BIN} buildx version >/dev/null 2>&1; then
        ${RUNTIME_BIN} buildx build --load \
          --build-arg BUILDKIT_INLINE_CACHE=1 \
          --build-arg "AIBOX_IMAGE_SOURCE_SHA=${source_sha}" \
          --build-arg "AIBOX_IMAGE_BUILD_VERSION=${build_version}" \
          --cache-from "type=registry,ref=${build_cache_ref}" \
          --cache-from "type=registry,ref=${latest}" \
          --cache-to "type=registry,ref=${build_cache_ref},mode=max,ignore-error=true" \
          -t "${latest}" \
          -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
          "${PROJECT_ROOT}/images/${flavor}/"
      else
        "${build_env[@]}" ${RUNTIME_BIN} build \
          --build-arg BUILDKIT_INLINE_CACHE=1 \
          --build-arg "AIBOX_IMAGE_SOURCE_SHA=${source_sha}" \
          --build-arg "AIBOX_IMAGE_BUILD_VERSION=${build_version}" \
          --cache-from "${latest}" \
          -t "${latest}" \
          -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
          "${PROJECT_ROOT}/images/${flavor}/"
      fi
    fi
    ok "Built ${latest}"
  done

  ok "All images built"
}

cmd_push_images() {
  _require_runtime
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh push-images <version>  (e.g. 0.2.0)"

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "Version must be semver: X.Y.Z (got: ${version})"
  fi

  ensure_ghcr_login

  local flavors=("base-debian")

  # Verify all latest images exist and create versioned tags
  for flavor in "${flavors[@]}"; do
    local latest="${IMAGE_REGISTRY}:${flavor}-latest"
    local versioned="${IMAGE_REGISTRY}:${flavor}-v${version}"
    local source_sha source_tag
    source_sha="$(image_source_sha "${flavor}")"
    source_tag="$(image_source_tag "${flavor}" "${source_sha}")"
    if ! ${RUNTIME_BIN} image exists "${latest}" 2>/dev/null && \
       ! ${RUNTIME_BIN} inspect "${latest}" &>/dev/null; then
      die "Image ${latest} not found locally. Run 'build-images' first."
    fi
    ${RUNTIME_BIN} tag "${latest}" "${versioned}"
    ${RUNTIME_BIN} tag "${latest}" "${source_tag}"
  done

  ok "All images found and tagged for v${version}"

  # Push versioned and latest tags
  for flavor in "${flavors[@]}"; do
    local versioned="${IMAGE_REGISTRY}:${flavor}-v${version}"
    local latest="${IMAGE_REGISTRY}:${flavor}-latest"
    local source_sha source_tag
    source_sha="$(image_source_sha "${flavor}")"
    source_tag="$(image_source_tag "${flavor}" "${source_sha}")"

    info "Pushing ${flavor}..."
    ${RUNTIME_BIN} push "${versioned}" || die "Failed to push ${versioned}"
    ${RUNTIME_BIN} push "${latest}" || die "Failed to push ${latest}"
    ${RUNTIME_BIN} push "${source_tag}" || die "Failed to push ${source_tag}"
    ok "Pushed ${flavor}-v${version} + ${flavor}-latest + source cache marker"
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
    local latest="${IMAGE_REGISTRY}:${flavor}-latest"
    local versioned="${IMAGE_REGISTRY}:${flavor}-v${version}"
    local current_sha source_tag
    current_sha="$(image_source_sha "${flavor}")"
    source_tag="$(image_source_tag "${flavor}" "${current_sha}")"

    if ${RUNTIME_BIN} buildx version >/dev/null 2>&1 \
      && ${RUNTIME_BIN} buildx imagetools inspect "${source_tag}" >/dev/null 2>&1; then
      info "${flavor} source unchanged (${current_sha}); retagging existing GHCR manifest"
      ${RUNTIME_BIN} buildx imagetools create \
        -t "${versioned}" \
        -t "${latest}" \
        "${source_tag}" \
        || die "Failed to retag ${latest} as ${versioned}"
      ok "Retagged ${source_tag} as ${versioned} and ${latest} without rebuilding layers"
    else
      all_retagged=false
      if ! ${RUNTIME_BIN} buildx version >/dev/null 2>&1; then
        warn "buildx is unavailable; rebuilding ${flavor} instead of retagging by source hash"
      else
        info "${flavor} source hash ${current_sha} has no GHCR marker tag yet; rebuilding once to seed it"
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

# Post-publish guard: confirm every release-image tag is actually live in
# GHCR before declaring success. v0.26.0's host-side release passed all of
# build → upload → retag → smoke locally but landed in GHCR with the source
# marker present and the versioned + latest tags missing, leaving downstream
# `aibox apply` runs resolving 'latest' → v0.25.12 forever. The earlier code
# trusted `buildx imagetools create` and `${RUNTIME_BIN} push` exit codes;
# this verifier re-asserts state-of-the-world afterwards.
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
    local versioned="${IMAGE_REGISTRY}:${flavor}-v${version}"
    local latest="${IMAGE_REGISTRY}:${flavor}-latest"
    local versioned_digest latest_digest
    versioned_digest="$(${probe} buildx imagetools inspect --raw "${versioned}" 2>/dev/null | sha256sum 2>/dev/null | awk '{print $1}')"
    latest_digest="$(${probe} buildx imagetools inspect --raw "${latest}" 2>/dev/null | sha256sum 2>/dev/null | awk '{print $1}')"
    if [[ -z "${versioned_digest}" ]]; then
      missing+=("${versioned}")
    fi
    if [[ -z "${latest_digest}" ]]; then
      missing+=("${latest}")
    fi
    if [[ -n "${versioned_digest}" && -n "${latest_digest}" \
       && "${versioned_digest}" != "${latest_digest}" ]]; then
      warn "${versioned} and ${latest} resolve to different manifests in GHCR — 'latest' likely stale"
      missing+=("${latest} (digest mismatch with ${versioned})")
    fi
    if [[ -n "${versioned_digest}" ]]; then
      ok "Verified ${versioned} is live in GHCR"
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

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "Version must be semver: X.Y.Z (got: ${version})"
  fi

  "${SCRIPT_DIR}/release-runtime-smoke.sh" "${version}" \
    || die "Release runtime smoke failed. See dist/release-smoke/v${version}/ for logs."
}

cmd_docs_serve() {
  cd "${PROJECT_ROOT}/docs-site"
  info "Serving docs with Docusaurus at http://localhost:3000 ..."
  npx docusaurus start --host 0.0.0.0
}

cmd_docs_deploy() {
  local dry_run=false
  # Bug (b): declare tmpdir early so the EXIT trap below never sees an unbound
  # variable under set -u if the function exits before reaching mktemp -d.
  local tmpdir=""
  [[ "${1:-}" == "--dry-run" ]] && dry_run=true

  command -v npx &>/dev/null    || die "npx not found. Install Node.js."
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

  cd "${PROJECT_ROOT}/docs-site"
  info "Building docs with Docusaurus..."
  npx docusaurus build
  ok "Site built in docs-site/build/"

  if [[ "${dry_run}" == "true" ]]; then
    warn "Dry run — site is in site/"
    return 0
  fi

  tmpdir=$(mktemp -d)
  # Bug (b): use ${tmpdir:-} so trap is safe even if mktemp never ran (set -u).
  trap '[[ -n "${tmpdir:-}" ]] && rm -rf "${tmpdir}"' EXIT

  cp -r build/* "${tmpdir}/"
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
  # username and aborts non-interactively. Inject the gh CLI's OAuth token
  # into the URL when (a) the URL is HTTPS-on-github.com and (b) `gh auth
  # token` succeeds. SSH and other hosts pass through unchanged.
  local push_url="${remote_url}"
  if [[ "${remote_url}" == https://github.com/* ]] && command -v gh &>/dev/null; then
    local gh_token
    if gh_token=$(gh auth token 2>/dev/null) && [[ -n "${gh_token}" ]]; then
      push_url="https://x-access-token:${gh_token}@github.com/${repo_slug}.git"
    fi
  fi
  git push --force "${push_url}" gh-pages:gh-pages
  cd "${PROJECT_ROOT}"
  ok "Deployed to gh-pages branch"

  # Configure GitHub Pages if gh is available. Probe-first: only POST when
  # Pages doesn't exist yet. Avoids the spurious "Could not configure"
  # warning we used to print on every release because PUT 422s when Pages
  # is already managed by an Actions workflow or already pinned to gh-pages
  # with identical settings.
  if command -v gh &>/dev/null && [[ -n "${repo_slug}" ]]; then
    if gh api "repos/${repo_slug}/pages" --silent 2>/dev/null; then
      ok "GitHub Pages already configured (skipping)"
    elif gh api --method POST "repos/${repo_slug}/pages" \
           -f "source[branch]=gh-pages" -f "source[path]=/" \
           --silent 2>/dev/null; then
      ok "GitHub Pages configured"
    else
      warn "Pages auto-config skipped (likely managed by an Actions workflow — non-fatal)"
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
  push-main       Push main to origin
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
  built_archives=()
  for target in aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu; do
    archive="${DIST_DIR}/aibox-v${version}-${target}.tar.gz"
    checksum="${archive}.sha256"
    [[ -f "${archive}" ]] || die "Missing release archive ${archive}; run --steps build first."
    [[ -f "${checksum}" ]] || die "Missing release checksum ${checksum}; run --steps build first."
    built_archives+=("${archive}" "${checksum}")
  done
}

cmd_release() {
  local version="${1:-}"
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
  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "Version must be semver: X.Y.Z (got: ${version})"
  fi

  local tag="v${version}"
  local release_steps=()
  local release_step_tokens=()
  local built_archives=()
  release_parse_steps "${steps_spec}"
  release_apply_skip_steps "${skip_spec}"

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
        || die "failed to commit Cargo.toml/Cargo.lock bump"
      ok "Cargo.toml bumped and committed"
    else
      ok "Cargo.toml already at ${version}"
    fi
  fi

  if release_step_requested test; then
    info "Running tests..."
    cmd_test
    # Starship Tier 3 (vt100 rendered-color) runs locally without a companion
    # and only adds ~6s. Run it alongside the regular test step so we always
    # catch a regression where the generated starship.toml is silently
    # ignored or rendered with the wrong palette. Skip gracefully via
    # AIBOX_RELEASE_SKIP_RENDER_LOCAL=1 if the host doesn't have `starship`.
    case "${AIBOX_RELEASE_SKIP_RENDER_LOCAL:-}" in
      1|true|yes)
        warn "Skipping Tier 3 local Starship rendered-color tests because AIBOX_RELEASE_SKIP_RENDER_LOCAL=${AIBOX_RELEASE_SKIP_RENDER_LOCAL}."
        ;;
      *)
        cmd_test_e2e_render_starship
        ;;
    esac
  fi

  if release_step_requested e2e; then
    case "${AIBOX_RELEASE_SKIP_COMPANION_E2E:-}" in
      1|true|yes)
        warn "Skipping Tier 2 SSH companion E2E during release because AIBOX_RELEASE_SKIP_COMPANION_E2E=${AIBOX_RELEASE_SKIP_COMPANION_E2E}. Re-run ./scripts/maintain.sh test-e2e after rebuilding the companion."
        ;;
      *)
        cmd_test_e2e
        ;;
    esac
  fi

  if release_step_requested visual; then
    case "${AIBOX_RELEASE_VISUAL_E2E:-skip}" in
      skip|"")
        warn "Skipping opt-in visual E2E during release. The release agent must justify this in notes or handover, or run AIBOX_RELEASE_VISUAL_E2E=<status|tabs|yazi|render|full|docs>."
        ;;
      status)
        cmd_test_e2e_visual_status
        ;;
      tabs|tools)
        cmd_test_e2e_visual_tabs
        ;;
      yazi)
        cmd_test_e2e_visual_yazi
        ;;
      render)
        # Tier 3 vt100 cell-color suite (tmux + yazi). Starship Tier 3
        # already ran during the `test` step; this adds the companion tiers.
        cmd_test_e2e_render_tmux
        cmd_test_e2e_render_yazi
        ;;
      full)
        cmd_test_e2e_visual
        # `full` extends to Tier 3 companion tiers too — release gating
        # should verify every themed surface actually paints palette colors.
        cmd_test_e2e_render_tmux
        cmd_test_e2e_render_yazi
        ;;
      docs|captures)
        cmd_test_e2e_doc_captures
        ;;
      *)
        die "Unknown AIBOX_RELEASE_VISUAL_E2E=${AIBOX_RELEASE_VISUAL_E2E}; expected skip, status, tabs, yazi, render, full, or docs"
        ;;
    esac
  fi

  if release_step_requested audit; then
    info "Running cargo audit..."
    command -v cargo-audit &>/dev/null \
      || (cd "${CLI_DIR}" && cargo install cargo-audit --quiet)
    local audit_db="${TMPDIR:-/tmp}/aibox-cargo-advisory-db"
    mkdir -p "${audit_db}"
    (cd "${CLI_DIR}" && cargo audit --db "${audit_db}") \
      || die "cargo audit found advisories — resolve before releasing"
    ok "Audit clean"
  fi

  if release_step_requested build-linux; then
    info "Building CLI (release mode) for all Linux targets..."
    mkdir -p "${DIST_DIR}"

    local linux_targets=("aarch64-unknown-linux-gnu" "x86_64-unknown-linux-gnu")

    for target in "${linux_targets[@]}"; do
      info "  → ${target}"
      (cd "${CLI_DIR}" && \
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
        cargo build --release --target "${target}") \
        || die "cargo build failed for ${target}"
      local binary_name="aibox-v${version}-${target}"
      cp "${CLI_DIR}/target/${target}/release/aibox" "${DIST_DIR}/${binary_name}"
      tar -czf "${DIST_DIR}/${binary_name}.tar.gz" -C "${DIST_DIR}" "${binary_name}"
      rm "${DIST_DIR}/${binary_name}"
      sha256sum "${DIST_DIR}/${binary_name}.tar.gz" | awk '{print $1}' > "${DIST_DIR}/${binary_name}.tar.gz.sha256"
      built_archives+=("${DIST_DIR}/${binary_name}.tar.gz")
      built_archives+=("${DIST_DIR}/${binary_name}.tar.gz.sha256")
      ok "Built ${binary_name}.tar.gz"
    done
  fi

  if release_step_requested version-smoke; then
    info "Verifying 'aibox --version' matches ${version}..."
    local host_binary="${CLI_DIR}/target/release/aibox"
    (cd "${CLI_DIR}" && cargo build --release --quiet) \
      || die "native cargo build failed (needed for --version smoke test)"
    local reported
    reported=$("${host_binary}" --version | awk '{print $NF}')
    if [[ "${reported}" != "${version}" ]]; then
      die "aibox --version reports '${reported}' but the tag being cut is '${version}'. Fix Cargo.toml before retrying."
    fi
    ok "aibox --version = ${reported} (matches tag)"
  fi

  if release_step_requested push-main; then
    info "Pushing main to origin (version-bump commit)..."
    git push origin main
    ok "main pushed to origin"
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
      "${built_archives[@]}"
    ok "GitHub release ${tag} created with Linux binaries"
  fi

  if release_step_requested docs; then
    info "Deploying documentation..."
    cmd_docs_deploy
    ok "Documentation deployed"
  fi

  if release_step_requested prompt; then
    local prompt_file="${DIST_DIR}/RELEASE-PROMPT.md"
    {
      echo "# Host-side steps for aibox ${tag}"
      echo ""
      echo "Linux binaries are already uploaded to the GitHub release."
      echo "Run the following on the macOS host to complete the release:"
      echo ""
      echo "\`\`\`bash"
      echo "./scripts/maintain.sh release-host ${version}"
      echo "\`\`\`"
      echo ""
      echo "This will:"
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

cmd_release_finalize_runtime() {
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh release-finalize-runtime <version>  (e.g. 0.10.2)"

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "Version must be semver: X.Y.Z (got: ${version})"
  fi

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
      git commit -m "chore: refresh generated runtime for v${version}"
      git push origin main
      ok "Generated runtime surfaces committed and pushed for v${version}."
    fi
  )
}

# ── Host-side release (run on macOS after container-side `release`) ──────────

cmd_release_host() {
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh release-host <version>  (e.g. 0.10.2)"

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "Version must be semver: X.Y.Z (got: ${version})"
  fi

  local tag="v${version}"

  # ── Step 1: Build macOS binaries ──────────────────────────────────────────
  info "Building macOS binaries..."
  "${SCRIPT_DIR}/build-macos.sh" "${version}"

  # ── Step 2: Upload macOS binaries to existing GitHub release ──────────────
  info "Uploading macOS binaries to GitHub release ${tag}..."
  if ! gh release view "${tag}" --repo "${GITHUB_REPO}" &>/dev/null; then
    die "GitHub release ${tag} not found. Run 'release' in the container first."
  fi
  gh release upload "${tag}" "${DIST_DIR}"/aibox-v${version}-*-apple-darwin.tar.gz "${DIST_DIR}"/aibox-v${version}-*-apple-darwin.tar.gz.sha256 \
    --repo "${GITHUB_REPO}" \
    || warn "Upload failed — binaries may already be attached"
  ok "macOS binaries and checksums uploaded to ${tag}"

  # ── Step 3: Publish container images ─────────────────────────────────────
  info "Publishing container images..."
  cmd_publish_images_for_release "${version}"

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
