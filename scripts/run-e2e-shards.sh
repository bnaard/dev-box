#!/usr/bin/env bash
# Run Tier 2 E2E in retryable shards.
#
# The addon-download tests use isolated workspace and Compose project names, so
# their three groups may run concurrently. LaTeX still runs after them because
# it owns a separate sizeable image lifecycle. A release orchestrator captures
# evidence per invocation and may rerun only a failed shard for the same
# candidate.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CLI_DIR="${PROJECT_ROOT}/cli"
SHARD="${1:-all}"
CORE_THREADS="${AIBOX_E2E_TEST_THREADS:-4}"

usage() {
  cat <<'USAGE'
Usage: ./scripts/run-e2e-shards.sh <all|core|addon|addon-languages|addon-platforms|addon-tools|latex>

Runs Tier 2 SSH-companion tests in deterministic shards:
  core   normal Tier 2 suite; excludes the two ignored heavy image builds
  addon  all three isolated download-addon groups in parallel
  addon-languages  language and documentation toolchains
  addon-platforms  cloud, infrastructure, and Kubernetes tools
  addon-tools      AI, preview, and supply-chain tools
  latex  LaTeX watcher/preview image build, one test thread
  all    core, addon, then latex (default)

The caller owns companion startup, cleanup, candidate identity, and evidence.
For release retries, rerun the failed shard against the unchanged candidate.
USAGE
}

[[ "${CORE_THREADS}" =~ ^[1-9][0-9]*$ ]] || {
  echo "AIBOX_E2E_TEST_THREADS must be a positive integer" >&2
  exit 2
}

run_core() {
  echo "[e2e-shard] core (test threads: ${CORE_THREADS})"
  (
    cd "${CLI_DIR}"
    cargo test --features e2e --test e2e -- --test-threads="${CORE_THREADS}"
  )
}

run_addon_test() {
  local group="$1"
  echo "[e2e-shard] addon-${group} started at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  (
    cd "${CLI_DIR}"
    cargo test --features e2e --test e2e \
      "addon::download_based_addons_build_with_published_defaults_${group}" \
      -- --ignored --exact --nocapture --test-threads=1
  )
  echo "[e2e-shard] addon-${group} completed at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
}

run_addon() {
  local group pid status=0
  local groups=(languages platforms tools)
  local pids=()
  echo "[e2e-shard] addon groups (parallelism: ${#groups[@]})"
  for group in "${groups[@]}"; do
    run_addon_test "${group}" &
    pids+=("$!")
  done
  for pid in "${pids[@]}"; do
    wait "${pid}" || status=1
  done
  return "${status}"
}

run_latex() {
  echo "[e2e-shard] latex (one test thread)"
  (
    cd "${CLI_DIR}"
    cargo test --features e2e --test e2e \
      latex_preview::latex_watcher_builds_and_preview_sidecar_serves_updated_pdf \
      -- --ignored --exact --test-threads=1
  )
}

case "${SHARD}" in
  all)
    run_core
    run_addon
    run_latex
    ;;
  core) run_core ;;
  addon) run_addon ;;
  addon-languages) run_addon_test languages ;;
  addon-platforms) run_addon_test platforms ;;
  addon-tools) run_addon_test tools ;;
  latex) run_latex ;;
  help|--help|-h) usage ;;
  *)
    usage >&2
    exit 2
    ;;
esac
