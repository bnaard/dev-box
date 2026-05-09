#!/usr/bin/env bash
# release-runtime-smoke.sh -- host-side release smoke for generated runtime UX.
#
# Runs TUI probes inside the release-smoke container, captures raw PTY output
# to files, and never streams raw terminal control sequences to the host
# terminal.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CLI_DIR="${PROJECT_ROOT}/cli"
DIST_DIR="${PROJECT_ROOT}/dist"

version="${1:-}"
if [[ -z "${version}" ]]; then
  echo "Usage: ./scripts/release-runtime-smoke.sh <version>" >&2
  exit 2
fi
if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Version must be semver: X.Y.Z (got: ${version})" >&2
  exit 2
fi

run_id="$(date +%Y%m%d-%H%M%S)"
log_dir="${AIBOX_RELEASE_SMOKE_DIR:-${DIST_DIR}/release-smoke/v${version}/${run_id}}"
project_dir="${AIBOX_RELEASE_SMOKE_PROJECT_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/aibox-release-smoke-v${version}.XXXXXX")}"
container_name="${AIBOX_RELEASE_SMOKE_CONTAINER:-aibox-release-smoke-${version//./-}}"
tmux_status="${AIBOX_RELEASE_SMOKE_TMUX_STATUS:-extended}"
smoke_tier="${AIBOX_RELEASE_SMOKE_TIER:-addons}"
probe_script="${log_dir}/container-probe.sh"
run_log="${log_dir}/run.log"

mkdir -p "${log_dir}"
exec > >(tee -a "${run_log}") 2>&1

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

case "${tmux_status}" in
  shell|sidecar|native|enabled|powerline) tmux_status="extended" ;;
  hidden) tmux_status="disabled" ;;
  extended|plain|disabled) ;;
  *) die "AIBOX_RELEASE_SMOKE_TMUX_STATUS must be extended, plain, or disabled (legacy aliases: powerline, enabled, shell, sidecar, native, hidden; got: ${tmux_status})" ;;
esac

case "${smoke_tier}" in
  minimal) smoke_git_ui=0; smoke_preview=0 ;;
  addons)  smoke_git_ui=1; smoke_preview=0 ;;
  full)    smoke_git_ui=1; smoke_preview=1 ;;
  *) die "AIBOX_RELEASE_SMOKE_TIER must be minimal, addons, or full (got: ${smoke_tier})" ;;
esac

if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  runtime_bin="docker"
elif command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; then
  runtime_bin="podman"
else
  die "Neither docker nor podman is available."
fi

compose() {
  if [[ "${runtime_bin}" == "docker" ]]; then
    docker compose "$@"
  else
    podman compose "$@"
  fi
}

runtime() {
  "${runtime_bin}" "$@"
}

host_target=""
case "$(uname -s):$(uname -m)" in
  Darwin:arm64) host_target="aarch64-apple-darwin" ;;
  Darwin:x86_64) host_target="x86_64-apple-darwin" ;;
esac

if [[ -n "${AIBOX_RELEASE_SMOKE_BIN:-}" ]]; then
  aibox_bin="${AIBOX_RELEASE_SMOKE_BIN}"
elif [[ -n "${host_target}" && -x "${CLI_DIR}/target/${host_target}/release/aibox" ]]; then
  aibox_bin="${CLI_DIR}/target/${host_target}/release/aibox"
elif [[ -x "${CLI_DIR}/target/debug/aibox" ]]; then
  aibox_bin="${CLI_DIR}/target/debug/aibox"
else
  die "Could not find a built aibox binary. Set AIBOX_RELEASE_SMOKE_BIN."
fi

copy_if_exists() {
  local src="$1"
  local dest="$2"
  if [[ -e "${src}" ]]; then
    mkdir -p "$(dirname "${dest}")"
    cp -R "${src}" "${dest}"
  fi
}

compose_file() {
  printf '%s/.devcontainer/docker-compose.yml' "${project_dir}"
}

