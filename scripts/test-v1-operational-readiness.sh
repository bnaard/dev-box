#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI_MANIFEST="${PROJECT_ROOT}/cli/Cargo.toml"
EVIDENCE_DIR="${PROJECT_ROOT}/.aibox/release-evidence/v1-readiness"
LOG_DIR="${EVIDENCE_DIR}/logs"

: "${RELEASE_CANDIDATE_SHA:?set RELEASE_CANDIDATE_SHA to the exact candidate commit}"
: "${AIBOX_RELEASE_BINARY_SHA256:?set AIBOX_RELEASE_BINARY_SHA256 to the tested binary digest}"
[[ "${RELEASE_CANDIDATE_SHA}" =~ ^[0-9a-f]{40}$ ]] || exit 2
[[ "${AIBOX_RELEASE_BINARY_SHA256}" =~ ^sha256:[0-9a-f]{64}$ ]] || exit 2
command -v jq >/dev/null || {
  echo "jq is required for operational-readiness evidence" >&2
  exit 2
}

mkdir -p "${LOG_DIR}"

record_gate() {
  local gate="$1" started_at="$2" completed_at="$3" log_path="$4"
  shift 4
  local relative_log="${log_path#"${PROJECT_ROOT}/"}"
  local log_sha256="sha256:$(sha256sum "${log_path}" | awk '{print $1}')"
  local evidence_tmp="${EVIDENCE_DIR}/.${gate}.json.tmp"
  local scenarios_json
  scenarios_json="$(printf '%s\n' "$@" | jq -Rsc 'split("\n")[:-1]')"

  jq -n \
    --arg gate "${gate}" \
    --arg commit "${RELEASE_CANDIDATE_SHA}" \
    --arg digest "${AIBOX_RELEASE_BINARY_SHA256}" \
    --arg started "${started_at}" \
    --arg completed "${completed_at}" \
    --arg path "${relative_log}" \
    --arg artifact_digest "${log_sha256}" \
    --argjson scenarios "${scenarios_json}" \
    '{
      apiVersion: "aibox.projectious.work/release-evidence/v1alpha1",
      kind: "ReleaseGateEvidence",
      gate: $gate,
      status: "passed",
      candidateCommit: $commit,
      binarySha256: $digest,
      command: "scripts/test-v1-operational-readiness.sh",
      startedAt: $started,
      completedAt: $completed,
      scenarios: $scenarios,
      artifacts: [{path: $path, sha256: $artifact_digest}]
    }' > "${evidence_tmp}"
  mv "${evidence_tmp}" "${EVIDENCE_DIR}/${gate}.json"
}

run_gate() {
  local gate="$1"
  shift
  local scenarios=()
  while [[ "${1:-}" != "--" ]]; do
    scenarios+=("$1")
    shift
  done
  shift

  local started_at completed_at
  local log_tmp="${LOG_DIR}/.${gate}.log.tmp"
  local log_path="${LOG_DIR}/${gate}.log"
  started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if ! "$@" > "${log_tmp}" 2>&1; then
    cat "${log_tmp}" >&2
    rm -f "${log_tmp}" "${EVIDENCE_DIR}/${gate}.json"
    return 1
  fi
  completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  cat "${log_tmp}"
  mv "${log_tmp}" "${log_path}"
  record_gate "${gate}" "${started_at}" "${completed_at}" "${log_path}" "${scenarios[@]}"
}

support_policy_gate() {
  grep -Fq "v0 remains supported" \
    "${PROJECT_ROOT}/docs-site/content/docs/reference/v1-support-and-retirement.md"
  grep -Fq "Retirement is evidence-based" \
    "${PROJECT_ROOT}/docs-site/content/docs/reference/v1-support-and-retirement.md"
  grep -Fq "does not delete v1 deployments" \
    "${PROJECT_ROOT}/docs-site/content/docs/reference/v1-support-and-retirement.md"
  grep -Fq "v1.x-release" \
    "${PROJECT_ROOT}/docs-site/content/docs/contributing/maintenance.md"
  grep -Fq "evaluation release: v0 remains the supported stable line" \
    "${PROJECT_ROOT}/release-notes/v1.0.0-alpha.1.md"
  echo "support, deprecation, coexistence, rollback, and retirement policy checks passed"
}

portfolio_boundary_gate() {
  cargo test --manifest-path "${CLI_MANIFEST}" \
    processkit_protocol::tests::disabled_boundary_performs_no_discovery_or_process_invocation \
    -- --nocapture
  cargo test --manifest-path "${CLI_MANIFEST}" \
    processkit_protocol::tests::direct_boundary_matches_the_opaque_protocol_outcome \
    -- --nocapture
  cargo test --manifest-path "${CLI_MANIFEST}" \
    processkit_protocol::tests::v1_boundary_contains_no_legacy_distribution_or_layout_policy \
    -- --nocapture
  grep -Fq "Infrastructure supplied by the operator" \
    "${PROJECT_ROOT}/docs-site/content/docs/reference/v1-deployment-boundaries.md"
  grep -Fq "does not provision infrastructure" \
    "${PROJECT_ROOT}/docs-site/content/docs/reference/v1-support-and-retirement.md"
  grep -Fq "processkit owns" \
    "${PROJECT_ROOT}/docs-site/content/docs/reference/v1-support-and-retirement.md"
  echo "ainfra, aibox, and processkit portfolio boundaries passed"
}

run_gate \
  support-deprecation-retirement-policy \
  support-window \
  deprecation-notice \
  v0-coexistence \
  rollback-safety \
  evidence-based-retirement \
  -- support_policy_gate

run_gate \
  portfolio-boundary-audit \
  ainfra-provisions \
  aibox-deploys \
  processkit-owns-process-policy \
  disabled-processkit \
  direct-opaque-processkit \
  -- portfolio_boundary_gate

echo "stable-v1 operational evidence written to ${EVIDENCE_DIR}"
