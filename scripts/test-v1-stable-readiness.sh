#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI_MANIFEST="${PROJECT_ROOT}/cli/Cargo.toml"
EVIDENCE_DIR="${PROJECT_ROOT}/.aibox/release-evidence/v1-readiness"
LOG_DIR="${EVIDENCE_DIR}/logs"

: "${RELEASE_CANDIDATE_SHA:?set RELEASE_CANDIDATE_SHA to the exact 40-character candidate commit}"
: "${AIBOX_RELEASE_BINARY_SHA256:?set AIBOX_RELEASE_BINARY_SHA256 to the tested sha256:<hex> binary digest}"

[[ "${RELEASE_CANDIDATE_SHA}" =~ ^[0-9a-f]{40}$ ]] || {
  echo "RELEASE_CANDIDATE_SHA must be a 40-character lowercase commit SHA" >&2
  exit 2
}
[[ "${AIBOX_RELEASE_BINARY_SHA256}" =~ ^sha256:[0-9a-f]{64}$ ]] || {
  echo "AIBOX_RELEASE_BINARY_SHA256 must be a sha256:<64 lowercase hex> digest" >&2
  exit 2
}
command -v jq >/dev/null || {
  echo "jq is required to record typed stable-v1 release evidence" >&2
  exit 2
}

mkdir -p "${LOG_DIR}"

record_gate() {
  local gate="$1" command_label="$2" started_at="$3" completed_at="$4" log_path="$5"
  shift 5
  local relative_log="${log_path#"${PROJECT_ROOT}/"}"
  local log_sha256="sha256:$(sha256sum "${log_path}" | awk '{print $1}')"
  local evidence_tmp="${EVIDENCE_DIR}/.${gate}.json.tmp"
  local scenarios_json
  scenarios_json="$(printf '%s\n' "$@" | jq -Rsc 'split("\n")[:-1]')"

  jq -n \
    --arg gate "${gate}" \
    --arg candidate_commit "${RELEASE_CANDIDATE_SHA}" \
    --arg binary_sha256 "${AIBOX_RELEASE_BINARY_SHA256}" \
    --arg command "${command_label}" \
    --arg started_at "${started_at}" \
    --arg completed_at "${completed_at}" \
    --arg artifact_path "${relative_log}" \
    --arg artifact_sha256 "${log_sha256}" \
    --argjson scenarios "${scenarios_json}" \
    '{
      apiVersion: "aibox.projectious.work/release-evidence/v1alpha1",
      kind: "ReleaseGateEvidence",
      gate: $gate,
      status: "passed",
      candidateCommit: $candidate_commit,
      binarySha256: $binary_sha256,
      command: $command,
      startedAt: $started_at,
      completedAt: $completed_at,
      scenarios: $scenarios,
      artifacts: [{path: $artifact_path, sha256: $artifact_sha256}]
    }' > "${evidence_tmp}"
  mv "${evidence_tmp}" "${EVIDENCE_DIR}/${gate}.json"
}

run_gate() {
  local gate="$1" command_label="$2"
  shift 2
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
  record_gate \
    "${gate}" "${command_label}" "${started_at}" "${completed_at}" "${log_path}" \
    "${scenarios[@]}"
}

run_migration_gate() {
  cargo test --manifest-path "${CLI_MANIFEST}" \
    v1_release_readiness::tests:: -- --nocapture
  cargo test --manifest-path "${CLI_MANIFEST}" \
    v1_config_migration_preview_apply_and_restore_are_explicit_and_isolated -- --nocapture
}

run_security_gate() {
  local filter
  for filter in \
    deployment_target_serializes_credential_references_without_secret_values \
    destroy_refuses_every_ownership_mismatch_and_is_idempotent_after_success \
    connection_argv_never_receives_credential_references_or_secret_canaries \
    reconciler_uses_existing_facilities_and_never_serializes_secret_canary
  do
    cargo test --manifest-path "${CLI_MANIFEST}" "${filter}" -- --nocapture
  done
  (cd "${PROJECT_ROOT}/cli" && cargo audit)
}

run_gate \
  "v0-to-v1-config-migration" \
  "scripts/test-v1-stable-readiness.sh migration" \
  "preview-without-mutation" \
  "exact-private-backup" \
  "disabled-v1-boundary" \
  "restore-v0-config" \
  "preserve-v1-deployment-records" \
  -- run_migration_gate

run_gate \
  "ownership-credentials-supply-chain-canaries" \
  "scripts/test-v1-stable-readiness.sh security" \
  "credential-reference-serialization" \
  "secret-canary-absence" \
  "complete-ownership-before-destroy" \
  "dependency-advisory-audit" \
  -- run_security_gate

m5_started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
m5_log_tmp="${LOG_DIR}/.m5-real-producer.log.tmp"
m5_log_path="${LOG_DIR}/m5-real-producer.log"
if ! "${PROJECT_ROOT}/scripts/test-processkit-v1-consumer.sh" \
  > "${m5_log_tmp}" 2>&1
then
  cat "${m5_log_tmp}" >&2
  rm -f "${m5_log_tmp}"
  exit 1
fi
m5_completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cat "${m5_log_tmp}"
mv "${m5_log_tmp}" "${m5_log_path}"

for gate in \
  m5-alpha3-exact-lifecycle \
  m5-interruption-recovery \
  m5-v0-coexistence-and-rollback \
  m5-secret-safety
do
  case "${gate}" in
    m5-alpha3-exact-lifecycle)
      scenarios=(signed-install verify no-op-update changed-update uninstall)
      ;;
    m5-interruption-recovery)
      scenarios=(interrupted-operation retry-refusal recover retry-success)
      ;;
    m5-v0-coexistence-and-rollback)
      scenarios=(v0-coexistence failed-install-rollback v1-only-uninstall)
      ;;
    m5-secret-safety)
      scenarios=(argv-canary-absence request-cleanup log-canary-absence error-canary-absence)
      ;;
  esac
  record_gate \
    "${gate}" \
    "scripts/test-processkit-v1-consumer.sh" \
    "${m5_started_at}" \
    "${m5_completed_at}" \
    "${m5_log_path}" \
    "${scenarios[@]}"
done

echo "stable-v1 producer evidence written to ${EVIDENCE_DIR}"