collect_artifacts() {
  local status=$?
  trap - EXIT INT TERM
  set +e
  stty sane >/dev/null 2>&1 || true

  info "Collecting release smoke artifacts..."
  {
    echo "version=${version}"
    echo "run_id=${run_id}"
    echo "project_dir=${project_dir}"
    echo "log_dir=${log_dir}"
    echo "runtime=${runtime_bin}"
    echo "aibox_bin=${aibox_bin}"
    echo "container_name=${container_name}"
    echo "tmux_status=${tmux_status}"
    echo "smoke_tier=${smoke_tier}"
    echo "exit_status=${status}"
  } > "${log_dir}/metadata.env"

  copy_if_exists "${project_dir}/aibox.toml" "${log_dir}/project/aibox.toml"
  copy_if_exists "${project_dir}/aibox.lock" "${log_dir}/project/aibox.lock"
  copy_if_exists "${project_dir}/.devcontainer/Dockerfile" "${log_dir}/project/.devcontainer/Dockerfile"
  copy_if_exists "${project_dir}/.devcontainer/docker-compose.yml" "${log_dir}/project/.devcontainer/docker-compose.yml"
  copy_if_exists "${project_dir}/.devcontainer/devcontainer.json" "${log_dir}/project/.devcontainer/devcontainer.json"
  copy_if_exists "${project_dir}/.aibox-home/.config/tmux/tmux.conf" "${log_dir}/project/.aibox-home/.config/tmux/tmux.conf"
  copy_if_exists "${project_dir}/.aibox-home/.config/tmux/status.conf" "${log_dir}/project/.aibox-home/.config/tmux/status.conf"
  copy_if_exists "${project_dir}/.aibox-home/.config/tmux/layouts/ai.sh" "${log_dir}/project/.aibox-home/.config/tmux/layouts/ai.sh"
  copy_if_exists "${project_dir}/.aibox-home/.config/tmux/aibox-session.sh" "${log_dir}/project/.aibox-home/.config/tmux/aibox-session.sh"
  copy_if_exists "${project_dir}/.aibox-home/.config/yazi/yazi.toml" "${log_dir}/project/.aibox-home/.config/yazi/yazi.toml"
  copy_if_exists "${project_dir}/.aibox-home/.config/yazi/theme.toml" "${log_dir}/project/.aibox-home/.config/yazi/theme.toml"
  copy_if_exists "${project_dir}/.aibox-home/.config/yazi/keymap.toml" "${log_dir}/project/.aibox-home/.config/yazi/keymap.toml"

  runtime ps -a > "${log_dir}/runtime-ps.txt" 2>&1
  runtime inspect "${container_name}" > "${log_dir}/container-inspect.json" 2>&1
  if [[ -f "$(compose_file)" ]]; then
    compose -f "$(compose_file)" logs --no-color > "${log_dir}/compose.log" 2>&1
  fi

  runtime cp "${container_name}:/tmp/aibox-yazi-debug.txt" "${log_dir}/yazi-debug.txt" >/dev/null 2>&1
  runtime cp "${container_name}:/tmp/aibox-status-plugin.json" "${log_dir}/aibox-status-plugin.json" >/dev/null 2>&1
  runtime cp "${container_name}:/tmp/aibox-tmux.typescript" "${log_dir}/tmux.typescript" >/dev/null 2>&1
  runtime cp "${container_name}:/tmp/aibox-tmux-pty.log" "${log_dir}/tmux-pty.log" >/dev/null 2>&1
  runtime cp "${container_name}:/tmp/aibox-tmux-generated-state.txt" "${log_dir}/tmux-generated-state.txt" >/dev/null 2>&1
  runtime cp "${container_name}:/workspace/.aibox/diagnostics" "${log_dir}/diagnostics" >/dev/null 2>&1
  runtime exec --user aibox "${container_name}" bash -lc \
    "for socket in \"\$HOME/.tmux/aibox.sock\" \"\$HOME/.tmux/aibox-smoke.sock\"; do echo \"--- socket: \${socket} ---\"; tmux -S \"\${socket}\" list-sessions 2>&1 || true; tmux -S \"\${socket}\" list-windows -a 2>&1 || true; tmux -S \"\${socket}\" list-panes -a 2>&1 || true; done" \
    > "${log_dir}/tmux-state.txt" 2>&1

  if [[ "${AIBOX_RELEASE_SMOKE_KEEP:-0}" == "1" || "${status}" -ne 0 ]]; then
    warn "Keeping smoke project/container for inspection: ${project_dir}"
  elif [[ -f "$(compose_file)" ]]; then
    compose -f "$(compose_file)" down -v > "${log_dir}/compose-down.log" 2>&1
    rm -rf "${project_dir}"
  fi

  if [[ "${status}" -eq 0 ]]; then
    ok "Release runtime smoke passed. Logs: ${log_dir}"
  else
    warn "Release runtime smoke failed. Logs: ${log_dir}"
  fi

  exit "${status}"
}
trap collect_artifacts EXIT INT TERM

