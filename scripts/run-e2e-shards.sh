#!/usr/bin/env bash
# Run Tier 2 E2E in retryable shards.
#
# The addon-download and LaTeX tests both build sizeable derived images on the
# single SSH companion.  They must never share that runtime with each other or
# with the normal Tier 2 suite.  `all` therefore runs core, addon, and latex in
# sequence.  A release orchestrator should capture evidence per invocation and
# may rerun only the failed shard for the same candidate.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CLI_DIR="${PROJECT_ROOT}/cli"
SHARD="${1:-all}"
CORE_THREADS="${AIBOX_E2E_TEST_THREADS:-4}"

usage() {
  cat <<'USAGE'
Usage: ./scripts/run-e2e-shards.sh <all|core|addon|latex>

Runs Tier 2 SSH-companion tests in deterministic shards:
  core   normal Tier 2 suite; excludes the two ignored heavy image builds
  addon  full download-addon composition build, one test thread
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

run_addon() {
  echo "[e2e-shard] addon (one test thread)"
  (
    cd "${CLI_DIR}"
    cargo test --features e2e --test e2e \
      addon::download_based_addons_build_with_published_defaults \
      -- --ignored --exact --test-threads=1
  )
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
  latex) run_latex ;;
  help|--help|-h) usage ;;
  *)
    usage >&2
    exit 2
    ;;
esac
