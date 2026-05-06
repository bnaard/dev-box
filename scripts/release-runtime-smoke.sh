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
zellij_status="${AIBOX_RELEASE_SMOKE_ZELLIJ_STATUS:-shell}"
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

case "${zellij_status}" in
  shell|native|hidden) ;;
  *) die "AIBOX_RELEASE_SMOKE_ZELLIJ_STATUS must be shell, native, or hidden (got: ${zellij_status})" ;;
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
    echo "zellij_status=${zellij_status}"
    echo "exit_status=${status}"
  } > "${log_dir}/metadata.env"

  copy_if_exists "${project_dir}/aibox.toml" "${log_dir}/project/aibox.toml"
  copy_if_exists "${project_dir}/aibox.lock" "${log_dir}/project/aibox.lock"
  copy_if_exists "${project_dir}/.devcontainer/Dockerfile" "${log_dir}/project/.devcontainer/Dockerfile"
  copy_if_exists "${project_dir}/.devcontainer/docker-compose.yml" "${log_dir}/project/.devcontainer/docker-compose.yml"
  copy_if_exists "${project_dir}/.devcontainer/devcontainer.json" "${log_dir}/project/.devcontainer/devcontainer.json"
  copy_if_exists "${project_dir}/.aibox-home/.config/zellij/config.kdl" "${log_dir}/project/.aibox-home/.config/zellij/config.kdl"
  copy_if_exists "${project_dir}/.aibox-home/.config/zellij/layouts/ai.kdl" "${log_dir}/project/.aibox-home/.config/zellij/layouts/ai.kdl"
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
  runtime cp "${container_name}:/tmp/aibox-zellij.typescript" "${log_dir}/zellij.typescript" >/dev/null 2>&1
  runtime cp "${container_name}:/tmp/aibox-zellij-pty.log" "${log_dir}/zellij-pty.log" >/dev/null 2>&1
  runtime exec --user aibox "${container_name}" bash -lc \
    "find /tmp -path '*/zellij-log/zellij.log' -type f -print -exec tail -n 240 {} \\;" \
    > "${log_dir}/zellij-logs.txt" 2>&1

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
echo "Zellij status: ${zellij_status}"

mkdir -p "${project_dir}"
cd "${project_dir}"

run git init -q
run env AIBOX_ADDONS_DIR="${PROJECT_ROOT}/addons" "${aibox_bin}" init "${container_name}" \
  --base debian \
  --profile human-dev \
  --theme tokyo-night \
  --prompt arrow \
  --zellij-status "${zellij_status}" \
  --addon git-ui \
  --addon preview-archive \
  --addon preview-enhanced \
  --processkit-version latest \
  --no-container \
  < /dev/null

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

[addons.git-ui.tools]
gh = {}
lazygit = {}

[addons.preview-archive.tools]

[addons.preview-enhanced.tools]
ffmpeg = {}
imagemagick = {}
ghostscript = {}

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

[customization.zellij_status]
mode = "${zellij_status}"
EOF

run env AIBOX_ADDONS_DIR="${PROJECT_ROOT}/addons" "${aibox_bin}" apply --no-cache --standardize-config
run compose -f "$(compose_file)" up -d "${container_name}"

cat > "${probe_script}" <<'EOF'
#!/usr/bin/env bash
set -u

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
soft_run zellij --version || fail=1
soft_run yazi --version || fail=1
soft_run lazygit --version || fail=1
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
fi

section zellij-pty
if ! command -v script >/dev/null 2>&1; then
  echo "script command missing; cannot run zellij PTY smoke"
  fail=1
elif ! command -v timeout >/dev/null 2>&1; then
  echo "timeout command missing; cannot run zellij PTY smoke"
  fail=1
else
  rm -rf /tmp/zellij-*
  timeout 16s script -q -c 'zellij --layout ai attach --create aibox-smoke' /tmp/aibox-zellij.typescript >/tmp/aibox-zellij-pty.log 2>&1
  code=$?
  zellij kill-session aibox-smoke >/dev/null 2>&1 || true
  if [[ "${code}" -ne 0 && "${code}" -ne 124 ]]; then
    echo "zellij PTY smoke failed with ${code}"
    fail=1
  fi
  if [[ ! -s /tmp/aibox-zellij.typescript ]]; then
    echo "zellij PTY transcript is empty"
    fail=1
  fi
  find /tmp -path '*/zellij-log/zellij.log' -type f -print -exec tail -n 240 {} \; || true
  if find /tmp -path '*/zellij-log/zellij.log' -type f -print0 | xargs -0 grep -E 'ERROR IN PLUGIN|panicked|failed to load plugin' >/tmp/aibox-zellij-errors.txt 2>&1; then
    echo "zellij errors detected:"
    cat /tmp/aibox-zellij-errors.txt
    fail=1
  fi
  if find /tmp -path '*/zellij-log/zellij.log' -type f -print0 | xargs -0 grep -E 'over 1000 consecutive unknown messages|Received unknown message from client' >/tmp/aibox-zellij-shutdown-noise.txt 2>&1; then
    echo "warning: zellij logged forced-shutdown client noise after timeout:"
    cat /tmp/aibox-zellij-shutdown-noise.txt
  fi
fi

exit "${fail}"
EOF

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