run() {
  echo "+ $*"
  "$@"
}

info "Release runtime smoke for v${version}"
echo "Project:       ${project_dir}"
echo "Logs:          ${log_dir}"
echo "Runtime:       ${runtime_bin}"
echo "aibox:         ${aibox_bin}"
echo "tmux status:   ${tmux_status}"
echo "Smoke tier:    ${smoke_tier}"

mkdir -p "${project_dir}"
cd "${project_dir}"

run git init -q
init_args=(
  init "${container_name}"
  --base debian
  --profile human-dev
  --theme tokyo-night
  --prompt arrow
  --tmux-status "${tmux_status}"
  --processkit-version latest
  --no-container
)
if [[ "${smoke_git_ui}" == "1" ]]; then
  init_args+=(--addon git-ui)
fi
if [[ "${smoke_preview}" == "1" ]]; then
  init_args+=(--addon preview-archive --addon preview-enhanced)
fi
run env AIBOX_ADDONS_DIR="${PROJECT_ROOT}/addons" "${aibox_bin}" "${init_args[@]}" < /dev/null

cat > aibox.toml <<EOF
apiVersion = "aibox.projectious.work/v1"
kind = "Workspace"

[aibox]
project_name = "${container_name}"
profile = "human-dev"

[container]
name = "${container_name}"
hostname = "${container_name}"
user = "aibox"

[container.image]
release_version = "${version}"
base = "debian"

[container.paths]
devcontainer_json = ".devcontainer/devcontainer.json"
docker_compose = ".devcontainer/docker-compose.yml"
docker_compose_override = ".devcontainer/docker-compose.override.yml"
dockerfile = ".devcontainer/Dockerfile"
dockerfile_local = ".devcontainer/Dockerfile.local"
local_env = ".aibox-local.env"

[skills]
enabled = []
disabled = []

[ai]
model_providers = []

[ai.harness.claude]
enabled = false
install = false

[ai.agents]
canonical = "AGENTS.md"
provider_mode = "pointer"

[ai.mcp.gateway]
mode = "auto"
lazy_catalog = false
host = "127.0.0.1"
port = 8765
path = "/mcp"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
version = "latest"
src_path = "src"

[processkit.context]
schema_version = "1.0.0"

[customization]
theme = "tokyo-night"
mode = "auto"
prompt = "arrow"
layout = "ai"

[customization.tmux.status]
mode = "${tmux_status}"
EOF

if [[ "${smoke_git_ui}" == "1" ]]; then
  cat >> aibox.toml <<'EOF'

[addons.git-ui.tools]
gh = {}
lazygit = {}
EOF
fi
if [[ "${smoke_preview}" == "1" ]]; then
  cat >> aibox.toml <<'EOF'

[addons.preview-archive.tools]

[addons.preview-enhanced.tools]
ffmpeg = {}
imagemagick = {}
ghostscript = {}
EOF
fi

apply_args=(apply --standardize-config)
if [[ "${AIBOX_RELEASE_SMOKE_NO_CACHE:-0}" =~ ^(1|true|yes)$ || "${smoke_tier}" == "full" ]]; then
  apply_args+=(--no-cache)
fi
run env AIBOX_ADDONS_DIR="${PROJECT_ROOT}/addons" "${aibox_bin}" "${apply_args[@]}"

attach_smoke_log="${log_dir}/up-forget-tmux-state.log"
info "Running attach smoke: aibox up --forget-tmux-state"
if command -v timeout >/dev/null 2>&1; then
  if env AIBOX_ADDONS_DIR="${PROJECT_ROOT}/addons" timeout 25s \
    "${aibox_bin}" up --forget-tmux-state >"${attach_smoke_log}" 2>&1 < /dev/null; then
    ok "Attach smoke exited cleanly"
  else
    code=$?
    if [[ "${code}" -eq 124 ]]; then
      ok "Attach smoke reached timeout while attached (expected for interactive tmux)"
    elif grep -q "stdin is not a terminal" "${attach_smoke_log}"; then
      ok "Attach smoke reached non-interactive attach boundary"
    else
      warn "Attach smoke failed with ${code}; see ${attach_smoke_log}"
      exit "${code}"
    fi
  fi
else
  warn "timeout command missing on host; running attach smoke without timeout"
  set +e
  env AIBOX_ADDONS_DIR="${PROJECT_ROOT}/addons" \
    "${aibox_bin}" up --forget-tmux-state >"${attach_smoke_log}" 2>&1 < /dev/null
  code=$?
  set -e
  if [[ "${code}" -ne 0 ]]; then
    if grep -q "stdin is not a terminal" "${attach_smoke_log}"; then
      ok "Attach smoke reached non-interactive attach boundary"
    else
      warn "Attach smoke failed with ${code}; see ${attach_smoke_log}"
      exit "${code}"
    fi
  fi
fi

if grep -q "can't find pane: 1" "${attach_smoke_log}"; then
  warn "Attach smoke reproduced tmux pane-index startup regression; see ${attach_smoke_log}"
  exit 1
fi

cat > "${probe_script}" <<'EOF'
#!/usr/bin/env bash
set -u
tmux_status="__AIBOX_RELEASE_SMOKE_TMUX_STATUS__"
smoke_git_ui="__AIBOX_RELEASE_SMOKE_GIT_UI__"

fail=0
section() { printf '\n== %s ==\n' "$*"; }
soft_run() {
  echo "+ $*"
  "$@"
  local code=$?
  echo "exit=${code}"
  return "${code}"
}

cd /workspace || exit 10

section versions
soft_run tmux -V || fail=1
soft_run yazi --version || fail=1
if [[ "${smoke_git_ui}" == "1" ]]; then
  soft_run lazygit --version || fail=1
else
  echo "lazygit version probe skipped for minimal smoke tier"
fi
soft_run vim --version | sed -n '1,4p' || true

section yazi-config
nl -ba "$HOME/.config/yazi/yazi.toml" | sed -n '1,140p'
nl -ba "$HOME/.config/yazi/theme.toml" | sed -n '1,140p'
if yazi --debug >/tmp/aibox-yazi-debug.txt 2>&1; then
  sed -n '1,140p' /tmp/aibox-yazi-debug.txt
else
  code=$?
  sed -n '1,180p' /tmp/aibox-yazi-debug.txt
  echo "yazi --debug failed with ${code}"
  fail=1
fi

if [[ "${smoke_git_ui}" == "1" ]]; then
  section lazygit-state
  state_dir="${XDG_STATE_HOME:-$HOME/.local/state}"
  mkdir -p "$HOME/.config/lazygit"
  if mkdir -p "${state_dir}/lazygit" 2>/tmp/aibox-lazygit-state-mkdir.err; then
    ls -ld "$HOME" "$HOME/.config/lazygit" "${state_dir}" "${state_dir}/lazygit" 2>&1 || fail=1
  else
    echo "warning: lazygit state directory is not writable in this release:"
    cat /tmp/aibox-lazygit-state-mkdir.err
    ls -ld "$HOME" "$HOME/.config/lazygit" "${state_dir}" 2>&1 || true
  fi
else
  section lazygit-state
  echo "skipped for minimal smoke tier"
fi

section status-helper
if command -v aibox-status >/dev/null 2>&1; then
  if aibox-status --plugin-json >/tmp/aibox-status-plugin.json 2>&1; then
    cat /tmp/aibox-status-plugin.json
    jq -e '.plain and .memory_current and .processes' /tmp/aibox-status-plugin.json >/dev/null || fail=1
  else
    cat /tmp/aibox-status-plugin.json
    fail=1
  fi
else
  echo "aibox-status helper not present in this release"
  if [[ "${tmux_status}" == "extended" ]]; then
    fail=1
  fi
fi

section tmux-status-contract
if [[ "${tmux_status}" != "disabled" ]]; then
  for path in "$HOME/.config/tmux/tmux.conf"; do
    if [[ ! -r "${path}" ]]; then
      echo "missing readable tmux status/config file: ${path}"
      fail=1
      continue
    fi
    sed -n '1,160p' "${path}"
  done

  if ! grep -RInE 'aibox-status|status-left|status-right|@aibox' "$HOME/.config/tmux" >/tmp/aibox-tmux-status-config.txt 2>&1; then
    echo "tmux config does not reference aibox status integration"
    fail=1
  else
    cat /tmp/aibox-tmux-status-config.txt
  fi

  # Persistence policy guardrail: plugins may be installed in the image, but
  # generated runtime defaults must keep session restore/save disabled until
  # the owner accepts a persistence policy decision.
  if ! grep -Eq "@continuum-restore 'off'" "$HOME/.config/tmux/tmux.conf"; then
    echo "tmux persistence policy regression: @continuum-restore must default to off"
    fail=1
  fi
  if ! grep -Eq "@continuum-save-interval '0'" "$HOME/.config/tmux/tmux.conf"; then
    echo "tmux persistence policy regression: @continuum-save-interval must default to 0"
    fail=1
  fi
  if ! grep -Eq "@resurrect-capture-pane-contents 'off'" "$HOME/.config/tmux/tmux.conf"; then
    echo "tmux persistence policy regression: @resurrect-capture-pane-contents must default to off"
    fail=1
  fi
else
  echo "tmux status contract skipped for tmux_status=${tmux_status}"
fi

section diagnostics-sidecar
if [[ -d /workspace/.aibox/diagnostics ]]; then
  find /workspace/.aibox/diagnostics -maxdepth 2 -type f -print -exec sed -n '1,80p' {} \; || true
else
  echo "diagnostics directory missing; generated sidecar may not have emitted yet"
fi

section tmux-pty
if ! command -v script >/dev/null 2>&1; then
  echo "script command missing; cannot run tmux PTY smoke"
  fail=1
elif ! command -v timeout >/dev/null 2>&1; then
  echo "timeout command missing; cannot run tmux PTY smoke"
  fail=1
else
  layout_script="$HOME/.config/tmux/layouts/ai.sh"
  tmux_socket="$HOME/.tmux/aibox-smoke.sock"
  mkdir -p "$(dirname "${tmux_socket}")"
  if [[ ! -x "${layout_script}" ]]; then
    echo "missing executable generated ai tmux layout: ${layout_script}"
    fail=1
  fi
  ln -sf "$HOME/.config/tmux/tmux.conf" "$HOME/.tmux.conf"
  tmux -S "${tmux_socket}" kill-session -t aibox-smoke >/dev/null 2>&1 || true
  if [[ -x "${layout_script}" ]]; then
    timeout 16s script -q -c "AIBOX_TMUX_SOCKET=\"${tmux_socket}\" AIBOX_TMUX_SESSION=aibox-smoke AIBOX_WORKSPACE=/workspace \"${layout_script}\"" /tmp/aibox-tmux.typescript >/tmp/aibox-tmux-pty.log 2>&1
    code=$?
    {
      echo "--- sessions ---"
      tmux -S "${tmux_socket}" list-sessions 2>&1 || true
      echo "--- windows ---"
      tmux -S "${tmux_socket}" list-windows -t aibox-smoke -F '#I #{window_name} #{window_panes}' 2>&1 || true
      echo "--- panes ---"
      tmux -S "${tmux_socket}" list-panes -t aibox-smoke: -F '#I.#P #{pane_current_command} #{pane_title}' 2>&1 || true
    } >/tmp/aibox-tmux-generated-state.txt
    cat /tmp/aibox-tmux-generated-state.txt
    tmux -S "${tmux_socket}" kill-session -t aibox-smoke >/dev/null 2>&1 || true
    if [[ "${code}" -ne 0 && "${code}" -ne 124 ]]; then
      echo "generated ai tmux PTY smoke failed with ${code}"
      fail=1
    fi
    if [[ ! -s /tmp/aibox-tmux.typescript ]]; then
      echo "tmux PTY transcript is empty"
      fail=1
    fi
    expected_windows="ai shell"
    if [[ "${smoke_git_ui}" == "1" ]]; then
      expected_windows="${expected_windows} git"
    fi
    for expected_window in ${expected_windows}; do
      if ! grep -E "^[0-9]+ ${expected_window} " /tmp/aibox-tmux-generated-state.txt >/tmp/aibox-tmux-layout-windows.txt 2>&1; then
        echo "generated ai layout did not create expected ${expected_window} window"
        fail=1
      fi
    done
    if ! awk '/--- panes ---/{seen=1; next} seen && NF {count++} END {exit count >= 2 ? 0 : 1}' /tmp/aibox-tmux-generated-state.txt; then
      echo "generated ai layout did not create multiple real tmux panes"
      fail=1
    fi
    if [[ "${tmux_status}" != "disabled" ]]; then
      if ! grep -aEi 'Ctrl-g|Prefix|pane|window|tmux|AI|dev|shell|aibox' /tmp/aibox-tmux.typescript >/tmp/aibox-tmux-key-row.txt 2>&1; then
        echo "tmux status/key text was not visible in generated-layout PTY smoke"
        fail=1
      fi
      if ! grep -aE 'AIBOX|MEM .+/unlimited|OOM [0-9]+|PROC [0-9]+ AI [0-9]+|MCP (gateway|granular|none) [0-9]+' /tmp/aibox-tmux.typescript >/tmp/aibox-tmux-runtime-row.txt 2>&1; then
        if [[ "${tmux_status}" == "extended" ]]; then
          echo "tmux runtime status row was not visible in generated-layout PTY smoke"
          fail=1
        else
          echo "warning: tmux runtime status row was not visible in PTY smoke"
        fi
      fi
    fi
  fi
fi

exit "${fail}"
EOF
sed -i.bak "s/__AIBOX_RELEASE_SMOKE_TMUX_STATUS__/${tmux_status}/g" "${probe_script}"
sed -i.bak "s/__AIBOX_RELEASE_SMOKE_GIT_UI__/${smoke_git_ui}/g" "${probe_script}"

run runtime cp "${probe_script}" "${container_name}:/tmp/aibox-release-smoke-probe.sh"
run runtime exec --user root "${container_name}" chmod 0755 /tmp/aibox-release-smoke-probe.sh
info "Running container probe; raw TUI output is captured under ${log_dir}, not streamed."
if runtime exec --user aibox "${container_name}" bash /tmp/aibox-release-smoke-probe.sh \
  > "${log_dir}/container-probe.log" 2>&1; then
  ok "Container probe passed"
else
  code=$?
  warn "Container probe failed with ${code}. See ${log_dir}/container-probe.log"
  exit "${code}"
fi
